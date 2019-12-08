
use serde::{Serialize, Serializer};

use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::cmp::PartialEq;

use twox_hash::RandomXxHashBuilder64;

#[derive(Default)]
pub struct Interner {
    strings: HashSet<&'static str, RandomXxHashBuilder64>
}

impl Interner {
    pub fn intern(&mut self, s: &str) -> Atom {
        if let Some(interned) = self.strings.get(s) {
            return Atom(*interned);
        }

        let interned = Box::leak(s.to_string().into_boxed_str());
        self.strings.insert(interned);

        Atom(interned)
    }
}

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq)]
pub struct Atom(&'static str);

impl Serialize for Atom {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(ser)
    }
}

impl Hash for Atom {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        hasher.write_usize(self.0.as_ptr() as usize);
    }
}

impl PartialEq for Atom {
    fn eq(&self, atom: &Atom) -> bool {
        self.0.as_ptr() == atom.0.as_ptr()
    }
}
