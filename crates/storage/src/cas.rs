use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

#[derive(Debug, Clone)]
pub struct CasConfig {
    pub root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct CasStore {
    root: PathBuf,
}

impl CasStore {
    pub fn new(config: CasConfig) -> Result<Self> {
        if config.root.as_os_str().is_empty() {
            return Err(anyhow!("CAS root path cannot be empty"));
        }
        fs::create_dir_all(&config.root)
            .with_context(|| format!("creating CAS root at {}", config.root.display()))?;
        Ok(Self { root: config.root })
    }

    fn path_for_hash(&self, hash: &str) -> Result<PathBuf> {
        if hash.len() < 2 { return Err(anyhow!("invalid hash")); }
        let (p1, rest) = hash.split_at(2);
        Ok(self.root.join(p1).join(rest))
    }

    pub fn put_bytes(&self, data: &[u8]) -> Result<String> {
        let hash = blake3::hash(data).to_hex().to_string();
        let path = self.path_for_hash(&hash)?;
        if path.exists() {
            // already present; assume identical by content hash
            return Ok(hash);
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("creating CAS bucket {}", parent.display()))?;
        }
        // write atomically: write to temp then rename
        let tmp_path = path.with_extension(".tmp");
        {
            let mut f = fs::File::create(&tmp_path)
                .with_context(|| format!("creating temp file {}", tmp_path.display()))?;
            f.write_all(data).with_context(|| "writing data to temp file")?;
            f.sync_all().ok();
        }
        fs::rename(&tmp_path, &path).with_context(|| format!("committing CAS object {}", path.display()))?;
        Ok(hash)
    }

    pub fn get(&self, hash: &str) -> Result<Option<Vec<u8>>> {
        let path = self.path_for_hash(hash)?;
        if !path.exists() { return Ok(None); }
        let bytes = fs::read(&path).with_context(|| format!("reading CAS object {}", path.display()))?;
        Ok(Some(bytes))
    }

    pub fn has(&self, hash: &str) -> Result<bool> {
        let path = self.path_for_hash(hash)?;
        Ok(path.exists())
    }
}
