use alloc::{vec, vec::Vec};

pub struct HashMap<K, V> {
    buckets: Vec<Option<(K, V)>>,
    size: usize,
}

fn custom_hash<T: AsRef<str>>(input: T) -> u64 {
    let mut hash: u64 = 0;

    for byte in input.as_ref().bytes() {
        hash = (hash << 5).wrapping_add(hash) ^ u64::from(byte);
    }

    hash
}

impl<K: PartialEq + Clone + AsRef<str>, V: Clone> HashMap<K, V> {
    pub fn new() -> Self {
        const INITIAL_CAPACITY: usize = 10_2400;
        let buckets = vec![None; INITIAL_CAPACITY];
        HashMap { buckets, size: 0 }
    }

    fn hash(&self, key: &K) -> usize {
        custom_hash(key) as usize
    }

    pub fn insert(&mut self, key: K, value: V) {
        let index = self.hash(&key);

        let mut i = index;
        loop {
            i = (i + 1) % self.buckets.len();

            if self.buckets[i].is_none() {
                self.buckets[i] = Some((key, value));
                self.size += 1;
                return;
            } else if self.buckets[i].as_ref().unwrap().0 == key {
                self.buckets[i] = Some((key, value));
                return;
            }
        }
    }

    #[allow(dead_code)]
    fn get(&self, key: &K) -> Option<&V> {
        let index = self.hash(key);

        let mut i = index;
        loop {
            i = (i + 1) % self.buckets.len();

            match &self.buckets[i] {
                Some((k, v)) if k == key => return Some(v),
                None => return None,
                _ => (),
            }
        }
    }

    #[allow(dead_code)]
    fn remove(&mut self, key: &K) -> Option<V> {
        let index = self.hash(key);

        let mut i = index;
        loop {
            i = (i + 1) % self.buckets.len();

            match self.buckets[i].take() {
                Some((k, v)) if k == *key => {
                    self.size -= 1;
                    return Some(v);
                }
                None => return None,
                entry => self.buckets[i] = entry,
            }
        }
    }

    pub fn iter(&self) -> HashMapIterator<'_, K, V> {
        HashMapIterator {
            hashmap: self,
            index: 0,
        }
    }

    #[allow(dead_code)]
    fn size(&self) -> usize {
        self.size
    }
}

pub struct HashMapIterator<'a, K, V> {
    hashmap: &'a HashMap<K, V>,
    index: usize,
}

impl<'a, K, V> Iterator for HashMapIterator<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.hashmap.buckets.len() {
            if let Some((k, v)) = &self.hashmap.buckets[self.index] {
                self.index += 1;
                return Some((k, v));
            }
            self.index += 1;
        }
        None
    }
}
