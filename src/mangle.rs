//! blog content mangling

fn check_for_mangle_skip(dont_mangle: &[&str], mut input: &str) -> bool {
    while let Some(idx) = input.find('<') {
        input = &input[idx + 1..];
        if input.starts_with('/') {
            // see assert_eq on top of main@../build.rs
            input = &input[1..];
        }
        if dont_mangle.iter().any(|i| input.starts_with(*i)) {
            return true;
        }
    }
    false
}

pub fn mangle_content<'a>(dont_mangle: &[&str], input: &'a str) -> Vec<&'a str> {
    input
        .split("\n\n")
        .flat_map(|section| {
            if check_for_mangle_skip(dont_mangle, section) {
                vec![section, "<br />"]
            } else {
                vec!["<p>", section, "</p>"]
            }
        })
        .collect::<Vec<_>>()
}
