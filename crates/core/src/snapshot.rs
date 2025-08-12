use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMeta {
    pub root: String,
    pub total_files: usize,
    pub total_bytes: u64,
    pub timestamp: String,
    pub user: Option<String>,
    pub message: Option<String>,
    pub files: Vec<FileEntry>,
}

impl SnapshotMeta {
    pub fn new(root: String, total_files: usize, total_bytes: u64, message: Option<String>) -> Self {
        let timestamp = chrono::Utc::now().to_rfc3339();
        let user = std::env::var("USER").ok().or_else(|| std::env::var("USERNAME").ok());
        Self { root, total_files, total_bytes, timestamp, user, message, files: Vec::new() }
    }

    pub fn with_files(root: String, total_files: usize, total_bytes: u64, files: Vec<FileEntry>, message: Option<String>) -> Self {
        let mut s = Self::new(root, total_files, total_bytes, message);
        s.files = files;
        s
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub hash: String,
}
