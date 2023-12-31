use std::collections::{hash_map::Iter, HashMap};

use super::pval::{PValue, SlotNumber};

#[derive(Debug)]
pub struct Accepted {
    m: HashMap<SlotNumber, Box<PValue>>,
}

impl Accepted {
    pub fn insert(&mut self, k: SlotNumber, v: PValue) {
        match self.m.get_mut(&k) {
            Some(e) => {
                if e.ballot < v.ballot {
                    **e = v;
                }
            }
            None => {
                self.m.insert(k, Box::new(v));
            }
        }
    }

    pub fn new() -> Accepted {
        Accepted { m: HashMap::new() }
    }

    pub(crate) fn extend(&mut self, accepted: Accepted) -> () {
        for (k, v) in accepted.m {
            self.insert(k, *v);
        }
    }

    pub fn iter(&self) -> Iter<'_, u64, Box<PValue>> {
        self.m.iter()
    }

    pub fn map<T>(self, f: fn(PValue) -> T) -> HashMap<u64, T> {
        let mut res = HashMap::new();
        for (k, v) in self.m.into_iter().map(|(k, v)| (k, *v)) {
            res.insert(k, f(v));
        }
        res
    }
}

impl Clone for Accepted {
    fn clone(&self) -> Accepted {
        let mut res = HashMap::new();
        for (k, v) in self.m.iter() {
            res.insert(*k, v.clone());
        }
        return Accepted { m: res };
    }
}
