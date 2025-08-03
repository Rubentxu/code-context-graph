# Storage System: CAS and Merkle Trees

## Overview

Code Context Graph uses a sophisticated storage system combining Content-Addressable Storage (CAS) with Merkle Trees to provide efficient deduplication, versioning, and integrity verification. This system enables fast incremental updates and complete history tracking with minimal storage overhead.

## Content-Addressable Storage (CAS)

### Concept

Content-Addressable Storage stores data based on its content hash rather than location. Each piece of content is identified by a cryptographic hash of its data, providing automatic deduplication and integrity verification.

### Hash Function

We use Blake3 as the primary hash function due to its excellent performance and security properties:

```rust
use blake3::Hasher;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash(pub [u8; 32]);

impl ContentHash {
    pub fn new(content: &[u8]) -> Self {
        let hash = blake3::hash(content);
        ContentHash(hash.into())
    }
    
    pub fn from_reader<R: std::io::Read>(mut reader: R) -> Result<Self, std::io::Error> {
        let mut hasher = Hasher::new();
        std::io::copy(&mut reader, &mut hasher)?;
        Ok(ContentHash(hasher.finalize().into()))
    }
    
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
    
    pub fn from_hex(hex_str: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(hex_str)?;
        if bytes.len() != 32 {
            return Err(hex::FromHexError::InvalidStringLength);
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        Ok(ContentHash(array))
    }
}

impl std::fmt::Display for ContentHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}
```

### CAS Implementation

```rust
use sled::{Db, Tree};
use std::path::Path;
use async_trait::async_trait;

#[async_trait]
pub trait ContentStore: Send + Sync {
    async fn store(&self, content: &[u8]) -> Result<ContentHash, StorageError>;
    async fn retrieve(&self, hash: &ContentHash) -> Result<Vec<u8>, StorageError>;
    async fn exists(&self, hash: &ContentHash) -> Result<bool, StorageError>;
    async fn delete(&self, hash: &ContentHash) -> Result<bool, StorageError>;
    async fn size(&self, hash: &ContentHash) -> Result<u64, StorageError>;
}

pub struct SledCAS {
    db: Db,
    content_tree: Tree,
    metadata_tree: Tree,
    stats: Arc<Mutex<CASStats>>,
    config: CASConfig,
}

impl SledCAS {
    pub fn new<P: AsRef<Path>>(path: P, config: CASConfig) -> Result<Self, StorageError> {
        let db = sled::open(path)?;
        let content_tree = db.open_tree("content")?;
        let metadata_tree = db.open_tree("metadata")?;
        
        Ok(Self {
            db,
            content_tree,
            metadata_tree,
            stats: Arc::new(Mutex::new(CASStats::default())),
            config,
        })
    }
}

#[async_trait]
impl ContentStore for SledCAS {
    async fn store(&self, content: &[u8]) -> Result<ContentHash, StorageError> {
        let hash = ContentHash::new(content);
        let hash_key = hash.to_hex();
        
        // Check if content already exists (deduplication)
        if self.content_tree.contains_key(&hash_key)? {
            // Update access time and increment reference count
            self.update_metadata(&hash, MetadataUpdate::Access).await?;
            return Ok(hash);
        }
        
        // Compress content if enabled
        let stored_content = if self.config.compression_enabled {
            self.compress_content(content)?
        } else {
            content.to_vec()
        };
        
        // Store content
        self.content_tree.insert(&hash_key, stored_content)?;
        
        // Store metadata
        let metadata = ContentMetadata {
            hash: hash.clone(),
            original_size: content.len() as u64,
            compressed_size: stored_content.len() as u64,
            created_at: chrono::Utc::now(),
            accessed_at: chrono::Utc::now(),
            reference_count: 1,
            compression_type: if self.config.compression_enabled {
                CompressionType::Zstd
            } else {
                CompressionType::None
            },
        };
        
        self.metadata_tree.insert(
            &hash_key,
            bincode::serialize(&metadata)?
        )?;
        
        // Update statistics
        self.update_stats(StatsUpdate::Store {
            original_size: content.len() as u64,
            compressed_size: stored_content.len() as u64,
        }).await;
        
        self.db.flush_async().await?;
        
        Ok(hash)
    }
    
    async fn retrieve(&self, hash: &ContentHash) -> Result<Vec<u8>, StorageError> {
        let hash_key = hash.to_hex();
        
        // Get content
        let compressed_content = self.content_tree.get(&hash_key)?
            .ok_or(StorageError::ContentNotFound(hash.clone()))?;
        
        // Get metadata for decompression info
        let metadata_bytes = self.metadata_tree.get(&hash_key)?
            .ok_or(StorageError::MetadataNotFound(hash.clone()))?;
        let metadata: ContentMetadata = bincode::deserialize(&metadata_bytes)?;
        
        // Decompress if needed
        let content = match metadata.compression_type {
            CompressionType::None => compressed_content.to_vec(),
            CompressionType::Zstd => self.decompress_content(&compressed_content)?,
        };
        
        // Update access time
        self.update_metadata(hash, MetadataUpdate::Access).await?;
        
        // Update statistics
        self.update_stats(StatsUpdate::Retrieve {
            size: content.len() as u64,
        }).await;
        
        Ok(content)
    }
    
    async fn exists(&self, hash: &ContentHash) -> Result<bool, StorageError> {
        let hash_key = hash.to_hex();
        Ok(self.content_tree.contains_key(&hash_key)?)
    }
    
    async fn delete(&self, hash: &ContentHash) -> Result<bool, StorageError> {
        let hash_key = hash.to_hex();
        
        // Check reference count
        if let Some(metadata_bytes) = self.metadata_tree.get(&hash_key)? {
            let mut metadata: ContentMetadata = bincode::deserialize(&metadata_bytes)?;
            
            if metadata.reference_count > 1 {
                // Decrement reference count instead of deleting
                metadata.reference_count -= 1;
                self.metadata_tree.insert(
                    &hash_key,
                    bincode::serialize(&metadata)?
                )?;
                return Ok(false);
            }
        }
        
        // Delete content and metadata
        let content_existed = self.content_tree.remove(&hash_key)?.is_some();
        let metadata_existed = self.metadata_tree.remove(&hash_key)?.is_some();
        
        if content_existed {
            self.update_stats(StatsUpdate::Delete).await;
        }
        
        Ok(content_existed && metadata_existed)
    }
    
    async fn size(&self, hash: &ContentHash) -> Result<u64, StorageError> {
        let hash_key = hash.to_hex();
        let metadata_bytes = self.metadata_tree.get(&hash_key)?
            .ok_or(StorageError::MetadataNotFound(hash.clone()))?;
        let metadata: ContentMetadata = bincode::deserialize(&metadata_bytes)?;
        Ok(metadata.original_size)
    }
}

impl SledCAS {
    fn compress_content(&self, content: &[u8]) -> Result<Vec<u8>, StorageError> {
        match self.config.compression_type {
            CompressionType::Zstd => {
                zstd::bulk::compress(content, self.config.compression_level)
                    .map_err(StorageError::CompressionError)
            }
            CompressionType::None => Ok(content.to_vec()),
        }
    }
    
    fn decompress_content(&self, compressed: &[u8]) -> Result<Vec<u8>, StorageError> {
        zstd::bulk::decompress(compressed, self.config.max_decompressed_size)
            .map_err(StorageError::DecompressionError)
    }
    
    async fn update_metadata(&self, hash: &ContentHash, update: MetadataUpdate) -> Result<(), StorageError> {
        let hash_key = hash.to_hex();
        
        if let Some(metadata_bytes) = self.metadata_tree.get(&hash_key)? {
            let mut metadata: ContentMetadata = bincode::deserialize(&metadata_bytes)?;
            
            match update {
                MetadataUpdate::Access => {
                    metadata.accessed_at = chrono::Utc::now();
                }
                MetadataUpdate::IncrementRef => {
                    metadata.reference_count += 1;
                }
                MetadataUpdate::DecrementRef => {
                    metadata.reference_count = metadata.reference_count.saturating_sub(1);
                }
            }
            
            self.metadata_tree.insert(
                &hash_key,
                bincode::serialize(&metadata)?
            )?;
        }
        
        Ok(())
    }
}
```

## Merkle Trees

### Concept

Merkle Trees are binary trees where each leaf node contains the hash of a data block, and each non-leaf node contains the hash of its children. This structure allows efficient verification of data integrity and fast detection of changes.

### Tree Structure

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleNode {
    pub hash: ContentHash,
    pub node_type: MerkleNodeType,
    pub children: Vec<ContentHash>,
    pub metadata: NodeMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MerkleNodeType {
    Leaf {
        content_hash: ContentHash,
        file_path: PathBuf,
        size: u64,
        modified_time: chrono::DateTime<chrono::Utc>,
    },
    Branch {
        level: u32,
        child_count: usize,
    },
    Root {
        version: u64,
        timestamp: chrono::DateTime<chrono::Utc>,
        change_summary: ChangeSummary,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub path_prefix: Option<String>,
    pub file_count: u64,
    pub total_size: u64,
}

pub struct MerkleTree {
    cas: Arc<dyn ContentStore>,
    config: MerkleConfig,
    stats: Arc<Mutex<MerkleStats>>,
}

impl MerkleTree {
    pub fn new(cas: Arc<dyn ContentStore>, config: MerkleConfig) -> Self {
        Self {
            cas,
            config,
            stats: Arc::new(Mutex::new(MerkleStats::default())),
        }
    }
    
    pub async fn build_from_files<P: AsRef<Path>>(
        &self,
        root_path: P,
        file_filter: Option<Box<dyn Fn(&Path) -> bool + Send + Sync>>,
    ) -> Result<ContentHash, MerkleError> {
        let files = self.collect_files(root_path.as_ref(), file_filter)?;
        
        // Create leaf nodes for each file
        let mut leaf_nodes = Vec::new();
        for file_path in files {
            let leaf_hash = self.create_leaf_node(&file_path).await?;
            leaf_nodes.push(leaf_hash);
        }
        
        // Build tree bottom-up
        let root_hash = self.build_tree_recursive(leaf_nodes).await?;
        
        // Create root node
        self.create_root_node(root_hash).await
    }
    
    async fn create_leaf_node(&self, file_path: &Path) -> Result<ContentHash, MerkleError> {
        let content = tokio::fs::read(file_path).await?;
        let content_hash = self.cas.store(&content).await?;
        
        let metadata = tokio::fs::metadata(file_path).await?;
        let modified_time = metadata.modified()?
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();
        let modified_time = chrono::DateTime::from_timestamp(modified_time as i64, 0)
            .ok_or(MerkleError::InvalidTimestamp)?;
        
        let leaf_node = MerkleNode {
            hash: ContentHash::new(&[]), // Will be calculated
            node_type: MerkleNodeType::Leaf {
                content_hash,
                file_path: file_path.to_path_buf(),
                size: metadata.len(),
                modified_time,
            },
            children: vec![content_hash],
            metadata: NodeMetadata {
                created_at: chrono::Utc::now(),
                path_prefix: file_path.parent().map(|p| p.to_string_lossy().to_string()),
                file_count: 1,
                total_size: metadata.len(),
            },
        };
        
        let node_bytes = bincode::serialize(&leaf_node)?;
        let node_hash = self.cas.store(&node_bytes).await?;
        
        Ok(node_hash)
    }
    
    async fn build_tree_recursive(&self, node_hashes: Vec<ContentHash>) -> Result<ContentHash, MerkleError> {
        if node_hashes.is_empty() {
            return Err(MerkleError::EmptyTree);
        }
        
        if node_hashes.len() == 1 {
            return Ok(node_hashes[0].clone());
        }
        
        let mut next_level = Vec::new();
        let fanout = self.config.fanout;
        
        for chunk in node_hashes.chunks(fanout) {
            let branch_hash = self.create_branch_node(chunk).await?;
            next_level.push(branch_hash);
        }
        
        self.build_tree_recursive(next_level).await
    }
    
    async fn create_branch_node(&self, child_hashes: &[ContentHash]) -> Result<ContentHash, MerkleError> {
        // Calculate combined hash of all children
        let mut hasher = blake3::Hasher::new();
        for child_hash in child_hashes {
            hasher.update(&child_hash.0);
        }
        let combined_hash = ContentHash(hasher.finalize().into());
        
        // Calculate aggregated metadata
        let mut total_file_count = 0;
        let mut total_size = 0;
        
        for child_hash in child_hashes {
            let child_node = self.get_node(child_hash).await?;
            total_file_count += child_node.metadata.file_count;
            total_size += child_node.metadata.total_size;
        }
        
        let branch_node = MerkleNode {
            hash: combined_hash.clone(),
            node_type: MerkleNodeType::Branch {
                level: self.calculate_level(child_hashes).await?,
                child_count: child_hashes.len(),
            },
            children: child_hashes.to_vec(),
            metadata: NodeMetadata {
                created_at: chrono::Utc::now(),
                path_prefix: None,
                file_count: total_file_count,
                total_size,
            },
        };
        
        let node_bytes = bincode::serialize(&branch_node)?;
        self.cas.store(&node_bytes).await
    }
    
    async fn create_root_node(&self, tree_hash: ContentHash) -> Result<ContentHash, MerkleError> {
        let tree_node = self.get_node(&tree_hash).await?;
        
        let root_node = MerkleNode {
            hash: ContentHash::new(&[]), // Will be calculated
            node_type: MerkleNodeType::Root {
                version: chrono::Utc::now().timestamp_millis() as u64,
                timestamp: chrono::Utc::now(),
                change_summary: ChangeSummary::default(),
            },
            children: vec![tree_hash],
            metadata: tree_node.metadata.clone(),
        };
        
        let node_bytes = bincode::serialize(&root_node)?;
        self.cas.store(&node_bytes).await
    }
    
    pub async fn get_node(&self, hash: &ContentHash) -> Result<MerkleNode, MerkleError> {
        let node_bytes = self.cas.retrieve(hash).await?;
        let node: MerkleNode = bincode::deserialize(&node_bytes)?;
        Ok(node)
    }
}
```

### Efficient Diff Computation

```rust
#[derive(Debug, Clone)]
pub struct TreeDiff {
    pub added_files: Vec<PathBuf>,
    pub modified_files: Vec<PathBuf>,
    pub deleted_files: Vec<PathBuf>,
    pub unchanged_files: Vec<PathBuf>,
    pub total_changes: usize,
}

impl MerkleTree {
    pub async fn diff(
        &self,
        old_root: &ContentHash,
        new_root: &ContentHash,
    ) -> Result<TreeDiff, MerkleError> {
        if old_root == new_root {
            return Ok(TreeDiff {
                added_files: vec![],
                modified_files: vec![],
                deleted_files: vec![],
                unchanged_files: vec![],
                total_changes: 0,
            });
        }
        
        let mut diff = TreeDiff {
            added_files: vec![],
            modified_files: vec![],
            deleted_files: vec![],
            unchanged_files: vec![],
            total_changes: 0,
        };
        
        self.diff_recursive(old_root, new_root, &mut diff).await?;
        
        diff.total_changes = diff.added_files.len() + diff.modified_files.len() + diff.deleted_files.len();
        
        Ok(diff)
    }
    
    async fn diff_recursive(
        &self,
        old_hash: &ContentHash,
        new_hash: &ContentHash,
        diff: &mut TreeDiff,
    ) -> Result<(), MerkleError> {
        if old_hash == new_hash {
            // Subtrees are identical, count unchanged files
            let node = self.get_node(old_hash).await?;
            let unchanged_count = self.count_files(&node).await?;
            // Note: We'd need to collect actual file paths if needed
            return Ok(());
        }
        
        let old_node = self.get_node(old_hash).await?;
        let new_node = self.get_node(new_hash).await?;
        
        match (&old_node.node_type, &new_node.node_type) {
            (MerkleNodeType::Leaf { file_path: old_path, content_hash: old_content, .. },
             MerkleNodeType::Leaf { file_path: new_path, content_hash: new_content, .. }) => {
                if old_path == new_path {
                    if old_content != new_content {
                        diff.modified_files.push(old_path.clone());
                    } else {
                        diff.unchanged_files.push(old_path.clone());
                    }
                } else {
                    diff.deleted_files.push(old_path.clone());
                    diff.added_files.push(new_path.clone());
                }
            }
            
            (MerkleNodeType::Branch { .. }, MerkleNodeType::Branch { .. }) => {
                // Compare children
                let old_children = &old_node.children;
                let new_children = &new_node.children;
                
                self.diff_children(old_children, new_children, diff).await?;
            }
            
            _ => {
                // Different node types, treat as complete replacement
                self.collect_deleted_files(&old_node, diff).await?;
                self.collect_added_files(&new_node, diff).await?;
            }
        }
        
        Ok(())
    }
    
    async fn diff_children(
        &self,
        old_children: &[ContentHash],
        new_children: &[ContentHash],
        diff: &mut TreeDiff,
    ) -> Result<(), MerkleError> {
        // Create hash sets for efficient lookup
        let old_set: std::collections::HashSet<_> = old_children.iter().collect();
        let new_set: std::collections::HashSet<_> = new_children.iter().collect();
        
        // Find common children (unchanged subtrees)
        let common: Vec<_> = old_set.intersection(&new_set).cloned().collect();
        
        // Count unchanged files in common subtrees
        for &hash in &common {
            let node = self.get_node(hash).await?;
            let unchanged_count = self.count_files(&node).await?;
            // Add to unchanged_files if we're tracking paths
        }
        
        // Find deleted children
        for &hash in old_set.difference(&new_set) {
            let node = self.get_node(hash).await?;
            self.collect_deleted_files(&node, diff).await?;
        }
        
        // Find added children
        for &hash in new_set.difference(&old_set) {
            let node = self.get_node(hash).await?;
            self.collect_added_files(&node, diff).await?;
        }
        
        Ok(())
    }
}
```

## Versioning System

### Version Management

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub id: Uuid,
    pub root_hash: ContentHash,
    pub parent_version: Option<Uuid>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub author: String,
    pub message: String,
    pub change_summary: ChangeSummary,
    pub quality_delta: QualityDelta,
    pub metadata: VersionMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangeSummary {
    pub files_added: u32,
    pub files_modified: u32,
    pub files_deleted: u32,
    pub lines_added: u32,
    pub lines_deleted: u32,
    pub entities_added: u32,
    pub entities_modified: u32,
    pub entities_deleted: u32,
}

pub struct VersionManager {
    cas: Arc<dyn ContentStore>,
    merkle_tree: MerkleTree,
    version_store: Arc<dyn VersionStore>,
    config: VersionConfig,
}

impl VersionManager {
    pub async fn create_version(
        &self,
        root_path: &Path,
        parent_version: Option<Uuid>,
        author: String,
        message: String,
    ) -> Result<Version, VersionError> {
        
        // Build Merkle tree for current state
        let root_hash = self.merkle_tree.build_from_files(root_path, None).await?;
        
        // Calculate changes from parent
        let (change_summary, quality_delta) = if let Some(parent_id) = parent_version {
            let parent = self.version_store.get_version(parent_id).await?;
            let diff = self.merkle_tree.diff(&parent.root_hash, &root_hash).await?;
            
            let change_summary = self.calculate_change_summary(&diff).await?;
            let quality_delta = self.calculate_quality_delta(&parent.root_hash, &root_hash).await?;
            
            (change_summary, quality_delta)
        } else {
            (ChangeSummary::default(), QualityDelta::default())
        };
        
        let version = Version {
            id: Uuid::new_v4(),
            root_hash,
            parent_version,
            timestamp: chrono::Utc::now(),
            author,
            message,
            change_summary,
            quality_delta,
            metadata: VersionMetadata::default(),
        };
        
        // Store version
        self.version_store.store_version(&version).await?;
        
        Ok(version)
    }
    
    pub async fn get_version_history(&self, from_version: Uuid) -> Result<Vec<Version>, VersionError> {
        let mut history = Vec::new();
        let mut current_id = Some(from_version);
        
        while let Some(version_id) = current_id {
            let version = self.version_store.get_version(version_id).await?;
            current_id = version.parent_version;
            history.push(version);
        }
        
        history.reverse();
        Ok(history)
    }
    
    pub async fn checkout_version(&self, version_id: Uuid, target_path: &Path) -> Result<(), VersionError> {
        let version = self.version_store.get_version(version_id).await?;
        
        // Recreate file structure from Merkle tree
        self.extract_files_from_tree(&version.root_hash, target_path).await?;
        
        Ok(())
    }
    
    async fn extract_files_from_tree(&self, root_hash: &ContentHash, target_path: &Path) -> Result<(), VersionError> {
        let root_node = self.merkle_tree.get_node(root_hash).await?;
        
        match root_node.node_type {
            MerkleNodeType::Root { .. } => {
                // Extract from the actual tree node
                if let Some(tree_hash) = root_node.children.first() {
                    self.extract_files_recursive(tree_hash, target_path).await?;
                }
            }
            _ => {
                self.extract_files_recursive(root_hash, target_path).await?;
            }
        }
        
        Ok(())
    }
    
    async fn extract_files_recursive(&self, node_hash: &ContentHash, base_path: &Path) -> Result<(), VersionError> {
        let node = self.merkle_tree.get_node(node_hash).await?;
        
        match node.node_type {
            MerkleNodeType::Leaf { content_hash, file_path, .. } => {
                let content = self.cas.retrieve(&content_hash).await?;
                let full_path = base_path.join(&file_path);
                
                if let Some(parent) = full_path.parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                
                tokio::fs::write(&full_path, content).await?;
            }
            
            MerkleNodeType::Branch { .. } => {
                for child_hash in &node.children {
                    self.extract_files_recursive(child_hash, base_path).await?;
                }
            }
            
            MerkleNodeType::Root { .. } => {
                for child_hash in &node.children {
                    self.extract_files_recursive(child_hash, base_path).await?;
                }
            }
        }
        
        Ok(())
    }
}
```

## Garbage Collection

### Reference Counting and Cleanup

```rust
pub struct GarbageCollector {
    cas: Arc<dyn ContentStore>,
    version_store: Arc<dyn VersionStore>,
    config: GCConfig,
    stats: Arc<Mutex<GCStats>>,
}

impl GarbageCollector {
    pub async fn run_gc(&self) -> Result<GCResult, GCError> {
        let mut result = GCResult::default();
        
        // Mark phase: find all reachable content
        let reachable = self.mark_reachable_content().await?;
        
        // Sweep phase: delete unreachable content
        result.deleted_count = self.sweep_unreachable_content(&reachable).await?;
        
        // Compact phase: defragment storage
        if self.config.enable_compaction {
            result.compaction_savings = self.compact_storage().await?;
        }
        
        // Update statistics
        self.update_gc_stats(&result).await;
        
        Ok(result)
    }
    
    async fn mark_reachable_content(&self) -> Result<HashSet<ContentHash>, GCError> {
        let mut reachable = HashSet::new();
        
        // Get all versions within retention period
        let cutoff_time = chrono::Utc::now() - chrono::Duration::days(self.config.retention_days as i64);
        let versions = self.version_store.get_versions_since(cutoff_time).await?;
        
        // Mark all content reachable from these versions
        for version in versions {
            self.mark_tree_recursive(&version.root_hash, &mut reachable).await?;
        }
        
        Ok(reachable)
    }
    
    async fn mark_tree_recursive(&self, hash: &ContentHash, reachable: &mut HashSet<ContentHash>) -> Result<(), GCError> {
        if reachable.contains(hash) {
            return Ok(()); // Already processed
        }
        
        reachable.insert(hash.clone());
        
        // Get node and mark its children
        let node_bytes = self.cas.retrieve(hash).await?;
        let node: MerkleNode = bincode::deserialize(&node_bytes)?;
        
        for child_hash in &node.children {
            self.mark_tree_recursive(child_hash, reachable).await?;
        }
        
        Ok(())
    }
    
    async fn sweep_unreachable_content(&self, reachable: &HashSet<ContentHash>) -> Result<u64, GCError> {
        let mut deleted_count = 0;
        
        // This would require iterating over all stored content
        // Implementation depends on the specific storage backend
        // For sled, we'd iterate over all keys in the content tree
        
        // Pseudo-code:
        // for hash in all_stored_hashes {
        //     if !reachable.contains(&hash) {
        //         if can_delete(&hash) {
        //             self.cas.delete(&hash).await?;
        //             deleted_count += 1;
        //         }
        //     }
        // }
        
        Ok(deleted_count)
    }
}
```

## Performance Optimizations

### Caching Layer

```rust
use lru::LruCache;

pub struct CachedCAS {
    inner: Arc<dyn ContentStore>,
    cache: Arc<Mutex<LruCache<ContentHash, Vec<u8>>>>,
    config: CacheConfig,
    stats: Arc<Mutex<CacheStats>>,
}

impl CachedCAS {
    pub fn new(inner: Arc<dyn ContentStore>, config: CacheConfig) -> Self {
        let cache = LruCache::new(config.max_entries);
        
        Self {
            inner,
            cache: Arc::new(Mutex::new(cache)),
            config,
            stats: Arc::new(Mutex::new(CacheStats::default())),
        }
    }
}

#[async_trait]
impl ContentStore for CachedCAS {
    async fn store(&self, content: &[u8]) -> Result<ContentHash, StorageError> {
        let hash = self.inner.store(content).await?;
        
        // Cache the content if it's not too large
        if content.len() <= self.config.max_entry_size {
            let mut cache = self.cache.lock().await;
            cache.put(hash.clone(), content.to_vec());
        }
        
        Ok(hash)
    }
    
    async fn retrieve(&self, hash: &ContentHash) -> Result<Vec<u8>, StorageError> {
        // Try cache first
        {
            let mut cache = self.cache.lock().await;
            if let Some(content) = cache.get(hash) {
                self.update_cache_stats(CacheEvent::Hit).await;
                return Ok(content.clone());
            }
        }
        
        // Cache miss, retrieve from storage
        let content = self.inner.retrieve(hash).await?;
        
        // Add to cache if not too large
        if content.len() <= self.config.max_entry_size {
            let mut cache = self.cache.lock().await;
            cache.put(hash.clone(), content.clone());
        }
        
        self.update_cache_stats(CacheEvent::Miss).await;
        
        Ok(content)
    }
    
    async fn exists(&self, hash: &ContentHash) -> Result<bool, StorageError> {
        // Check cache first
        {
            let cache = self.cache.lock().await;
            if cache.contains(hash) {
                return Ok(true);
            }
        }
        
        self.inner.exists(hash).await
    }
    
    async fn delete(&self, hash: &ContentHash) -> Result<bool, StorageError> {
        // Remove from cache
        {
            let mut cache = self.cache.lock().await;
            cache.pop(hash);
        }
        
        self.inner.delete(hash).await
    }
    
    async fn size(&self, hash: &ContentHash) -> Result<u64, StorageError> {
        self.inner.size(hash).await
    }
}
```

### Parallel Processing

```rust
use rayon::prelude::*;
use tokio::sync::Semaphore;

impl MerkleTree {
    pub async fn build_parallel<P: AsRef<Path>>(
        &self,
        root_path: P,
        max_concurrency: usize,
    ) -> Result<ContentHash, MerkleError> {
        let semaphore = Arc::new(Semaphore::new(max_concurrency));
        let files = self.collect_files(root_path.as_ref(), None)?;
        
        // Process files in parallel
        let leaf_futures: Vec<_> = files
            .into_iter()
            .map(|file_path| {
                let semaphore = Arc::clone(&semaphore);
                let tree = self.clone();
                
                async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    tree.create_leaf_node(&file_path).await
                }
            })
            .collect();
        
        let leaf_results = futures::future::try_join_all(leaf_futures).await?;
        
        // Build tree structure
        self.build_tree_recursive(leaf_results).await
    }
}
```

## Configuration

### Storage Configuration

```toml
[cas]
enabled = true
storage_path = "./cas_store"
hash_algorithm = "blake3"
compression = "zstd"
compression_level = 3
dedup_threshold = 0.85
max_content_size_mb = 100
cleanup_interval_hours = 24
retention_days = 30

[cas.cache]
enabled = true
max_entries = 10000
max_entry_size_mb = 10
eviction_policy = "lru"

[versioning]
enabled = true
max_versions = 1000
auto_snapshot_interval_seconds = 3600
merkle_fanout = 16
compression_enabled = true
garbage_collection_enabled = true
gc_interval_hours = 6

[versioning.gc]
retention_days = 30
min_versions_to_keep = 10
enable_compaction = true
compaction_threshold = 0.3
parallel_gc = true
```

## Usage Examples

### Basic Usage

```rust
use code_context_graph::storage::{SledCAS, MerkleTree, VersionManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize storage
    let cas_config = CASConfig::default();
    let cas = Arc::new(SledCAS::new("./storage", cas_config)?);
    
    let merkle_config = MerkleConfig::default();
    let merkle_tree = MerkleTree::new(Arc::clone(&cas), merkle_config);
    
    // Create version manager
    let version_config = VersionConfig::default();
    let version_store = Arc::new(FileVersionStore::new("./versions")?);
    let version_manager = VersionManager::new(cas, merkle_tree, version_store, version_config);
    
    // Create initial version
    let version = version_manager.create_version(
        Path::new("./src"),
        None,
        "developer@example.com".to_string(),
        "Initial version".to_string(),
    ).await?;
    
    println!("Created version: {}", version.id);
    
    // Make some changes and create another version
    // ... modify files ...
    
    let version2 = version_manager.create_version(
        Path::new("./src"),
        Some(version.id),
        "developer@example.com".to_string(),
        "Added payment feature".to_string(),
    ).await?;
    
    // Get diff between versions
    let diff = merkle_tree.diff(&version.root_hash, &version2.root_hash).await?;
    println!("Changes: {} added, {} modified, {} deleted", 
             diff.added_files.len(), 
             diff.modified_files.len(), 
             diff.deleted_files.len());
    
    Ok(())
}
```

### Advanced Usage with Incremental Updates

```rust
pub struct IncrementalUpdater {
    version_manager: VersionManager,
    current_version: Option<Version>,
    pending_changes: Vec<FileChange>,
}

impl IncrementalUpdater {
    pub async fn update_file(&mut self, file_path: &Path) -> Result<(), UpdateError> {
        let content = tokio::fs::read(file_path).await?;
        let new_hash = self.version_manager.cas.store(&content).await?;
        
        // Find existing file in current version
        let old_hash = if let Some(version) = &self.current_version {
            self.find_file_hash(&version.root_hash, file_path).await?
        } else {
            None
        };
        
        let change = FileChange {
            path: file_path.to_path_buf(),
            old_hash,
            new_hash,
            change_type: if old_hash.is_some() {
                ChangeType::Modified
            } else {
                ChangeType::Added
            },
        };
        
        self.pending_changes.push(change);
        Ok(())
    }
    
    pub async fn commit_changes(&mut self, message: String) -> Result<Version, UpdateError> {
        if self.pending_changes.is_empty() {
            return Err(UpdateError::NoChanges);
        }
        
        // Create new Merkle tree with changes
        let new_root = self.apply_changes_to_tree().await?;
        
        // Create version
        let parent_id = self.current_version.as_ref().map(|v| v.id);
        let version = self.version_manager.create_version_from_tree(
            new_root,
            parent_id,
            "incremental-updater".to_string(),
            message,
        ).await?;
        
        self.current_version = Some(version.clone());
        self.pending_changes.clear();
        
        Ok(version)
    }
}
```

This storage system provides the foundation for efficient, versioned storage of code analysis results with automatic deduplication and fast change detection.