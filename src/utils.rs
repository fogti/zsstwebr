use chrono::naive::NaiveDate;
use serde::Deserialize;
use std::path::Path;
use walkdir::DirEntry;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub blog_name: String,
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
    pub cdate: NaiveDate,
    pub title: String,
    #[serde(default)]
    pub author: String,
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
    pub author: String,
}

impl IndexEntry {
    pub fn with_post_and_link(post: &Post, lnk: &str) -> Self {
        Self {
            cdate: post.cdate,
            href: lnk.to_string(),
            title: post.title.clone(),
            author: post.author.clone(),
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

pub fn back_to_idx(p: &Path) -> String {
    use std::iter::{once, repeat};
    repeat("../")
        .take(p.components().count() - 1)
        .chain(once("index.html"))
        .flat_map(|i| i.chars())
        .collect()
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
