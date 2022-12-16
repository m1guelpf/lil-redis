use std::collections::HashMap;

use crate::utils::now;

struct Entry {
    value: String,
    ttl: Option<u64>,
    inserted_at: u128,
}

pub struct Cache {
    cache: HashMap<String, Entry>,
}

impl Cache {
    pub fn new() -> Cache {
        Cache {
            cache: HashMap::with_capacity(128),
        }
    }

    pub fn get(&mut self, key: &str) -> Option<String> {
        match self.cache.get(key) {
            Some(entry) => {
                if let Some(ttl) = entry.ttl {
                    if now() - entry.inserted_at > ttl.into() {
                        self.cache.remove(key);

                        return None;
                    }
                }

                Some(entry.value.clone())
            }
            None => None,
        }
    }

    pub fn set(&mut self, key: String, value: String, ttl: Option<u64>) {
        self.cache.insert(
            key,
            Entry {
                value,
                ttl,
                inserted_at: now(),
            },
        );
    }
}
