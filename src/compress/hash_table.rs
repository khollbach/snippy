pub struct HashTable {
    buckets: Vec<u16>,
}

#[derive(Debug, Clone, Copy)]
pub struct Hash {
    bucket_index: usize,
}

impl HashTable {
    // TODO: choose # buckets using same heuristics as reference impl
    pub fn new(_block_size: usize) -> Self {
        Self {
            buckets: vec![0; 16 * 1024],
        }
    }

    /// This is a separate method, so you can avoid calling it twice if you know
    /// you're going to `get` and then `insert` using the same key.
    pub fn hash(&self, pattern: &[u8]) -> Hash {
        assert_eq!(pattern.len(), 4);
        let pattern = u32::from_le_bytes(pattern.try_into().unwrap());

        let hash: u32 = pattern.wrapping_mul(0x1e35_a7bd);
        Hash {
            bucket_index: usize::try_from(hash).unwrap() % self.buckets.len(),
        }
    }

    /// There may be collisions! This will return a valid index, which *might*
    /// be a match. It's up to you to check whether the bytes at that index are
    /// actually a match.
    pub fn get(&self, hash: Hash) -> usize {
        let last_seen: u16 = self.buckets[hash.bucket_index];
        last_seen.into()
    }

    pub fn insert(&mut self, hash: Hash, last_seen: usize) {
        let last_seen = u16::try_from(last_seen).unwrap();
        self.buckets[hash.bucket_index] = last_seen;
    }
}
