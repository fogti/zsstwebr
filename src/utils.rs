use atom_syndication::TextType;
use chrono::{naive::NaiveDate, DateTime, Utc};
use serde::Deserialize;
use std::{path::Path, time::SystemTime};
use walkdir::DirEntry;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub blog_name: String,
    pub web_root_url: String,
    pub id: String,
    pub author: String,
    pub stylesheet: String,
    #[serde(default)]
    pub x_head: String,
    #[serde(default)]
    pub x_nav: String,
    #[serde(default)]
    pub x_body_ph1: String,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PostTyp {
    Link,
    Text,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Post {
    pub title: String,

    // used for index page
    #[serde(default)]
    pub author: String,

    // used for Atom feed
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub x_head: String,
    #[serde(default)]
    pub x_nav: String,
    pub typ: PostTyp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum IndexTyp {
    Directory,
    Tag,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IndexEntry {
    pub cdate: NaiveDate,
    pub href: String,
    pub title: String,
    // used for index page
    pub author: String,
    // used for Atom feed
    pub authors: Vec<String>,
}

impl IndexEntry {
    pub fn with_post_and_etc(post: &Post, cdate: NaiveDate, lnk: &str) -> Self {
        Self {
            cdate,
            href: lnk.to_string(),
            title: post.title.clone(),
            author: post.author.clone(),
            authors: post.authors.clone(),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IndexRef {
    pub name: String,
    pub typ: IndexTyp,
}

pub struct Index {
    pub typ: IndexTyp,
    pub oidxrefs: Vec<IndexRef>,
    pub ents: Vec<IndexEntry>,
}

impl Default for Index {
    fn default() -> Self {
        Self {
            typ: IndexTyp::Directory,
            oidxrefs: Vec::new(),
            ents: Vec::new(),
        }
    }
}

impl Index {
    pub fn prepare(&mut self) {
        self.oidxrefs.sort_unstable();
        self.ents.sort_unstable();
    }
}

pub fn back_to_idx(p: &Path) -> String {
    use std::iter::{once, repeat};
    repeat("../")
        .take(p.components().count() - 1)
        .chain(once("index.html"))
        .flat_map(|i| i.chars())
        .collect()
}

pub fn needs_html_escape(text: &str) -> bool {
    text.contains(|i| matches!(i, '<' | '>' | '&'))
}

pub fn guess_text_type(text: &str) -> TextType {
    if crate::utils::needs_html_escape(text) {
        TextType::Xhtml
    } else {
        TextType::Text
    }
}

pub fn is_valid_tag(tag: &str) -> bool {
    !(tag.is_empty() || tag.contains(|i| matches!(i, '.' | '/' | '\0')))
}

pub fn is_not_hidden(entry: &DirEntry) -> bool {
    entry.depth() == 0
        || entry
            .file_name()
            .to_str()
            .map(|s| !s.starts_with('.'))
            .unwrap_or(false)
}

// source: https://users.rust-lang.org/t/convert-std-time-systemtime-to-chrono-datetime-datetime/7684/4
pub fn system_time_to_date_time(t: SystemTime) -> DateTime<Utc> {
    let (sec, nsec) = match t.duration_since(SystemTime::UNIX_EPOCH) {
        Ok(dur) => (dur.as_secs() as i64, dur.subsec_nanos()),
        Err(e) => {
            // unlikely but should be handled
            let dur = e.duration();
            let (sec, nsec) = (dur.as_secs() as i64, dur.subsec_nanos());
            if nsec == 0 {
                (-sec, 0)
            } else {
                (-sec - 1, 1_000_000_000 - nsec)
            }
        }
    };
    use chrono::TimeZone;
    Utc.timestamp(sec, nsec)
}
