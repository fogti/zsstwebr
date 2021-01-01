use std::path::Path;
use walkdir::DirEntry;

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

pub fn is_not_hidden(entry: &DirEntry) -> bool {
    entry.depth() == 0
        || entry
            .file_name()
            .to_str()
            .map(|s| !s.starts_with('.'))
            .unwrap_or(false)
}
