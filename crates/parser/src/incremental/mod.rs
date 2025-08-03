use crate::ast::SimplifiedAST;
use code_context_graph_core::{Result, Language, Hash};
use std::collections::HashMap;
use tree_sitter::Tree;

#[derive(Debug, Clone)]
pub struct ParseCache {
    entries: HashMap<Hash, CacheEntry>,
}

#[derive(Debug, Clone)]
struct CacheEntry {
    source_hash: Hash,
    ast: SimplifiedAST,
    tree: Option<Tree>, // Store Tree-sitter tree for incremental parsing
    timestamp: std::time::SystemTime,
}

#[derive(Debug, Clone)]
pub struct IncrementalParser {
    cache: ParseCache,
    max_cache_size: usize,
}

impl ParseCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn get(&self, key: &Hash) -> Option<&CacheEntry> {
        self.entries.get(key)
    }

    pub fn insert(&mut self, key: Hash, entry: CacheEntry) {
        self.entries.insert(key, entry);
    }

    pub fn contains_key(&self, key: &Hash) -> bool {
        self.entries.contains_key(key)
    }

    pub fn remove(&mut self, key: &Hash) -> Option<CacheEntry> {
        self.entries.remove(key)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn cleanup_expired(&mut self, max_age: std::time::Duration) {
        let now = std::time::SystemTime::now();
        self.entries.retain(|_, entry| {
            if let Ok(age) = now.duration_since(entry.timestamp) {
                age <= max_age
            } else {
                true // Keep entries with invalid timestamps
            }
        });
    }
}

impl IncrementalParser {
    pub fn new() -> Self {
        Self {
            cache: ParseCache::new(),
            max_cache_size: 1000, // Default cache size
        }
    }

    pub fn with_cache_size(max_cache_size: usize) -> Self {
        Self {
            cache: ParseCache::new(),
            max_cache_size,
        }
    }

    pub fn parse_incremental(
        &mut self,
        source: &str,
        language: Language,
        file_path: &std::path::Path,
    ) -> Result<SimplifiedAST> {
        let file_hash = Hash::from_string(&format!("{:?}", file_path));
        let source_hash = Hash::from_string(source);

        // Check if we have a cached entry
        if let Some(cached_entry) = self.cache.get(&file_hash) {
            // If source hasn't changed, return cached AST
            if cached_entry.source_hash == source_hash {
                return Ok(cached_entry.ast.clone());
            }

            // Source has changed, try incremental parsing
            if let Some(ref old_tree) = cached_entry.tree {
                if let Ok(new_ast) = self.parse_with_old_tree(source, language, Some(old_tree)) {
                    // Update cache with new result
                    let new_entry = CacheEntry {
                        source_hash,
                        ast: new_ast.clone(),
                        tree: None, // We don't store the new tree here for simplicity
                        timestamp: std::time::SystemTime::now(),
                    };
                    self.cache.insert(file_hash, new_entry);
                    return Ok(new_ast);
                }
            }
        }

        // Fallback to full parsing
        let ast = self.parse_full(source, language)?;
        
        // Cache the result
        let entry = CacheEntry {
            source_hash,
            ast: ast.clone(),
            tree: None, // Tree would be stored from the actual parser
            timestamp: std::time::SystemTime::now(),
        };
        
        self.cache.insert(file_hash, entry);
        self.cleanup_cache();
        
        Ok(ast)
    }

    fn parse_with_old_tree(
        &self,
        source: &str,
        language: Language,
        _old_tree: Option<&Tree>,
    ) -> Result<SimplifiedAST> {
        // This would use the actual language parsers with incremental parsing
        // For now, we'll delegate to the registry
        use crate::language::registry::ParserRegistry;
        let registry = ParserRegistry::new();
        
        // Note: This is a simplified version. In a real implementation,
        // we would need to modify the parser registry to support incremental parsing
        registry.parse(source, language)
    }

    fn parse_full(&self, source: &str, language: Language) -> Result<SimplifiedAST> {
        use crate::language::registry::ParserRegistry;
        let registry = ParserRegistry::new();
        registry.parse(source, language)
    }

    fn cleanup_cache(&mut self) {
        if self.cache.len() > self.max_cache_size {
            // Simple cleanup: remove oldest entries
            // In a real implementation, we might use LRU or other strategies
            let excess_count = self.cache.len() - self.max_cache_size;
            let mut entries_to_remove = Vec::new();
            
            // Collect entries sorted by timestamp
            let mut entries: Vec<_> = self.cache.entries.iter().collect();
            entries.sort_by_key(|(_, entry)| entry.timestamp);
            
            // Remove oldest entries
            for (key, _) in entries.iter().take(excess_count) {
                entries_to_remove.push((*key).clone());
            }
            
            for key in entries_to_remove {
                self.cache.remove(&key);
            }
        }
    }

    pub fn invalidate_file(&mut self, file_path: &std::path::Path) {
        let file_hash = Hash::from_string(&format!("{:?}", file_path));
        self.cache.remove(&file_hash);
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            total_entries: self.cache.len(),
            max_capacity: self.max_cache_size,
        }
    }

    pub fn cleanup_expired_entries(&mut self, max_age: std::time::Duration) {
        self.cache.cleanup_expired(max_age);
    }
}

#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub max_capacity: usize,
}

impl Default for IncrementalParser {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ParseCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_parse_cache() {
        let mut cache = ParseCache::new();
        let key = Hash::from_string("test");
        
        assert!(!cache.contains_key(&key));
        assert_eq!(cache.len(), 0);
        
        // This would require creating a mock SimplifiedAST
        // For now, just test the cache structure
        assert!(cache.get(&key).is_none());
    }

    #[test]
    fn test_incremental_parser_creation() {
        let parser = IncrementalParser::new();
        assert_eq!(parser.max_cache_size, 1000);
        
        let parser_custom = IncrementalParser::with_cache_size(500);
        assert_eq!(parser_custom.max_cache_size, 500);
    }

    #[test]
    fn test_cache_stats() {
        let parser = IncrementalParser::new();
        let stats = parser.cache_stats();
        
        assert_eq!(stats.total_entries, 0);
        assert_eq!(stats.max_capacity, 1000);
    }

    #[test]
    fn test_cache_cleanup_expired() {
        let mut cache = ParseCache::new();
        
        // Test cleanup with no entries
        cache.cleanup_expired(Duration::from_secs(3600));
        assert_eq!(cache.len(), 0);
    }
}