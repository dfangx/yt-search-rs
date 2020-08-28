use prettytable::{cell, row, Row};
#[derive(Debug)]
pub struct Playlist {
    name: String,
    url: String,
    published: String,
    video_count: String,
    owner: String,
}

impl Playlist {
    pub fn new(
        name: String,
        url: String,
        published: String,
        video_count: String,
        owner: String,
    ) -> Self {
        Playlist {
            name,
            url,
            published,
            video_count,
            owner,
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }
    pub fn to_string(&self) -> String {
        format!(
            "{:<100} {:<60} {:<30} {:<10}",
            self.name, self.owner, self.published, self.video_count
        )
    }

    pub fn to_row(&self, i: usize) -> Row {
        row![
            format!("P{}", i),
            self.name,
            self.owner,
            self.published,
            self.video_count
        ]
    }
}

#[derive(Debug)]
pub struct Video {
    name: String,
    length: String,
    url: String,
    owner: String,
    published: String,
}

impl Video {
    pub fn new(
        name: String,
        length: String,
        url: String,
        owner: String,
        published: String,
    ) -> Self {
        Video {
            name,
            length,
            url,
            owner,
            published,
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn to_string(&self) -> String {
        format!(
            "{:<100} {:<60} {:<30} {:<10}",
            self.name, self.owner, self.published, self.length
        )
    }

    pub fn to_row(&self, i: usize) -> Row {
        row![
            format!("V{}", i),
            self.name,
            self.owner,
            self.published,
            self.length
        ]
    }
}
