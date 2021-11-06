mod ofmt;
mod utils;

use std::collections::{HashMap, HashSet};
use std::{convert::TryInto, fs::File, path::Path};

fn main() {
    use crate::ofmt::{write_article_page, write_feed, write_index};
    use crate::utils::*;
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

    let mangler = Mangler::new(&[
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

    let mut mainidx = Index {
        typ: IndexTyp::Directory,
        oidxrefs: Vec::new(),
        ents: Vec::new(),
    };
    let mut tagents = HashMap::<_, Vec<_>>::new();
    let mut subents = HashMap::<_, Index>::new();

    let force_rebuild = matches.is_present("force-rebuild");
    let mut crds = HashSet::new();
    let indir = Path::new(indir);
    let outdir = Path::new(outdir);

    for dirent in walkdir::WalkDir::new(indir)
        .into_iter()
        // skip directories like .git
        .filter_entry(|e| is_not_hidden(e))
    {
        let dirent = match dirent {
            Ok(x) => x,
            Err(e) => {
                eprintln!("walkdir error: {}", e);
                continue;
            }
        };
        let fh_meta = match std::fs::metadata(dirent.path()) {
            Ok(x) => x,
            Err(x) => {
                eprintln!("stat() error @ {}: {}", dirent.path().display(), x);
                continue;
            }
        };
        if fh_meta.is_dir() {
            continue;
        }
        let fh_data = match readfilez::read_from_file(File::open(dirent.path())) {
            Ok(fh_data) => fh_data,
            Err(x) => {
                eprintln!(
                    "open() or mmap() error @ {}: {}",
                    dirent.path().display(),
                    x
                );
                continue;
            }
        };

        let fpap = dirent
            .path()
            .strip_prefix(indir)
            .expect("unable to strip path prefix")
            .with_extension("html");
        let outfilp = outdir.join(&fpap);
        let fpap: camino::Utf8PathBuf = fpap.try_into().expect("got invalid file name");
        if let Some(x) = outfilp.parent() {
            if !crds.contains(x) {
                std::fs::create_dir_all(x).expect("unable to create destination directory");
                crds.insert(x.to_path_buf());
            }
        }
        print!("- {}", fpap.as_str());
        let fh_data: &str = std::str::from_utf8(&*fh_data).expect("file doesn't contain UTF-8");
        let fh_data_spl = fh_data.find("\n---\n").expect("unable to get file header");
        let mut rd: Post =
            serde_yaml::from_str(&fh_data[..=fh_data_spl]).expect("unable to decode file as YAML");
        let content = &fh_data[fh_data_spl + 5..];
        let cdate = yz_diary_date::parse_from_utf8path(&fpap)
            .expect("file name without parsable diary date");

        let fparent = fpap
            .parent()
            .and_then(|x| if x == null_path { None } else { Some(x) });

        let (lnk, is_rel): (std::borrow::Cow<str>, bool) = match &rd.typ {
            PostTyp::Link => {
                let lnk = content.trim();
                if !(lnk.starts_with('/') || lnk.contains("://")) {
                    // relative URL, we need to prefix it with fparent
                    (
                        if let Some(x) = fparent {
                            format!("{}/{}", x.as_str(), lnk).into()
                        } else {
                            lnk.into()
                        },
                        true,
                    )
                } else {
                    (lnk.into(), false)
                }
            }
            PostTyp::Text => {
                let mut do_build = true;
                if !force_rebuild {
                    if let Some(config_mtime) = config_mtime {
                        if let Ok(dst_meta) = std::fs::metadata(&outfilp) {
                            if let Ok(src_mtime) = fh_meta.modified() {
                                if let Ok(dst_mtime) = dst_meta.modified() {
                                    if dst_mtime.duration_since(config_mtime).is_ok()
                                        && dst_mtime.duration_since(src_mtime).is_ok()
                                    {
                                        // (config_mtime <= dst_mtime) && (src_mtime <= dst_mtime)
                                        // source file, config, etc. wasn't modified since destination file was generated
                                        print!(" [rebuild skipped]");
                                        do_build = false;
                                    }
                                }
                            }
                        }
                    }
                }
                if do_build {
                    let fhout =
                        std::fs::File::create(&outfilp).expect("unable to open output file");
                    let wr = std::io::BufWriter::new(fhout);
                    if let Err(x) =
                        write_article_page(&mangler, &config, fpap.as_ref(), wr, &rd, content)
                    {
                        std::fs::remove_file(&outfilp)
                            .expect("unable to remove corrupted output file");
                        panic!(
                            "got error from write_article_page (src = {}, dst = {}): {:?}",
                            fpap.as_str(),
                            outfilp.display(),
                            x
                        );
                    }
                }
                (fpap.as_str().into(), true)
            }
        };
        println!();
        let idxent = IndexEntry::with_post_and_etc(&rd, cdate, &lnk);
        for i in std::mem::take(&mut rd.tags) {
            if is_valid_tag(&i) {
                tagents.entry(i).or_default().push(idxent.clone());
            } else {
                eprintln!("   - got invalid tag: {}", i);
            }
        }
        mainidx.ents.push(idxent);
        if let Some(x) = fparent {
            subents
                .entry(x.to_path_buf())
                .or_default()
                .ents
                .push(IndexEntry::with_post_and_etc(
                    &rd,
                    cdate,
                    if is_rel {
                        fpap.file_name().unwrap()
                    } else {
                        &lnk
                    },
                ));
        }
    }

    let mut kv: Vec<camino::Utf8PathBuf> = subents
        .keys()
        .flat_map(|i| i.ancestors())
        .map(camino::Utf8Path::to_path_buf)
        .collect();
    kv.sort_unstable();
    kv.dedup();

    for i in kv {
        if i == null_path {
            continue;
        }
        match i.parent() {
            None => &mut mainidx,
            Some(par) if par == null_path => &mut mainidx,
            Some(par) => subents.entry(par.to_path_buf()).or_default(),
        }
        .oidxrefs
        .push(IndexRef {
            name: i.file_name().unwrap().to_string(),
            typ: IndexTyp::Directory,
        });
    }

    mainidx.oidxrefs.extend(tagents.keys().map(|i| IndexRef {
        name: i.to_string(),
        typ: IndexTyp::Tag,
    }));

    mainidx.prepare();

    write_index(&config, outdir, "".as_ref(), &mainidx).expect("unable to write main-index");
    write_feed(&config, outdir, &mainidx).expect("unable to write atom feed");

    for (subdir, mut p_ents) in subents.into_iter() {
        p_ents.prepare();
        write_index(&config, outdir, subdir.as_ref(), &p_ents).expect("unable to write sub-index");
    }

    for (tag, mut p_ents) in tagents.into_iter() {
        p_ents.sort_unstable();
        write_index(
            &config,
            outdir,
            tag.as_ref(),
            &Index {
                typ: IndexTyp::Tag,
                oidxrefs: Vec::new(),
                ents: p_ents,
            },
        )
        .expect("unable to write tag-index");
    }
}
