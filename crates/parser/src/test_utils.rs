use crate::ast::{ASTNode, SimplifiedAST, ASTNodeType, NodeLocation};
use crate::language::registry::ParserRegistry;
use crate::visitor::base::{ASTVisitor, VisitorContext};
use code_context_graph_core::{Language, Result};
use std::collections::HashMap;
use std::path::PathBuf;
#[cfg(test)]
use proptest::prelude::Strategy;
#[cfg(test)]
use proptest::strategy::ValueTree;
use tempfile::TempDir;

/// Test utilities for parser testing
pub struct TestUtils;

impl TestUtils {
    /// Create a temporary directory with test files
    pub fn create_temp_project() -> std::io::Result<TempDir> {
        tempfile::tempdir()
    }

    /// Create a test file with given content and extension
    pub fn create_test_file(dir: &TempDir, filename: &str, content: &str) -> std::io::Result<PathBuf> {
        let file_path = dir.path().join(filename);
        std::fs::write(&file_path, content)?;
        Ok(file_path)
    }

    /// Parse a source string with given language
    pub fn parse_source(source: &str, language: Language) -> Result<SimplifiedAST> {
        let registry = ParserRegistry::new();
        registry.parse(source, language)
    }

    /// Create a visitor context for testing
    pub fn create_test_context(language: Language, source: &str, file_path: &str) -> VisitorContext {
        VisitorContext::new(language, source.to_string(), PathBuf::from(file_path))
    }

    /// Assert that AST contains expected node types
    pub fn assert_contains_node_types(ast: &SimplifiedAST, expected_types: &[ASTNodeType]) {
        let found_types = Self::collect_node_types(&ast.root);
        for expected_type in expected_types {
            assert!(
                found_types.contains(expected_type),
                "Expected to find node type {:?} in AST. Found types: {:?}",
                expected_type,
                found_types
            );
        }
    }

    /// Assert that AST contains expected number of nodes of a specific type
    pub fn assert_node_count(ast: &SimplifiedAST, node_type: &ASTNodeType, expected_count: usize) {
        let count = Self::count_node_type(&ast.root, node_type);
        assert_eq!(
            count, expected_count,
            "Expected {} nodes of type {:?}, found {}",
            expected_count, node_type, count
        );
    }

    /// Assert that AST contains node with specific name
    pub fn assert_contains_named_node(ast: &SimplifiedAST, name: &str, node_type: &ASTNodeType) {
        let found = Self::find_named_node(&ast.root, name, node_type);
        assert!(
            found,
            "Expected to find node named '{}' of type {:?}",
            name, node_type
        );
    }

    /// Assert that metadata contains expected key-value pair
    pub fn assert_metadata<T>(node: &ASTNode, key: &str, expected_value: T)
    where
        T: serde::Serialize + for<'de> serde::Deserialize<'de> + std::fmt::Debug + PartialEq,
    {
        let actual_value: Option<T> = node.get_metadata(key);
        assert_eq!(
            actual_value.as_ref(),
            Some(&expected_value),
            "Expected metadata key '{}' to have value {:?}, found {:?}",
            key,
            expected_value,
            actual_value
        );
    }

    /// Assert that parsing succeeds without errors
    pub fn assert_parsing_succeeds(source: &str, language: Language) {
        let result = Self::parse_source(source, language);
        assert!(
            result.is_ok(),
            "Expected parsing to succeed for {} code: {}",
            format!("{:?}", language),
            result.err().unwrap()
        );
    }

    /// Assert that parsing handles malformed code gracefully
    pub fn assert_parsing_handles_errors(source: &str, language: Language) {
        let result = Self::parse_source(source, language);
        // Even with malformed code, we should get some AST structure
        // The parser should be resilient
        match result {
            Ok(ast) => {
                // If parsing succeeds, ensure we have at least a program node
                assert_eq!(ast.root.node_type, ASTNodeType::Program);
            }
            Err(_) => {
                // If parsing fails, that's also acceptable for malformed code
                // The important thing is that it doesn't panic
            }
        }
    }

    /// Generate test data for property-based testing
    #[cfg(test)]
    pub fn generate_valid_identifier() -> String {
        use proptest::string::string_regex;
        // Generate valid identifiers for most languages
        string_regex(r"[a-zA-Z_][a-zA-Z0-9_]*").unwrap().new_tree(&mut proptest::test_runner::TestRunner::default()).unwrap().current()
    }

    // Helper methods

    fn collect_node_types(node: &ASTNode) -> Vec<ASTNodeType> {
        let mut types = vec![node.node_type.clone()];
        for child in &node.children {
            types.extend(Self::collect_node_types(child));
        }
        types
    }

    fn count_node_type(node: &ASTNode, target_type: &ASTNodeType) -> usize {
        let mut count = if &node.node_type == target_type { 1 } else { 0 };
        for child in &node.children {
            count += Self::count_node_type(child, target_type);
        }
        count
    }

    fn find_named_node(node: &ASTNode, name: &str, node_type: &ASTNodeType) -> bool {
        if &node.node_type == node_type && node.name.as_deref() == Some(name) {
            return true;
        }
        for child in &node.children {
            if Self::find_named_node(child, name, node_type) {
                return true;
            }
        }
        false
    }
}

/// Common test assertions with better error messages
pub mod assertions {
    use super::*;
    #[cfg(test)]
    use pretty_assertions::{assert_eq, assert_ne};

    /// Assert that two ASTs are structurally similar
    #[cfg(test)]
    pub fn assert_ast_similarity(actual: &SimplifiedAST, expected: &SimplifiedAST, tolerance: f32) {
        let actual_types = TestUtils::collect_node_types(&actual.root);
        let expected_types = TestUtils::collect_node_types(&expected.root);
        
        let similarity = calculate_similarity(&actual_types, &expected_types);
        assert!(
            similarity >= tolerance,
            "AST similarity {} is below tolerance {}. Actual: {:?}, Expected: {:?}",
            similarity,
            tolerance,
            actual_types,
            expected_types
        );
    }

    /// Assert that node location is valid
    #[cfg(test)]
    pub fn assert_valid_location(location: &NodeLocation, source: &str) {
        let source_lines = source.lines().count() as u32;
        let source_len = source.len() as u32;

        assert!(
            location.start_line >= 1 && location.start_line <= source_lines,
            "Start line {} is out of bounds for {} lines",
            location.start_line,
            source_lines
        );

        assert!(
            location.end_line >= location.start_line,
            "End line {} cannot be before start line {}",
            location.end_line,
            location.start_line
        );

        assert!(
            location.start_byte < source_len && location.end_byte <= source_len,
            "Byte positions ({}, {}) are out of bounds for {} bytes",
            location.start_byte,
            location.end_byte,
            source_len
        );
    }

    #[cfg(test)]
    fn calculate_similarity(actual: &[ASTNodeType], expected: &[ASTNodeType]) -> f32 {
        let actual_set: std::collections::HashSet<_> = actual.iter().collect();
        let expected_set: std::collections::HashSet<_> = expected.iter().collect();
        
        let intersection = actual_set.intersection(&expected_set).count();
        let union = actual_set.union(&expected_set).count();
        
        if union == 0 {
            1.0
        } else {
            intersection as f32 / union as f32
        }
    }
}

/// Performance testing utilities
pub mod perf_utils {
    use super::*;
    use std::time::Instant;

    pub struct ParseMetrics {
        pub parse_time_ms: u64,
        pub ast_node_count: usize,
        pub memory_used_kb: usize,
    }

    #[cfg(test)]
    pub fn measure_parsing_performance(source: &str, language: Language) -> Result<ParseMetrics> {
        let start = Instant::now();
        let ast = TestUtils::parse_source(source, language)?;
        let parse_time = start.elapsed();

        let node_count = count_all_nodes(&ast.root);
        
        Ok(ParseMetrics {
            parse_time_ms: parse_time.as_millis() as u64,
            ast_node_count: node_count,
            memory_used_kb: estimate_memory_usage(&ast),
        })
    }

    #[cfg(test)]
    fn count_all_nodes(node: &ASTNode) -> usize {
        1 + node.children.iter().map(count_all_nodes).sum::<usize>()
    }

    #[cfg(test)]
    fn estimate_memory_usage(ast: &SimplifiedAST) -> usize {
        // Rough estimation - in a real implementation you'd use a proper profiler
        std::mem::size_of_val(ast) + estimate_node_memory(&ast.root)
    }

    #[cfg(test)]
    fn estimate_node_memory(node: &ASTNode) -> usize {
        let base_size = std::mem::size_of_val(node);
        let children_size: usize = node.children.iter().map(estimate_node_memory).sum();
        let metadata_size = node.metadata.capacity() * std::mem::size_of::<String>();
        
        base_size + children_size + metadata_size
    }
}

/// Mock implementations for testing
pub mod mocks {
    use super::*;
    use crate::visitor::entity_extractor::{EntityExtractor, EntityInfo, EntityType};
    use crate::visitor::relation_extractor::{RelationExtractor, RelationInfo, RelationType};

    /// Create a simple test AST for testing visitors
    pub fn create_simple_test_ast() -> SimplifiedAST {
        let location = NodeLocation::new(1, 0, 10, 0, 0, 100);
        let mut root = ASTNode::new(ASTNodeType::Program, None, location.clone());
        
        // Add a class
        let mut class_node = ASTNode::new(
            ASTNodeType::ClassDeclaration,
            Some("TestClass".to_string()),
            location.clone(),
        );
        class_node.add_metadata("visibility", "public");
        
        // Add a method to the class
        let method_node = ASTNode::new(
            ASTNodeType::MethodDeclaration,
            Some("testMethod".to_string()),
            location.clone(),
        );
        
        class_node.add_child(method_node);
        root.add_child(class_node);
        
        // Add a function
        let function_node = ASTNode::new(
            ASTNodeType::FunctionDeclaration,
            Some("testFunction".to_string()),
            location.clone(),
        );
        root.add_child(function_node);
        
        SimplifiedAST::new(root, Language::Java, "test")
    }

    /// Create a visitor context with test data
    pub fn create_test_visitor_context() -> VisitorContext {
        TestUtils::create_test_context(
            Language::Java,
            "class TestClass { void testMethod() {} }",
            "test.java",
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utils_create_temp_project() {
        let temp_dir = TestUtils::create_temp_project().unwrap();
        assert!(temp_dir.path().exists());
    }

    #[test]
    fn test_utils_create_test_file() {
        let temp_dir = TestUtils::create_temp_project().unwrap();
        let file_path = TestUtils::create_test_file(&temp_dir, "test.java", "class Test {}").unwrap();
        assert!(file_path.exists());
        let content = std::fs::read_to_string(&file_path).unwrap();
        assert_eq!(content, "class Test {}");
    }

    #[test]
    fn test_parse_simple_java() {
        let source = "class Test { void method() {} }";
        TestUtils::assert_parsing_succeeds(source, Language::Java);
    }

    #[test]
    fn test_node_counting() {
        let ast = mocks::create_simple_test_ast();
        TestUtils::assert_node_count(&ast, &ASTNodeType::ClassDeclaration, 1);
        TestUtils::assert_node_count(&ast, &ASTNodeType::MethodDeclaration, 1);
        TestUtils::assert_node_count(&ast, &ASTNodeType::FunctionDeclaration, 1);
    }

    #[test]
    fn test_named_node_finding() {
        let ast = mocks::create_simple_test_ast();
        TestUtils::assert_contains_named_node(&ast, "TestClass", &ASTNodeType::ClassDeclaration);
        TestUtils::assert_contains_named_node(&ast, "testMethod", &ASTNodeType::MethodDeclaration);
        TestUtils::assert_contains_named_node(&ast, "testFunction", &ASTNodeType::FunctionDeclaration);
    }
}