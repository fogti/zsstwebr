use std::path::Path;
use walkdir::DirEntry;

pub fn ghandle_res2ok<T, E>(nam: &'static str) -> impl Fn(Result<T, E>) -> Option<T>
where
    E: std::error::Error,
{
    move |i| match i {
        Ok(x) => Some(x),
        Err(e) => {
            eprintln!("{} error: {}", nam, e);
            None
        }
    }
}

pub fn back_to_idx<P: AsRef<Path>>(p: P) -> String {
    let ccnt = p.as_ref().components().count() - 1;
    let mut ret = String::with_capacity(ccnt * 3 + 10);
    for _ in 0..ccnt {
        ret += "../";
    }
    ret += "index.html";
    ret
}

pub fn is_valid_tag(tag: &str) -> bool {
    !(tag.is_empty() || tag.contains(|i| matches!(i, '.' | '/' | '\0')))
}

pub fn fmt_article_link(rd: &crate::Post, lnk: &str) -> String {
    let mut ent_str = format!(
        "{}: <a href=\"{}\">{}</a>",
        rd.cdate.format("%d.%m.%Y"),
        lnk,
        rd.title
    );
    if !rd.author.is_empty() {
        ent_str += " <span class=\"authorspec\">by ";
        ent_str += &rd.author;
        ent_str += "</span>";
    }
    ent_str
}

pub fn is_not_hidden(entry: &DirEntry) -> bool {
    entry.depth() == 0
        || entry
            .file_name()
            .to_str()
            .map(|s| !s.starts_with('.'))
            .unwrap_or(false)
}
