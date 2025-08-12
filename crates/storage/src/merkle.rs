use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct MerkleBuilder {
    fanout: usize,
    entries: BTreeMap<String, Vec<u8>>, // keep sorted by path
}

impl MerkleBuilder {
    pub fn new() -> Self { Self { fanout: 2, entries: BTreeMap::new() } }
    pub fn fanout(mut self, fanout: usize) -> Self { if fanout >= 2 { self.fanout = fanout; } self }
    pub fn add<P: Into<String>>(&mut self, path: P, bytes: &[u8]) {
        self.entries.insert(path.into(), bytes.to_vec());
    }
    pub fn build(self) -> MerkleTree {
        let mut leaves: Vec<(String, String)> = Vec::with_capacity(self.entries.len());
        for (path, bytes) in self.entries {
            let h = blake3::hash(&bytes).to_hex().to_string();
            leaves.push((path, h));
        }
        // BTreeMap ensures sorted by path; compute root by hierarchical combining
        let root = compute_root(self.fanout, &leaves.iter().map(|(_, h)| h.as_str()).collect::<Vec<_>>());
        MerkleTree { fanout: self.fanout, leaves, root }
    }
}

fn compute_root(fanout: usize, items: &[&str]) -> String {
    if items.is_empty() { return blake3::hash(&[]).to_hex().to_string(); }
    let mut level: Vec<String> = items.iter().map(|h| h.to_string()).collect();
    while level.len() > 1 {
        let mut next: Vec<String> = Vec::new();
        for chunk in level.chunks(fanout) {
            let mut hasher = blake3::Hasher::new();
            for h in chunk { hasher.update(h.as_bytes()); }
            next.push(hasher.finalize().to_hex().to_string());
        }
        level = next;
    }
    level[0].clone()
}

#[derive(Debug, Clone)]
pub struct MerkleTree {
    fanout: usize,
    leaves: Vec<(String, String)>, // path -> hash
    root: String,
}

impl MerkleTree {
    pub fn root(&self) -> String { self.root.clone() }
    pub fn diff(&self, other: &MerkleTree) -> Diff {
        let mut changed = Vec::new();
        let mut i = 0usize;
        let mut j = 0usize;
        while i < self.leaves.len() && j < other.leaves.len() {
            let (p1, h1) = &self.leaves[i];
            let (p2, h2) = &other.leaves[j];
            if p1 == p2 {
                if h1 != h2 { changed.push(p1.clone()); }
                i += 1; j += 1;
            } else if p1 < p2 {
                changed.push(p1.clone());
                i += 1;
            } else {
                changed.push(p2.clone());
                j += 1;
            }
        }
        while i < self.leaves.len() { changed.push(self.leaves[i].0.clone()); i += 1; }
        while j < other.leaves.len() { changed.push(other.leaves[j].0.clone()); j += 1; }
        Diff { changed_paths: changed }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diff {
    pub changed_paths: Vec<String>,
}
