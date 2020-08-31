# yt-search-rs [![Build Status](https://travis-ci.com/dfangx/yt-search-rs.svg?branch=master)](https://travis-ci.com/dfangx/yt-search-rs)

A Youtube search CLI tool in Rust. Simply runs a search on a given search term
and outputs to stdout. This way, output can be piped or utilized in scripts.
This project is in its early stage, so fine tuning and additional CLI options
to come.

## Build 

Build using `cargo build --release`. You can find the resulting binary in
`./target/release/`. Copy the `yt-search` binary into your $PATH.

## Usage 

    yt-search 0.1.0

    USAGE:
        yt-search [FLAGS] [OPTIONS] <SEARCH_TERM>

    FLAGS:
        -h, --help           Prints help information
        -i, --interactive    Include this flag if you want to be able to select a search result
        -u, --url-only       Include this flag to output the url of selected result only. If not interactive, this does
                             nothing
        -V, --version        Prints version information

    OPTIONS:
        -b, --bin <bin>          Specify the binary to be used for interaction. If not provided, uses stdout/stdin
        -f, --filter <filter>    Search filter to apply [default: None]  [possible values: Video, Playlist, None]
        -p, --pages <pages>      Number of pages to search for [default: 3]
        -s, --sort <sort>        Sort to apply [default: Relevance]  [possible values: Relevance, UploadDate, ViewCount,
                                 Rating]

    ARGS:
        <SEARCH_TERM>    Words used for the search
