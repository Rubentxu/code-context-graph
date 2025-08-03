use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::Result;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub engine: EngineConfig,
    pub parser: ParserConfig,
    pub falkordb: FalkorDBConfig,
    pub cas: CASConfig,
    pub file_watcher: FileWatcherConfig,
    pub versioning: VersioningConfig,
    pub connascence: ConnascenceConfig,
    pub aase: AASEConfig,
    pub quality_metrics: QualityMetricsConfig,
    pub api: ApiConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub name: String,
    pub languages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParserConfig {
    pub max_file_size_kb: usize,
    pub ignore_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FalkorDBConfig {
    pub url: String,
    pub graph_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CASConfig {
    pub enabled: bool,
    pub storage_path: PathBuf,
    pub hash_algorithm: String,
    pub compression: String,
    pub dedup_threshold: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileWatcherConfig {
    pub enabled: bool,
    pub debounce_ms: u64,
    pub batch_threshold: usize,
    pub ignore_patterns: Vec<String>,
    pub recursive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersioningConfig {
    pub enabled: bool,
    pub max_versions: usize,
    pub auto_snapshot_interval: u64,
    pub merkle_tree_fanout: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnascenceConfig {
    pub enabled: bool,
    pub detect_static: bool,
    pub detect_dynamic: bool,
    pub strength_threshold: f32,
    pub auto_suggest_refactoring: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AASEConfig {
    pub enabled: bool,
    pub context_path: PathBuf,
    pub naming_convention: String,
    pub auto_propagate: bool,
    pub human_review_threshold: f32,
    pub artifact_versioning: bool,
    pub context_chain_depth: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetricsConfig {
    pub calculate_cohesion: bool,
    pub calculate_coupling: bool,
    pub maintainability_threshold: u32,
    pub complexity_warning: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub port: u16,
    pub max_context_size: usize,
    pub enable_version_api: bool,
    pub enable_quality_api: bool,
    pub enable_aase_api: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
}

impl Config {
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| crate::CodeGraphError::Config { 
                message: format!("Failed to parse config: {}", e) 
            })?;
        Ok(config)
    }
    
    pub fn default() -> Self {
        Self {
            engine: EngineConfig {
                name: "code-context-graph".to_string(),
                languages: vec!["python".to_string(), "javascript".to_string(), "java".to_string(), "kotlin".to_string()],
            },
            parser: ParserConfig {
                max_file_size_kb: 1024,
                ignore_patterns: vec!["*_test.py".to_string(), "*.min.js".to_string()],
            },
            falkordb: FalkorDBConfig {
                url: "redis://localhost:6379".to_string(),
                graph_name: "code_graph".to_string(),
            },
            cas: CASConfig {
                enabled: true,
                storage_path: PathBuf::from("./cas_store"),
                hash_algorithm: "blake3".to_string(),
                compression: "zstd".to_string(),
                dedup_threshold: 0.8,
            },
            file_watcher: FileWatcherConfig {
                enabled: true,
                debounce_ms: 100,
                batch_threshold: 50,
                ignore_patterns: vec![".git".to_string(), "node_modules".to_string(), "target".to_string()],
                recursive: true,
            },
            versioning: VersioningConfig {
                enabled: true,
                max_versions: 1000,
                auto_snapshot_interval: 3600,
                merkle_tree_fanout: 16,
            },
            connascence: ConnascenceConfig {
                enabled: true,
                detect_static: true,
                detect_dynamic: true,
                strength_threshold: 0.7,
                auto_suggest_refactoring: true,
            },
            aase: AASEConfig {
                enabled: true,
                context_path: PathBuf::from("./context"),
                naming_convention: "strict".to_string(),
                auto_propagate: true,
                human_review_threshold: 0.8,
                artifact_versioning: true,
                context_chain_depth: 5,
            },
            quality_metrics: QualityMetricsConfig {
                calculate_cohesion: true,
                calculate_coupling: true,
                maintainability_threshold: 65,
                complexity_warning: 10,
            },
            api: ApiConfig {
                port: 8080,
                max_context_size: 8192,
                enable_version_api: true,
                enable_quality_api: true,
                enable_aase_api: true,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
            },
        }
    }
}