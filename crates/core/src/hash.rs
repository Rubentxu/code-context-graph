use blake3::Hasher;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash(String);

impl Hash {
    pub fn new(data: &[u8]) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(data);
        Self(hasher.finalize().to_hex().to_string())
    }
    
    pub fn from_string(s: &str) -> Self {
        Self::new(s.as_bytes())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
    
    pub fn short(&self) -> &str {
        &self.0[..8]
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for Hash {
    fn from(s: &str) -> Self {
        Self::from_string(s)
    }
}

impl From<String> for Hash {
    fn from(s: String) -> Self {
        Self::from_string(&s)
    }
}