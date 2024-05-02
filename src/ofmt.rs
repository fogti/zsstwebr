use crate::utils::{back_to_idx, guess_text_type, Config, Index, IndexTyp, Mangler, Post};
use atom_syndication::Text;
use std::io::{Result, Write};
use std::path::Path;

const OIDXREFS_LINE_MAXLEN: usize = 100;

pub fn write_article_page<W: Write>(
    mangler: &Mangler,
    config: &Config,
    fpath: &Path,
    mut wr: W,
    rd: &Post,
    content: &str,
) -> Result<()> {
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
{}    <a href="#" onclick="window.history.back()">Zur&uuml;ck zur vorherigen Seite</a> - <a href="{}">Zur&uuml;ck zur Hauptseite</a>{}"##,
        config.stylesheet,
        rd.title,
        config.blog_name,
        config.x_head,
        rd.x_head,
        rd.title,
        config.x_body_ph1,
        back_to_idx(fpath),
        config.x_nav,
    )?;
    if !rd.x_nav.is_empty() {
        write!(&mut wr, " - {}", rd.x_nav)?;
    }
    write!(&mut wr, "<br />")?;
    let mut it = mangler.mangle_content(content);
    if let Some((do_mangle, i)) = it.next() {
        if do_mangle {
            write!(&mut wr, "\n    ")
        } else {
            writeln!(&mut wr, "<br />")
        }?;
        writeln!(&mut wr, "{}", i)?;
    }
    for (do_mangle, i) in it {
        if do_mangle {
            write!(&mut wr, "    ")?;
        }
        writeln!(&mut wr, "{}", i)?;
    }
    if !rd.author.is_empty() {
        writeln!(&mut wr, "    <p>Autor: {}</p>", rd.author)?;
    }
    writeln!(&mut wr, "  </body>\n</html>")?;
    wr.flush()?;
    Ok(())
}

pub fn write_index(
    config: &Config,
    outdir: &Path,
    idx_name: &Path,
    data: &Index,
) -> std::io::Result<()> {
    println!("- index: {}", idx_name.display());

    let mut fpath = Path::new(outdir).join(idx_name);
    let (it_pre, up) = match data.typ {
        IndexTyp::Directory => {
            fpath = fpath.join("index.html");
            if idx_name.to_str().map(|i| i.is_empty()).unwrap_or(false) {
                ("", "")
            } else {
                ("Ordner: ", "<a href=\"..\">[Übergeordneter Ordner]</a>")
            }
        }
        IndexTyp::Tag => {
            fpath.set_extension("html");
            ("Tag: ", "<a href=\"index.html\">[Hauptseite]</a>")
        }
    };
    let it_post = if it_pre.is_empty() { "" } else { " &mdash; " };

    let mut f = std::io::BufWriter::new(std::fs::File::create(fpath)?);
    let idx_name_s = idx_name.to_str().unwrap();

    write!(
        &mut f,
        r#"<!doctype html>
<html lang="de" dir="ltr">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <link rel="stylesheet" href="{}" type="text/css" />
{}
    <title>{}{}{}{}</title>
{}  </head>
  <body>
    <h1>{}{}{}{}</h1>
{}
<tt>
"#,
        &config.stylesheet,
        if it_pre.is_empty() {
            r#"    <link rel="alternate" type="application/atom+xml" title="Atom feed" href="feed.atom" />
"#
        } else {
            ""
        },
        it_pre,
        idx_name_s,
        it_post,
        &config.blog_name,
        &config.x_head,
        it_pre,
        idx_name_s,
        it_post,
        &config.blog_name,
        &config.x_body_ph1,
    )?;

    if !up.is_empty() {
        writeln!(&mut f, "{}<br />", up)?;
    }

    let mut refline = String::new();
    let mut refline_len = 0;

    for i in data.oidxrefs.iter().rev() {
        let il = i.name.len();
        if (refline_len + il + 3) > OIDXREFS_LINE_MAXLEN {
            writeln!(&mut f, "{}<br />", refline)?;
            refline.clear();
            refline_len = 0;
        }
        if !refline.is_empty() {
            refline += " - ";
            refline_len += 3;
        }
        use std::fmt::Write;
        write!(
            &mut refline,
            "<a href=\"{}{}.html\">{}</a>",
            i.name.replace('&', "&amp;"),
            if i.typ == IndexTyp::Directory {
                "/index"
            } else {
                ""
            },
            i.name
        )
        .unwrap();
        refline_len += il;
    }
    if !refline.is_empty() {
        writeln!(&mut f, "{}<br />", refline)?;
        std::mem::drop(refline);
    }

    for i in data.ents.iter().rev() {
        write!(
            &mut f,
            "{}: <a href=\"{}\">{}</a>",
            i.cdate.format("%d.%m.%Y"),
            i.href,
            i.title
        )?;
        if !i.author.is_empty() {
            write!(&mut f, " <span class=\"authorspec\">by {}</span>", i.author)?;
        }
        writeln!(&mut f, "<br />")?;
    }

    writeln!(&mut f, "</tt>\n  </body>\n</html>")?;

    f.flush()?;
    f.into_inner()?.sync_all()?;
    Ok(())
}

pub fn write_feed(config: &Config, outdir: &Path, data: &Index) -> std::io::Result<()> {
    use atom_syndication::{Entry, Link, Person};
    use chrono::{DateTime, TimeZone, Utc};

    assert_eq!(data.typ, IndexTyp::Directory);
    println!("- atom feed");

    let now: DateTime<Utc> = Utc::now();
    let nult = chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap();

    let feed = atom_syndication::Feed {
        authors: vec![{
            let mut p = Person::default();
            p.set_name(&config.author);
            p
        }],
        links: vec![
            {
                Link {
                    href: config.id.as_str().to_string(),
                    rel: "alternate".to_string(),
                    ..Default::default()
                }
            },
            {
                Link {
                    href: format!("{}/feed.atom", config.id),
                    rel: "self".to_string(),
                    ..Default::default()
                }
            },
        ],
        title: Text {
            value: config.blog_name.clone(),
            base: None,
            lang: None,
            r#type: guess_text_type(&config.blog_name),
        },
        id: config.id.to_string(),
        entries: data
            .ents
            .iter()
            .rev()
            .take(20)
            .map(|i| {
                let (url, updts) = if i.href.starts_with('/') || i.href.contains("://") {
                    // absolute link, use cdate as update timestamp
                    (
                        i.href.clone(),
                        TimeZone::from_utc_datetime(&Utc, &i.cdate.and_time(nult)),
                    )
                } else {
                    // relative link, use mtime, or use cdate as fallback
                    (
                        format!("{}/{}", config.id, i.href),
                        match std::fs::metadata(outdir.join(&i.href)) {
                            Ok(x) => crate::utils::system_time_to_date_time(x.modified().unwrap()),
                            Err(e) => {
                                eprintln!(
                                    "  warning: unable to get mtime of: {}, error = {}",
                                    i.href, e
                                );
                                TimeZone::from_utc_datetime(&Utc, &i.cdate.and_time(nult))
                            }
                        },
                    )
                };
                Entry {
                    title: Text {
                        value: i.title.clone(),
                        base: None,
                        lang: None,
                        r#type: guess_text_type(&i.title),
                    },
                    id: url,
                    links: vec![{
                        Link {
                            href: i.href.clone(),
                            rel: "alternate".to_string(),
                            ..Default::default()
                        }
                    }],
                    authors: i
                        .authors
                        .iter()
                        .map(|a| Person {
                            name: a.clone(),
                            email: None,
                            uri: None,
                        })
                        .collect(),
                    updated: updts.into(),
                    ..Default::default()
                }
            })
            .collect(),
        updated: now.into(),
        ..Default::default()
    };

    let fpath = outdir.join("feed.atom");
    let f = std::io::BufWriter::new(std::fs::File::create(fpath)?);
    let mut f = feed.write_to(f).expect("unable to serialize atom feed");
    f.flush()?;
    f.into_inner()?.sync_all()?;

    Ok(())
}
