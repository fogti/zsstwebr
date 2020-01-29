//! blog content mangling
use aho_corasick::AhoCorasick;

pub struct Mangler {
    ahos: AhoCorasick,
}

impl Mangler {
    pub fn new(dont_mangle: Vec<&str>) -> Mangler {
        let pats: Vec<_> = dont_mangle.into_iter().flat_map(|i| vec!["<".to_string() + i + ">", "</".to_string() + i + ">"]).collect();
        Mangler {
            ahos: AhoCorasick::new_auto_configured(&pats),
        }
    }

    pub fn mangle_content<'a>(&self, input: &'a str) -> Vec<&'a str> {
        input
            .split("\n\n")
            .flat_map(|section| {
                if self.ahos.is_match(section) {
                    vec![section]
                } else {
                    vec!["<p>", section, "</p>"]
                }
            })
            .flat_map(|i| i.lines())
            .collect()
    }
}
