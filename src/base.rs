use chrono::prelude::*;
use serde::Deserialize;
use std::{collections::HashSet, fs::File};

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub blog_name: String,
    pub stylesheet: String,
    pub x_head: String,
    pub x_nav: String,
    pub x_body_ph1: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case", tag = "typ", content = "c")]
pub enum PostData {
    Link(String),
    Text(String),
}

#[derive(Clone, Debug, Deserialize)]
pub struct Post {
    pub cdate: NaiveDate,
    pub title: String,
    pub data: PostData,
    pub tags: Vec<String>,
    pub x_head: String,
}

fn ghandle_res2ok<T, E>(nam: &'static str) -> impl Fn(Result<T, E>) -> Option<T>
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

pub fn tr_folder2<P, F, T>(inp: P, outp: P, mut f: F)
where
    P: AsRef<std::path::Path>,
    F: FnMut(&str, T, std::io::BufWriter<File>),
    T: for<'de> serde::de::Deserialize<'de>,
{
    let mut crds = HashSet::new();
    let inp = inp.as_ref();
    let outp = outp.as_ref();

    for (i, fh_data) in glob::glob(inp.join("**/*.yaml").to_str().unwrap())
        .expect("invalid source path")
        .filter_map(ghandle_res2ok("glob"))
        .map(|i| {
            let mut fh = File::open(&i)?;
            let fh_data =
                readfilez::read_part_from_file(&mut fh, 0, readfilez::LengthSpec::new(None, true))?;
            std::io::Result::<_>::Ok((i, fh_data))
        })
        .filter_map(ghandle_res2ok("file open"))
    {
        let stin = i
            .strip_prefix(inp)
            .expect("unable to strip path prefix")
            .with_extension("html");
        let outfilp = outp.join(&stin);
        if let Some(x) = outfilp.parent() {
            if !crds.contains(x) {
                std::fs::create_dir_all(x).expect("unable to create output directory");
                crds.insert(x.to_path_buf());
            }
        }
        let fhout = std::fs::File::create(&outfilp).expect("unable to create output file");
        f(
            stin.to_str().expect("got invalid file name"),
            serde_yaml::from_slice(&*fh_data).expect("unable to decode file as YAML"),
            std::io::BufWriter::new(fhout),
        );
    }
}

pub fn back_to_idx<P: AsRef<std::path::Path>>(p: P) -> String {
    let ccnt = p.as_ref().components().count() - 1;
    let mut ret = String::with_capacity(ccnt * 3 + 10);
    for _ in 0..ccnt {
        ret += "../";
    }
    ret += "index.html";
    ret
}
