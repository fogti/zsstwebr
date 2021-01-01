use crate::utils::back_to_idx;
use crate::{mangle::Mangler, Config, Index, IndexTyp, Post};
use std::io::{Result, Write};
use std::path::Path;

const OIDXREFS_LINE_MAXLEN: usize = 200;

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
    mut data: Index,
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
    <title>{}{}{}{}</title>
{}  </head>
  <body>
    <h1>{}{}{}{}</h1>
{}
<tt>
"#,
        &config.stylesheet,
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

    data.oidxrefs.sort_unstable();

    let mut refline = String::new();

    for i in data.oidxrefs.iter().rev() {
        let cur = format!("<a href=\"{}.html\">{}</a>", i.replace('&', "&amp;"), i);
        if refline.is_empty() {
            refline = cur;
        } else if (refline.len() + cur.len() + 3) <= OIDXREFS_LINE_MAXLEN {
            refline += " - ";
            refline += &cur;
        } else {
            writeln!(&mut f, "{}<br />", refline)?;
            refline = cur;
        }
    }
    if !refline.is_empty() {
        writeln!(&mut f, "{}<br />", refline)?;
        std::mem::drop(refline);
    }

    data.ents.sort_unstable();

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
