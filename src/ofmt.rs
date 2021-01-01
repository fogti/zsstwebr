use crate::utils::back_to_idx;
use crate::{mangle::Mangler, Config, Post};
use std::io::{Result, Write};
use std::path::Path;

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
    f.into_inner()?.sync_all()?;
    Ok(())
}

pub fn write_tag_index(
    config: &Config,
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
    f.into_inner()?.sync_all()?;
    Ok(())
}
