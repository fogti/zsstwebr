use aho_corasick::{AhoCorasick, AhoCorasickBuilder};
use atom_syndication::TextType;
use chrono::{naive::NaiveDate, DateTime, Utc};
use serde::Deserialize;
use std::{path::Path, time::SystemTime};
use walkdir::DirEntry;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub blog_name: String,
    pub id: url::Url,
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

pub fn guess_text_type(text: &str) -> TextType {
    if text.contains(|i| matches!(i, '<' | '>' | '&')) {
        TextType::Html
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
    Utc.timestamp_opt(sec, nsec).unwrap()
}

/// blog content mangler (inserts paragraph tags)
pub struct Mangler {
    ahos: AhoCorasick,
}

fn diiter<T>(a: T, b: T) -> impl Iterator<Item = T> {
    use core::iter::once;
    once(a).chain(once(b))
}

struct SectionState<'i> {
    do_mangle: bool,
    section: core::str::Lines<'i>,
}

pub struct MangleIter<'a, 'i> {
    ahos: &'a AhoCorasick,
    input: core::str::Split<'i, &'static str>,
    state: Option<SectionState<'i>>,
}

impl<'a, 'i> Iterator for MangleIter<'a, 'i> {
    type Item = (bool, &'i str);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.state.take() {
                None => {
                    let section = self.input.next()?;
                    let do_mangle = !self.ahos.is_match(section);
                    self.state = Some(SectionState {
                        do_mangle,
                        section: section.lines(),
                    });
                    if do_mangle {
                        break Some((true, "<p>"));
                    }
                }
                Some(SectionState {
                    do_mangle,
                    mut section,
                }) => {
                    if let Some(x) = section.next() {
                        self.state = Some(SectionState { do_mangle, section });
                        break Some((do_mangle, x));
                    } else if do_mangle {
                        break Some((true, "</p>"));
                    }
                }
            }
        }
    }
}

impl Mangler {
    pub fn new(dont_mangle: &[&str]) -> Mangler {
        let pats: Vec<_> = dont_mangle
            .iter()
            .flat_map(|&i| diiter("<".to_string() + i + ">", "</".to_string() + i + ">"))
            .collect();
        Mangler {
            ahos: AhoCorasickBuilder::new()
                .build(&pats)
                .expect("unable to build mangle filter"),
        }
    }

    /// You should only prepend each line with spaces if the associated $mangle boolean is 'true'.
    pub fn mangle_content<'a, 'i>(&'a self, input: &'i str) -> MangleIter<'a, 'i> {
        MangleIter {
            ahos: &self.ahos,
            input: input.split("\n\n"),
            state: None,
        }
    }
}
