use std::{collections::HashSet, fs::File, io::Write};

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

pub fn tr_folder2<P, F, T>(
    min_mtime: Option<std::time::SystemTime>,
    force_rebuild: bool,
    inp: P,
    outp: P,
    mut f: F,
) -> std::io::Result<()>
where
    P: AsRef<std::path::Path>,
    F: FnMut(&str, T, &mut std::io::BufWriter<File>, &str) -> std::io::Result<bool>,
    T: for<'de> serde::de::Deserialize<'de>,
{
    let mut crds = HashSet::new();
    let inp = inp.as_ref();
    let outp = outp.as_ref();

    for (i, fh_meta, fh_data) in glob::glob(inp.join("**/*.yaml").to_str().unwrap())
        .expect("invalid source path")
        .filter_map(ghandle_res2ok("glob"))
        .map(|i| {
            let mut fh = File::open(&i)?;
            let fh_meta = fh.metadata()?;
            let fh_data =
                readfilez::read_part_from_file(&mut fh, 0, readfilez::LengthSpec::new(None, true))?;
            std::io::Result::<_>::Ok((i, fh_meta, fh_data))
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
                std::fs::create_dir_all(x)?;
                crds.insert(x.to_path_buf());
            }
        }
        let stin = stin.to_str().expect("got invalid file name");
        if !force_rebuild {
            if let Some(min_mtime) = min_mtime {
                if let Ok(dst_meta) = std::fs::metadata(&outfilp) {
                    if let Ok(src_mtime) = fh_meta.modified() {
                        if let Ok(dst_mtime) = dst_meta.modified() {
                            if dst_mtime.duration_since(min_mtime).is_ok()
                                && dst_mtime.duration_since(src_mtime).is_ok()
                            {
                                // (min_mtime <= dst_mtime) && (src_mtime <= dst_mtime)
                                // source file, config, etc. wasn't modified since destination file was generated
                                println!("- {} [skipped]", stin);
                                continue;
                            }
                        }
                    }
                }
            }
        }
        println!("- {} ", stin);
        let fhout = std::fs::File::create(&outfilp)?;
        let mut bw = std::io::BufWriter::new(fhout);
        let fh_data: &str = std::str::from_utf8(&*fh_data).expect("file doesn't contain UTF-8");
        let fh_data_spl = fh_data.find("\n---\n").expect("unable to get file header");
        let fh_data_hdr = &fh_data[..=fh_data_spl];
        let fh_data_ctn = &fh_data[fh_data_spl + 5..];
        match f(
            stin,
            serde_yaml::from_str(fh_data_hdr).expect("unable to decode file as YAML"),
            &mut bw,
            fh_data_ctn,
        ) {
            Ok(true) => {
                bw.flush()?;
                bw.into_inner()?.sync_all()?;
            }
            Ok(false) => {
                std::mem::drop(bw);
                std::fs::remove_file(&outfilp)?;
            }
            Err(x) => {
                std::mem::drop(bw);
                std::fs::remove_file(&outfilp)?;
                return Err(x);
            }
        }
    }
    Ok(())
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

pub fn is_valid_tag(tag: &str) -> bool {
    !(tag.is_empty()
        || tag.contains(|i| match i {
            '.' | '/' | '\0' => true,
            _ => false,
        }))
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
