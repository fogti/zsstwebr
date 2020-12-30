mod mangle;
mod ofmt;
mod utils;

use chrono::prelude::*;
use serde::Deserialize;
use std::{collections::HashMap, fs::File, path::Path};

use crate::ofmt::{write_article_page, write_index, write_tag_index};
use crate::utils::*;

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

#[derive(Clone, Debug, Deserialize)]
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

fn main() {
    use clap::Arg;

    let null_path = Path::new("");

    let matches = clap::App::new("zsstwebr")
        .version(clap::crate_version!())
        .author("Erik Zscheile <erik.zscheile@gmail.com>")
        .about("a blog post renderer")
        .arg(
            Arg::with_name("INPUT_DIR")
                .help("sets the input directory")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("output_dir")
                .short("o")
                .long("output-dir")
                .takes_value(true)
                .required(true)
                .help("sets the output directory"),
        )
        .arg(
            Arg::with_name("config")
                .long("config")
                .takes_value(true)
                .required(true)
                .help("sets the config file path"),
        )
        .arg(
            Arg::with_name("force-rebuild")
                .short("f")
                .long("force-rebuild")
                .help("force overwriting of destination files even if the source files weren't modified")
        )
        .get_matches();

    let mangler = mangle::Mangler::new(&[
        "address",
        "article",
        "aside",
        "blockquote",
        "code",
        "div",
        "dl",
        "fieldset",
        "footer",
        "form",
        "h1",
        "h2",
        "h3",
        "h4",
        "h5",
        "h6",
        "header",
        "hr",
        "menu",
        "nav",
        "ol",
        "p",
        "pre",
        "section",
        "table",
        "tt",
        "ul",
    ]);

    let indir = matches.value_of("INPUT_DIR").unwrap();
    let outdir = matches.value_of("output_dir").unwrap();
    std::fs::create_dir_all(&outdir).expect("unable to create output directory");

    let (config, config_mtime): (Config, Option<_>) = {
        let mut fh =
            File::open(matches.value_of("config").unwrap()).expect("unable to open config file");
        let config_mtime = fh
            .metadata()
            .expect("unable to get config file stat()")
            .modified()
            .ok();
        let fh_data =
            readfilez::read_part_from_file(&mut fh, 0, readfilez::LengthSpec::new(None, true))
                .expect("unable to read config file");
        (
            serde_yaml::from_slice(&*fh_data).expect("unable to decode file as YAML"),
            config_mtime,
        )
    };

    let mut ents = Vec::new();
    let mut tagents = HashMap::<_, Vec<_>>::new();
    let mut subents = HashMap::<_, Vec<_>>::new();

    tr_folder2(
        config_mtime,
        matches.is_present("force-rebuild"),
        indir,
        &outdir,
        |fpath, mut rd: Post, mut wr, content| {
            let (lnk, ret): (&str, bool) = match &rd.typ {
                PostTyp::Link => (content.trim(), false),
                PostTyp::Text => {
                    write_article_page(&mangler, &config, fpath.as_ref(), &mut wr, &rd, &content)?;
                    (fpath, true)
                }
            };
            let ent_str = fmt_article_link(&rd, lnk);
            for i in std::mem::take(&mut rd.tags) {
                if is_valid_tag(&i) {
                    tagents.entry(i).or_default().push(ent_str.clone());
                } else {
                    eprintln!("   - got invalid tag: {}", i);
                }
            }
            ents.push(ent_str);
            let fpap = Path::new(fpath);
            if let Some(x) = fpap
                .parent()
                .and_then(|x| if x == null_path { None } else { Some(x) })
            {
                let bname = fpap.file_name().unwrap();
                subents
                    .entry(x.to_path_buf())
                    .or_default()
                    .push(fmt_article_link(
                        &rd,
                        if lnk == fpath {
                            bname.to_str().unwrap()
                        } else {
                            lnk
                        },
                    ));
            }
            Ok(ret)
        },
    )
    .expect("I/O error while transforming dirs");

    let mut kv: Vec<std::path::PathBuf> = subents
        .keys()
        .flat_map(|i| i.ancestors())
        .map(Path::to_path_buf)
        .collect();
    kv.sort();
    kv.dedup();

    for i in kv {
        if i == null_path {
            continue;
        }
        let ibn = i.file_name().unwrap().to_str().unwrap();
        match i.parent() {
            None => &mut ents,
            Some(par) if par == null_path => &mut ents,
            Some(par) => subents.entry(par.to_path_buf()).or_default(),
        }
        .push(format!("<a href=\"{}/index.html\">{}</a>", ibn, ibn));
    }

    let mut tags: Vec<_> = tagents.keys().collect();
    tags.sort_unstable_by(|a, b| a.cmp(b).reverse());
    for tag in tags {
        ents.push(format!(
            "<a href=\"{}.html\">{}</a>",
            tag.replace('&', "&amp;"),
            tag
        ));
    }

    write_index(&config, outdir.as_ref(), "".as_ref(), &ents).expect("unable to write main-index");

    for (subdir, p_ents) in subents.iter() {
        write_index(&config, outdir.as_ref(), subdir.as_ref(), &p_ents)
            .expect("unable to write sub-index");
    }

    for (tag, p_ents) in tagents.iter() {
        write_tag_index(&config, outdir.as_ref(), tag, &p_ents).expect("unable to write tag-index");
    }
}
