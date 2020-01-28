mod base;
mod mangle;

fn main() {
    use clap::Arg;
    use std::{fs::File, io::Write};

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

    let dont_mangle = vec!["code>", "dl>", "h2>", "h3>", "ul>", "ol>", "pre>"];

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

    base::tr_folder2(indir, &outdir, |fpath, rd: base::Post, mut wr| {
        println!("- {}", fpath);
        let lnk = match &rd.data {
            base::PostData::Link(ref l) => l,
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
                for i in mangle::mangle_content(&dont_mangle, t).lines() {
                    writeln!(&mut wr, "    {}", i).unwrap();
                }
                writeln!(&mut wr, "  </body>\n</html>").unwrap();
                fpath
            }
        };
        ents.push(format!(
            "{}: <a href=\"{}\">{}</a><br />",
            rd.cdate.format("%d.%m.%Y"),
            lnk,
            &rd.title
        ));
    });

    let mut f = std::io::BufWriter::new(
        std::fs::File::create(std::path::Path::new(outdir).join("index.html"))
            .expect("unable to open index file"),
    );

    writeln!(
        &mut f,
        r#"<!doctype html>
<html lang="de" dir="ltr">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <link rel="stylesheet" href="{}" type="text/css" />
    <title>{}</title>
{}  </head>
  <body>
    <h1>{}</h1>
{}<tt>
"#,
        &config.stylesheet,
        &config.blog_name,
        &config.x_head,
        &config.blog_name,
        &config.x_body_ph1,
    )
    .unwrap();
    for i in ents.iter().rev() {
        writeln!(&mut f, "{}", i).unwrap();
    }

    writeln!(&mut f, "</tt>\n  </body>\n</html>").unwrap();
}
