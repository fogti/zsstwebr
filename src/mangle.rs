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

struct SectionState<'i> {
    do_mangle: bool,
    section: core::str::Lines<'i>,
}

pub struct MangleIter<'a, 'i> {
    ahos: &'a AhoCorasick,
    input: core::str::Split<'i, &'static str>,
    state: Option<SectionState<'i>>,
}

impl<'a, 'i> Iterator for MangleIter<'a, 'i> {
    type Item = (bool, &'i str);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.state.take() {
                None => {
                    let section = self.input.next()?;
                    let do_mangle = !self.ahos.is_match(section);
                    let section = section.lines();
                    self.state = Some(SectionState { do_mangle, section });
                    if do_mangle {
                        break Some((true, "<p>"));
                    }
                }
                Some(SectionState {
                    do_mangle,
                    mut section,
                }) => {
                    let (state, ret) = match section.next() {
                        None => (None, if do_mangle { Some("</p>") } else { None }),
                        x => (Some(SectionState { do_mangle, section }), x),
                    };
                    self.state = state;
                    if let Some(x) = ret {
                        break Some((do_mangle, x));
                    }
                }
            }
        }
    }
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
    #[inline]
    pub fn mangle_content<'a, 'i>(&'a self, input: &'i str) -> MangleIter<'a, 'i> {
        MangleIter {
            ahos: &self.ahos,
            input: input.split("\n\n"),
            state: None,
        }
    }
}
