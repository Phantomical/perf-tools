
use std::collections::HashSet;

#[derive(Default)]
pub struct Interner {
    strings: HashSet<&'static str>
}

impl Interner {
    pub fn intern(&mut self, s: &str) -> &'static str {
        if let Some(interned) = self.strings.get(s) {
            return *interned;
        }

        let interned = Box::leak(s.to_string().into_boxed_str());
        self.strings.insert(interned);

        interned
    }
}
