mod base;
mod mangle;

use std::{collections::HashMap, fs::File, io::Write, path::Path};

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
        .get_matches();

    let mangler = mangle::Mangler::new(vec![
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

    let config: base::Config = {
        let mut fh =
            File::open(matches.value_of("config").unwrap()).expect("unable to open config file");
        let fh_data =
            readfilez::read_part_from_file(&mut fh, 0, readfilez::LengthSpec::new(None, true))
                .expect("unable to read config file");
        serde_yaml::from_slice(&*fh_data).expect("unable to decode file as YAML")
    };

    let mut ents = Vec::new();
    let mut tagents = HashMap::<_, Vec<_>>::new();
    let mut subents = HashMap::<_, Vec<_>>::new();

    base::tr_folder2(indir, &outdir, |fpath, mut rd: base::Post, mut wr| {
        let (lnk, ret): (&str, bool) = match &rd.data {
            base::PostData::Link(ref l) => (&l, false),
            base::PostData::Text(ref t) => {
                writeln!(
                    &mut wr,
                    r##"<!doctype html>
<html lang="de" dir="ltr">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <link rel="stylesheet" href="{}" type="text/css" />
    <title>{} &mdash; {}</title>
{}{}  </head>
  <body>
    <h1>{}</h1>
{}    <a href="#" onclick="window.history.back()">Zur&uuml;ck zur vorherigen Seite</a> - <a href="{}">Zur&uuml;ck zur Hauptseite</a>{}<br />
"##,
    &config.stylesheet,
    &rd.title, &config.blog_name,
&config.x_head, &rd.x_head,
&rd.title,
&config.x_body_ph1,
base::back_to_idx(fpath), &config.x_nav,
).unwrap();
                for (do_mangle, i) in mangler.mangle_content(t) {
                    if do_mangle {
                        write!(&mut wr, "    ").unwrap();
                    }
                    writeln!(&mut wr, "{}", i).unwrap();
                }
                writeln!(&mut wr, "  </body>\n</html>").unwrap();
                (fpath, true)
            }
        };
        let cdatef = rd.cdate.format("%d.%m.%Y");
        let ent_str = format!("{}: <a href=\"{}\">{}</a>", &cdatef, lnk, &rd.title);
        for i in std::mem::take(&mut rd.tags).into_iter() {
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
            subents.entry(x.to_path_buf()).or_default().push(format!(
                "{}: <a href=\"{}\">{}</a>",
                &cdatef,
                if lnk == fpath {
                    bname.to_str().unwrap()
                } else {
                    lnk
                },
                &rd.title
            ));
        }
        ret
    });

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

    for tag in tagents.keys() {
        ents.push(format!(
            "<a href=\"{}.html\">{}</a>",
            tag.replace('&', "&amp;"),
            tag
        ));
    }

    write_index(&config, outdir, "", &ents).expect("unable to write main-index");

    for (subdir, p_ents) in subents.iter() {
        write_index(&config, outdir, subdir, &p_ents).expect("unable to write sub-index");
    }

    for (tag, p_ents) in tagents.iter() {
        write_tag_index(&config, outdir.as_ref(), tag, &p_ents).expect("unable to write tag-index");
    }
}

fn is_valid_tag(tag: &str) -> bool {
    !(tag.is_empty()
        || tag.contains(|i| match i {
            '.' | '/' | '\0' => true,
            _ => false,
        }))
}

fn write_index<P1, P2>(
    config: &base::Config,
    outdir: P1,
    idx_name: P2,
    ents: &[String],
) -> std::io::Result<()>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    write_index_inner(config, outdir.as_ref(), idx_name.as_ref(), ents)
}

fn write_index_inner(
    config: &base::Config,
    outdir: &Path,
    idx_name: &Path,
    ents: &[String],
) -> std::io::Result<()> {
    println!("- index: {}", idx_name.display());

    let mut f = std::io::BufWriter::new(std::fs::File::create(
        Path::new(outdir).join(idx_name).join("index.html"),
    )?);

    let is_main_idx = idx_name.to_str().map(|i| i.is_empty()) == Some(true);

    let (it_pre, it_post) = if is_main_idx {
        ("", "")
    } else {
        ("Ordner: ", " &mdash; ")
    };

    write!(
        &mut f,
        r#"<!doctype html>
<html lang="de" dir="ltr">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <link rel="stylesheet" href="{}" type="text/css" />
    <title>{}{}{}{}</title>
{}  </head>
  <body>
    <h1>{}{}{}{}</h1>
{}
<tt>
"#,
        &config.stylesheet,
        it_pre,
        idx_name.to_str().unwrap(),
        it_post,
        &config.blog_name,
        &config.x_head,
        it_pre,
        idx_name.to_str().unwrap(),
        it_post,
        &config.blog_name,
        &config.x_body_ph1,
    )?;

    if !is_main_idx {
        writeln!(
            &mut f,
            "<a href=\"..\">[&Uuml;bergeordneter Ordner]</a><br />"
        )?;
    }

    for i in ents.iter().rev() {
        writeln!(&mut f, "{}<br />", i)?;
    }

    writeln!(&mut f, "</tt>\n  </body>\n</html>")?;

    f.flush()?;
    Ok(())
}

fn write_tag_index(
    config: &base::Config,
    outdir: &Path,
    idx_name: &str,
    ents: &[String],
) -> std::io::Result<()> {
    println!("- tag index: {}", &idx_name);

    let mut fpath = Path::new(outdir).join(idx_name);
    fpath.set_extension("html");
    let mut f = std::io::BufWriter::new(std::fs::File::create(fpath)?);

    write!(
        &mut f,
        r#"<!doctype html>
<html lang="de" dir="ltr">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <link rel="stylesheet" href="{}" type="text/css" />
    <title>Tag: {} &mdash; {}</title>
{}  </head>
  <body>
    <h1>Tag: {} &mdash; {}</h1>
{}
<tt>
<a href="index.html">[Hauptseite]</a><br />
"#,
        &config.stylesheet,
        &idx_name,
        &config.blog_name,
        &config.x_head,
        &idx_name,
        &config.blog_name,
        &config.x_body_ph1,
    )?;

    for i in ents.iter().rev() {
        writeln!(&mut f, "{}<br />", i)?;
    }

    writeln!(&mut f, "</tt>\n  </body>\n</html>")?;
    f.flush()?;
    Ok(())
}
