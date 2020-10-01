//! blog content mangling
use aho_corasick::AhoCorasick;

pub struct Mangler {
    ahos: AhoCorasick,
}

#[inline]
fn diiter<T>(a: T, b: T) -> impl Iterator<Item = T> {
    use core::iter::once;
    once(a).chain(once(b))
}

impl Mangler {
    pub fn new(dont_mangle: &[&str]) -> Mangler {
        let pats: Vec<_> = dont_mangle
            .iter()
            .flat_map(|&i| diiter("<".to_string() + i + ">", "</".to_string() + i + ">"))
            .collect();
        Mangler {
            ahos: AhoCorasick::new_auto_configured(&pats),
        }
    }

    /// You should only prepend each line with spaces if the associated $mangle boolean is 'true'.
    pub fn mangle_content<'b, 'a: 'b>(&'b self, input: &'a str) -> impl Iterator<Item = (bool, &'a str)> + 'b {
        input
            .split("\n\n")
            .flat_map(move |section| {
                let do_mangle = !self.ahos.is_match(section);
                if do_mangle {
                    vec!["<p>", section, "</p>"]
                } else {
                    vec![section]
                }
                .into_iter()
                .flat_map(|i| i.lines())
                .map(move |i| (do_mangle, i))
            })
    }
}
