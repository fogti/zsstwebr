use crate::utils::back_to_idx;
use crate::{mangle::Mangler, Config, Index, IndexTyp, Post};
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
    let mut it = mangler.mangle_content(&content);
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
        } else { "" },
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
        refline += &format!(
            "<a href=\"{}{}.html\">{}</a>",
            i.name.replace('&', "&amp;"),
            if i.typ == IndexTyp::Directory {
                "/index"
            } else {
                ""
            },
            i.name
        );
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

pub fn write_feed(
    config: &Config,
    outdir: &Path,
    data: &Index,
) -> std::io::Result<()> {
    use chrono::{DateTime, Utc, SecondsFormat};

    const CDATEFMTS: &str = "%Y-%m-%dT00:00:00Z";
    let now: DateTime<Utc> = Utc::now();

    assert_eq!(data.typ, IndexTyp::Directory);
    println!("- atom feed");

    let fpath = Path::new(outdir).join("feed.atom");
    let mut f = std::io::BufWriter::new(std::fs::File::create(fpath)?);
    writeln!(&mut f, "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n<feed xmlns=\"http://www.w3.org/2005/Atom\">")?;
    writeln!(&mut f, "  <author><name>{}</name></author>", config.author)?;
    writeln!(&mut f, "  <title>{}</title>", config.blog_name)?;
    writeln!(&mut f, "  <id>{}</id>", config.id)?;
    writeln!(&mut f, "  <updated>{}</updated>", now.to_rfc3339_opts(SecondsFormat::Secs, true))?;

    for i in data.ents.iter().rev() {
        writeln!(&mut f, "  <entry>")?;
        writeln!(&mut f, "    <title type=\"xhtml\" xml:base=\"http://www.w3.org/1999/xhtml\" xmlns=\"http://www.w3.org/1999/xhtml\">{}</title>", i.title)?;
        writeln!(&mut f, "    <link href=\"{}\" />", i.href)?;
        let updts = if i.href.starts_with('/') || i.href.contains("://") {
            // absolute link, use cdate as update timestamp
            i.cdate.format(CDATEFMTS).to_string()
        } else {
            // relative link, use mtime, or use cdate as fallback
            match std::fs::metadata(&i.href) {
                Ok(x) => {
                    crate::utils::system_time_to_date_time(x.modified()?).to_rfc3339_opts(SecondsFormat::Secs, true)
                }
                Err(e) => {
                    eprintln!("  warning: unable to get mtime of: {}, error = {}", i.href, e);
                    i.cdate.format(CDATEFMTS).to_string()
                }
            }
        };
        writeln!(&mut f, "    <updated>{}</updated>", updts)?;
        for a in &i.authors {
            writeln!(&mut f, "    <author><name>{}</name></author>", a)?;
        }
        writeln!(&mut f, "  </entry>")?;
    }

    f.flush()?;
    f.into_inner()?.sync_all()?;
    Ok(())
}
