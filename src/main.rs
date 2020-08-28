mod data;

use data::{Playlist, Video};
use prettytable::{cell, row, Table};
use reqwest::Client;
use std::{
    io::prelude::*,
    process::{Command, Stdio},
    sync::Arc,
};
use structopt::{clap::arg_enum, StructOpt};
use tokio::task::JoinHandle;

const BASE_URL: &str = "https://youtube.com";

#[tokio::main]
async fn main() -> Result<(), reqwest::Error> {
    let opts = Opts::from_args();
    let process = if let Some(ref bin) = opts.bin {
        if opts.interactive {
            Command::new(bin)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .ok()
        } else {
            None
        }
    } else {
        None
    };

    let search_handles = construct_handles(&opts);
    let data_list = futures::future::join_all(search_handles).await;
    let (video_list, playlist_list) = data_list.into_iter().fold(
        (Vec::new(), Vec::new()),
        |(mut video_list, mut playlist_list), page| {
            let (mut video_by_page, mut playlist_by_page) = page.unwrap().unwrap();
            video_list.append(&mut video_by_page);
            playlist_list.append(&mut playlist_by_page);
            (video_list, playlist_list)
        },
    );
    let (videos_as_string, playlists_as_string) = data_to_string(&video_list, &playlist_list);

    if let Some(mut process) = process {
        data_pipe_to_process(&mut process, videos_as_string, playlists_as_string);
        let selection = get_selection(process);
        selection.and_then(|selection| {
            if !selection.is_empty() {
                print_selection(video_list, playlist_list, selection, opts.url_only);
            }
            Some("")
        });
    } else {
        output(&video_list, &playlist_list);
        if opts.interactive {
            let selection = input();
            if !selection.is_empty() {
                print_selection(video_list, playlist_list, selection, opts.url_only);
            }
        }
    }
    Ok(())
}

fn get_selection(process: std::process::Child) -> Option<String> {
    let selection = match process.wait_with_output() {
        Ok(output) => String::from_utf8(output.stdout).ok(),
        Err(_) => None,
    };
    //let selection = String::from_utf8(output.stdout).ok();
    selection
}

fn data_pipe_to_process(
    process: &mut std::process::Child,
    videos_as_string: Vec<String>,
    playlists_as_string: Vec<String>,
) {
    for video in videos_as_string {
        match process.stdin {
            Some(ref mut input) => match input.write_all(&video.as_bytes()) {
                Ok(_) => {}
                Err(e) => panic!("Could not send to child process: {}", e),
            },
            None => panic!("Something went wrong"),
        }
    }
    for pl in playlists_as_string {
        match process.stdin {
            Some(ref mut input) => match input.write_all(&pl.as_bytes()) {
                Ok(_) => {}
                Err(e) => panic!("Could not send to child process: {}", e),
            },
            None => panic!("Something went wrong"),
        }
    }
}

fn data_to_string(
    video_list: &Vec<Video>,
    playlist_list: &Vec<Playlist>,
) -> (Vec<String>, Vec<String>) {
    let videos_as_string: Vec<String> = video_list
        .iter()
        .enumerate()
        .map(|(i, video)| format!("V{} {}\n", i, video.to_string()))
        .collect();
    let playlists_as_string: Vec<String> = playlist_list
        .iter()
        .enumerate()
        .map(|(i, pl)| format!("P{} {}\n", i, pl.to_string()))
        .collect();
    (videos_as_string, playlists_as_string)
}
async fn search_and_parse(
    params: Vec<(&str, String)>,
    client: Arc<Client>,
) -> Result<(Vec<Video>, Vec<Playlist>), reqwest::Error> {
    let rsp = make_request(params, client).await?;
    let html = rsp.text().await?;
    Ok(parse_rsp(html).await?)
}

fn construct_handles(
    opts: &Opts,
) -> Vec<JoinHandle<Result<(Vec<Video>, Vec<Playlist>), reqwest::Error>>> {
    let client = Arc::new(Client::new());
    let mut handles = vec![];
    let filter = match opts.filter {
        YTFilter::Video => String::from("EgIQAQ%3D%3D"),
        YTFilter::Channel => String::from("EgIQAg%3D%3D"),
        YTFilter::Playlist => String::from("EgIQAw%3D%3D"),
        _ => String::new(),
    };

    for p in 1..=opts.pages {
        let p = p.to_string();
        let mut params = vec![
            ("search_query", opts.search_term.clone()),
            ("p", p.to_string()),
        ];
        if !filter.is_empty() {
            params.push(("sp", filter.clone()));
        }
        handles.push(tokio::spawn(search_and_parse(params, client.clone())));
    }
    handles
}

fn extract_json(html: String) -> json::JsonValue {
    let data = &html[html.find("ytInitialData").unwrap() + 17..];
    let json = json::parse(&data[0..data.find("window[\"ytInitialPlayerResponse\"]").unwrap() - 6])
        .unwrap();
    let content = &json["contents"]["twoColumnSearchResultsRenderer"]["primaryContents"]
        ["sectionListRenderer"]["contents"][0]["itemSectionRenderer"]["contents"];
    content.to_owned()
}

fn match_item_type(content: json::JsonValue) -> (Vec<Video>, Vec<Playlist>) {
    let mut video_list = vec![];
    let mut playlist_list = vec![];

    for item in content.members() {
        if item.has_key("videoRenderer") {
            video_list.push(create_video_item(&item["videoRenderer"]));
        }
        if item.has_key("playlistRenderer") {
            playlist_list.push(create_playlist_item(&item["playlistRenderer"]));
        }
    }

    (video_list, playlist_list)
}

async fn parse_rsp(html: String) -> Result<(Vec<Video>, Vec<Playlist>), reqwest::Error> {
    let content = extract_json(html);
    let (video_list, playlist_list) = match_item_type(content);
    Ok((video_list, playlist_list))
}

fn print_selection(
    video_list: Vec<Video>,
    playlist_list: Vec<Playlist>,
    selection: String,
    url_only: bool,
) {
    let id = selection.split(' ').collect::<Vec<&str>>()[0];
    let index = id[1..].parse::<usize>().unwrap();
    if id.to_lowercase().starts_with('p') {
        let pl = &playlist_list[index];
        if url_only {
            println!("{}{}", BASE_URL, pl.url());
        } else {
            println!("{}", pl.to_string());
        }
    } else if id.to_lowercase().starts_with('v') {
        let video = &video_list[index];
        if url_only {
            println!("{}{}", BASE_URL, video.url());
        } else {
            println!("{}", video.to_string());
        }
    }
}

fn create_playlist_item(item: &json::JsonValue) -> Playlist {
    let name = item["title"]["simpleText"]
        .dump()
        .trim_matches('"')
        .to_string();
    let url = item["navigationEndpoint"]["commandMetadata"]["webCommandMetadata"]["url"]
        .dump()
        .trim_matches('"')
        .to_string();
    let published = item["publishedTimeText"]["simpleText"]
        .dump()
        .trim_matches('"')
        .to_string();
    let video_count = item["videoCountText"]["runs"][0]["text"]
        .dump()
        .trim_matches('"')
        .to_string();
    let owner = item["shortBylineText"]["runs"][0]["text"]
        .dump()
        .trim_matches('"')
        .to_string();
    let pl = Playlist::new(name, url, published, video_count, owner);
    pl
}

fn create_video_item(item: &json::JsonValue) -> Video {
    let name = item["title"]["runs"][0]["text"]
        .dump()
        .trim_matches('"')
        .to_string();
    let length = item["lengthText"]["simpleText"]
        .dump()
        .trim_matches('"')
        .to_string();
    let url = item["navigationEndpoint"]["commandMetadata"]["webCommandMetadata"]["url"]
        .dump()
        .trim_matches('"')
        .to_string();
    let owner = item["ownerText"]["runs"][0]["text"]
        .dump()
        .trim_matches('"')
        .to_string();
    let published = item["publishedTimeText"]["simpleText"]
        .dump()
        .trim_matches('"')
        .to_string();
    let video = Video::new(name, length, url, owner, published);
    video
}

fn input() -> String {
    let mut line = String::new();
    while line.is_empty()
        || line[1..].trim().parse::<usize>().is_err()
            && (!line.to_lowercase().starts_with('p') || !line.to_lowercase().starts_with('v'))
    {
        line = String::new();
        println!(
            "Enter the ID of your selection. You will be prompted again if an invalid input is given:"
        );
        std::io::stdin().read_line(&mut line).unwrap();
    }
    line.trim().to_string()
}

fn output(video_list: &Vec<Video>, playlist_list: &Vec<Playlist>) {
    if !video_list.is_empty() {
        display_videos(video_list);
        println!("");
    }

    if !playlist_list.is_empty() {
        display_playlists(playlist_list);
        println!("");
    }
}

fn display_videos(video_list: &Vec<Video>) {
    println!("VIDEOS");
    let mut table = Table::new();
    table.set_titles(row![
        "ID",
        "Video Name",
        "Video Owner",
        "Publishing Date",
        "Duration"
    ]);
    for (i, video) in video_list.iter().enumerate() {
        table.add_row(video.to_row(i));
    }
    table.printstd();
    //println!(
    //    "{:<120} {:<40} {:<20} {:<10}",
    //    "Video Name", "Video Owner", "Publishing Date", "Duration"
    //);
    //for video in video_list {
    //    println!("{}", video.trim());
    //}
}

fn display_playlists(playlist_list: &Vec<Playlist>) {
    //println!("PLAYLISTS");
    let mut table = Table::new();
    table.set_titles(row![
        "ID",
        "Playlist Name",
        "Playlist Owner",
        "Publishing Date",
        "Video Count"
    ]);
    for (i, pl) in playlist_list.iter().enumerate() {
        table.add_row(pl.to_row(i));
    }
    table.printstd();
    //println!(
    //    "{:<120} {:<40} {:<20} {:<10}",
    //    "Playlist Name", "Playlist Owner", "Publishing Date", "Video Count"
    //);
    //for pl in playlist_list {
    //    println!("{}", pl.trim());
    //}
}
async fn make_request(
    params: Vec<(&str, String)>,
    client: Arc<Client>,
) -> reqwest::Result<reqwest::Response> {
    client
        .get(&(BASE_URL.to_owned() + "/results"))
        .query(&params)
        .send()
        .await
}

arg_enum! {
#[derive(Debug)]
    enum YTFilter {
        Video,
        Playlist,
        Channel,
        None,
    }
}

arg_enum! {
#[derive(Debug)]
    enum InteractiveMethod {
        Stdin,
        Fzf,
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "yt-search")]
struct Opts {
    #[structopt(short, long)]
    interactive: bool,
    #[structopt(short, long, required_if("interactive", "true"))]
    bin: Option<String>,
    #[structopt(short, long)]
    url_only: bool,
    #[structopt(short, long, default_value = "3")]
    pages: u32,
    #[structopt(short, long, default_value = "None", possible_values = &YTFilter::variants(), case_insensitive = true)]
    filter: YTFilter,
    #[structopt(name = "SEARCH_TERM")]
    search_term: String,
}
