pub struct HashTable {
    buckets: Vec<usize>,
}

impl HashTable {
    pub fn new(block_size: usize) -> Self {
        Self {
            buckets: vec![0; block_size],
        }
    }

    /// There may be collisions! This will return a valid index, which *might*
    /// be a match. It's up to you to check whether the bytes at that index are
    /// actually a match.
    pub fn get(&self, pattern: &[u8]) -> usize {
        assert_eq!(pattern.len(), 4);
        let pattern = u32::from_le_bytes(pattern.try_into().unwrap());

        let bucket = self.hash(pattern);
        let last_seen = self.buckets[bucket];
        last_seen
    }

    pub fn insert(&mut self, pattern: &[u8], last_seen: usize) {
        assert_eq!(pattern.len(), 4);
        let pattern = u32::from_le_bytes(pattern.try_into().unwrap());

        let bucket = self.hash(pattern);
        self.buckets[bucket] = last_seen;
    }

    fn hash(&self, pattern: u32) -> usize {
        let bucket: u32 = pattern.wrapping_mul(0x1e35_a7bd);
        usize::try_from(bucket).unwrap() % self.buckets.len()
    }
}
