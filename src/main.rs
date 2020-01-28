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
        match &rd.data {
            base::PostData::Link(ref l) => {
                ents.push(format!(
                    "{}: <a href=\"{}\">{}</a><br />",
                    rd.cdate.format("%d.%m.%Y"),
                    l,
                    &rd.title
                ));
            }
            base::PostData::Text(ref t) => {
                ents.push(format!(
                    "{}: <a href=\"{}\">{}</a><br />",
                    rd.cdate.format("%d.%m.%Y"),
                    fpath,
                    &rd.title
                ));
                writeln!(
                    &mut wr,
                    "<!doctype html>\n<html lang=\"de\" dir=\"ltr\">\n  <head>"
                ).unwrap();
                writeln!(&mut wr, "    <meta charset=\"utf-8\" />").unwrap();
                writeln!(&mut wr, "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />").unwrap();
                writeln!(
                    &mut wr,
                    "    <link rel=\"stylesheet\" href=\"{}\" type=\"text/css\" />",
                    &config.stylesheet
                ).unwrap();
                writeln!(
                    &mut wr,
                    "    <title>{} &mdash; {}</title>",
                    &rd.title, &config.blog_name
                ).unwrap();
                write!(&mut wr, "{}{}", &config.x_head, &rd.x_head).unwrap();
                writeln!(&mut wr, "  </head>\n  <body>").unwrap();
                writeln!(&mut wr, "    <h1>{}</h1>", &rd.title).unwrap();
                write!(&mut wr, "{}", &config.x_body_ph1).unwrap();
                writeln!(&mut wr, "    <a href=\"#\" onclick=\"window.history.back()\">Zur&uuml;ck zur vorherigen Seite</a> - <a href=\"{}\">Zur&uuml;ck zur Hauptseite</a>{}<br />", base::back_to_idx(fpath), &config.x_nav).unwrap();
                writeln!(&mut wr).unwrap();
                for i in mangle::mangle_content(&dont_mangle, t).lines() {
                    writeln!(&mut wr, "    {}", i).unwrap();
                }
                writeln!(&mut wr, "  </body>\n</html>").unwrap();
            }
        }
    });

    let mut f = std::io::BufWriter::new(
        std::fs::File::create(std::path::Path::new(outdir).join("index.html"))
            .expect("unable to open index file"),
    );

    writeln!(
        &mut f,
        "<!doctype html>\n<html lang=\"de\" dir=\"ltr\">\n  <head>"
    ).unwrap();
    writeln!(&mut f, "    <meta charset=\"utf-8\" />").unwrap();
    writeln!(
        &mut f,
        "    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\" />"
    ).unwrap();
    writeln!(
        &mut f,
        "    <link rel=\"stylesheet\" href=\"{}\" type=\"text/css\" />",
        &config.stylesheet
    ).unwrap();
    writeln!(&mut f, "    <title>{}</title>", &config.blog_name).unwrap();
    write!(&mut f, "{}", &config.x_head).unwrap();
    writeln!(&mut f, "  </head>\n  <body>").unwrap();
    writeln!(&mut f, "    <h1>{}</h1>", &config.blog_name).unwrap();
    write!(&mut f, "{}", &config.x_body_ph1).unwrap();
    writeln!(&mut f, "<tt>").unwrap();

    for i in ents.iter().rev() {
        writeln!(&mut f, "{}", i).unwrap();
    }

    writeln!(&mut f, "</tt>\n  </body>\n</html>").unwrap();
}
