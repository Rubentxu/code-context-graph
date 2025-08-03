use thiserror::Error;

#[derive(Error, Debug)]
pub enum CodeGraphError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Parser error: {message}")]
    Parser { message: String },
    
    #[error("Storage error: {message}")]
    Storage { message: String },
    
    #[error("Graph error: {message}")]
    Graph { message: String },
    
    #[error("Hash error: {message}")]
    Hash { message: String },
    
    #[error("Configuration error: {message}")]
    Config { message: String },
    
    #[error("Connascence analysis error: {message}")]
    Connascence { message: String },
}

pub type Result<T> = std::result::Result<T, CodeGraphError>;