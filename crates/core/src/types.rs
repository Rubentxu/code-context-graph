use crate::Hash;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    Python,
    Java,
    Kotlin,
    JavaScript,
    TypeScript,
    Unknown,
}

impl Language {
    pub fn from_extension(ext: &str) -> Self {
        match ext {
            "py" => Self::Python,
            "java" => Self::Java,
            "kt" => Self::Kotlin,
            "js" => Self::JavaScript,
            "ts" => Self::TypeScript,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    File,
    Module,
    Class,
    Interface,
    Function,
    Method,
    Variable,
    Type,
    Enum,
    ConnascenceNode,
    ContextArtifact,
    QualityMetric,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationType {
    Contains,
    Imports,
    Extends,
    Implements,
    Calls,
    References,
    Returns,
    Parameter,
    Instantiates,
    Uses,
    HasConnascence(ConnascenceType),
    ConsumesContext,
    ProducesContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConnascenceType {
    Name,
    Type,
    Meaning,
    Position,
    Algorithm,
    Execution,
    Timing,
    Values,
    Identity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeNode {
    pub id: Hash,
    pub node_type: NodeType,
    pub name: String,
    pub language: Language,
    pub file_path: PathBuf,
    pub line_range: (u32, u32),
    pub metadata: HashMap<String, serde_json::Value>,
}

impl CodeNode {
    pub fn new(
        node_type: NodeType,
        name: String,
        language: Language,
        file_path: PathBuf,
        line_range: (u32, u32),
    ) -> Self {
        let content = format!("{}:{}:{}:{}-{}", 
            file_path.display(), language as u8, name, line_range.0, line_range.1);
        let id = Hash::from_string(&content);
        
        Self {
            id,
            node_type,
            name,
            language,
            file_path,
            line_range,
            metadata: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    pub id: Hash,
    pub from_node: Hash,
    pub to_node: Hash,
    pub relation_type: RelationType,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Relation {
    pub fn new(from_node: Hash, to_node: Hash, relation_type: RelationType) -> Self {
        let content = format!("{}->{}:{:?}", from_node, to_node, relation_type);
        let id = Hash::from_string(&content);
        
        Self {
            id,
            from_node,
            to_node,
            relation_type,
            metadata: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnascenceNode {
    pub id: Hash,
    pub conn_type: ConnascenceType,
    pub strength: f32,
    pub locality: f32,
    pub degree: usize,
    pub entities: Vec<Hash>,
    pub impact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub cohesion: f32,
    pub afferent_coupling: usize,
    pub efferent_coupling: usize,
    pub instability: f32,
    pub connascence_score: f32,
    pub maintainability_index: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInput {
    pub root_path: PathBuf,
    pub included_extensions: Vec<String>,
    pub exclude_patterns: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CASEntry {
    pub content_hash: Hash,
    pub content_type: ContentType,
    pub size: usize,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    SourceCode,
    AST,
    Graph,
    Metadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleNode {
    pub content_hash: Hash,
    pub children: Vec<Hash>,
    pub node_type: NodeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphVersion {
    pub version_id: Uuid,
    pub merkle_root: Hash,
    pub parent_version: Option<Hash>,
    pub timestamp: DateTime<Utc>,
    pub author: String,
    pub change_summary: ChangeSummary,
    pub quality_delta: QualityDelta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSummary {
    pub files_added: Vec<PathBuf>,
    pub files_modified: Vec<PathBuf>,
    pub files_deleted: Vec<PathBuf>,
    pub entities_added: usize,
    pub entities_modified: usize,
    pub entities_deleted: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityDelta {
    pub cohesion_change: f32,
    pub coupling_change: f32,
    pub maintainability_change: f32,
    pub connascence_changes: Vec<ConnascenceChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnascenceChange {
    pub conn_type: ConnascenceType,
    pub strength_delta: f32,
    pub entities: Vec<Hash>,
    pub change_type: ChangeType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Modified,
    Removed,
    Strengthened,
    Weakened,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextArtifact {
    pub id: String,
    pub artifact_type: ArtifactType,
    pub domain: String,
    pub content: String,
    pub version: u32,
    pub dependencies: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactType {
    Context,
    Model,
    UseCase,
    Prompt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AASEChain {
    pub domain_context: String,
    pub model: String,
    pub use_case: String,
    pub prompt: String,
    pub generated_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEvent {
    pub path: PathBuf,
    pub event_type: FileEventType,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileEventType {
    Created,
    Modified,
    Deleted,
    Renamed { from: PathBuf },
    BatchStart,
    BatchEnd,
}