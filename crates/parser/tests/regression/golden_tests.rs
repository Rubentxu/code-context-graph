use code_context_graph_parser::visitor::{
    entity_extractor::{EntityExtractor, EntityInfo},
    relation_extractor::{RelationExtractor, RelationInfo},
    metadata_collector::MetadataCollector,
    base::ASTVisitor
};
use code_context_graph_core::Language;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::PathBuf;
use pretty_assertions::assert_eq;
use code_context_graph_parser::test_utils::TestUtils;

/// Golden file tests that verify parsing output hasn't changed unexpectedly.
/// These tests capture known-good parsing results and detect regressions.

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct GoldenData {
    pub source_hash: String,
    pub ast_summary: ASTSummary,
    pub entities: Vec<EntitySummary>,
    pub relations: Vec<RelationSummary>,
    pub metrics: MetricsSummary,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct ASTSummary {
    pub language: String,
    pub root_type: String,
    pub total_nodes: usize,
    pub node_type_counts: std::collections::HashMap<String, usize>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct EntitySummary {
    pub name: String,
    pub entity_type: String,
    pub location_summary: String, // "line:col-line:col"
    pub has_visibility: bool,
    pub modifiers_count: usize,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct RelationSummary {
    pub from_entity: String,
    pub to_entity: String,
    pub relation_type: String,
    pub location_summary: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct MetricsSummary {
    pub total_lines: u32,
    pub classes_count: u32,
    pub functions_count: u32,
    pub complexity_score: f64,
    pub language_specific_keys: Vec<String>,
}

fn get_golden_file_path(test_name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/regression/golden");
    path.push(format!("{}.json", test_name));
    path
}

fn get_fixture_path(relative_path: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixtures");
    path.push(relative_path);
    path
}

fn hash_source(source: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    source.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

fn count_nodes_recursive(node: &code_context_graph_parser::ast::ASTNode) -> usize {
    1 + node.children.iter().map(count_nodes_recursive).sum::<usize>()
}

fn collect_node_type_counts(node: &code_context_graph_parser::ast::ASTNode) -> std::collections::HashMap<String, usize> {
    let mut counts = std::collections::HashMap::new();
    collect_node_type_counts_recursive(node, &mut counts);
    counts
}

fn collect_node_type_counts_recursive(
    node: &code_context_graph_parser::ast::ASTNode,
    counts: &mut std::collections::HashMap<String, usize>
) {
    let type_name = format!("{:?}", node.node_type);
    *counts.entry(type_name).or_insert(0) += 1;
    
    for child in &node.children {
        collect_node_type_counts_recursive(child, counts);
    }
}

fn create_ast_summary(ast: &code_context_graph_parser::ast::SimplifiedAST) -> ASTSummary {
    ASTSummary {
        language: format!("{:?}", ast.language),
        root_type: format!("{:?}", ast.root.node_type),
        total_nodes: count_nodes_recursive(&ast.root),
        node_type_counts: collect_node_type_counts(&ast.root),
    }
}

fn create_entity_summary(entity: &EntityInfo) -> EntitySummary {
    EntitySummary {
        name: entity.name.clone(),
        entity_type: format!("{:?}", entity.entity_type),
        location_summary: format!(
            "{}:{}-{}:{}",
            entity.location.start_line,
            entity.location.start_column,
            entity.location.end_line,
            entity.location.end_column
        ),
        has_visibility: entity.visibility.is_some(),
        modifiers_count: entity.modifiers.len(),
    }
}

fn create_relation_summary(relation: &RelationInfo) -> RelationSummary {
    RelationSummary {
        from_entity: relation.from_entity.clone(),
        to_entity: relation.to_entity.clone(),
        relation_type: format!("{:?}", relation.relation_type),
        location_summary: format!(
            "{}:{}-{}:{}",
            relation.source_location.start_line,
            relation.source_location.start_column,
            relation.source_location.end_line,
            relation.source_location.end_column
        ),
    }
}

fn create_metrics_summary(
    metrics: &code_context_graph_parser::visitor::metadata_collector::CodeMetrics
) -> MetricsSummary {
    let mut language_keys: Vec<String> = metrics.language_specific.keys().cloned().collect();
    language_keys.sort(); // Ensure deterministic ordering
    
    MetricsSummary {
        total_lines: metrics.total_lines,
        classes_count: metrics.classes_count,
        functions_count: metrics.functions_count,
        complexity_score: (metrics.complexity_score * 100.0).round() / 100.0, // Round to 2 decimals
        language_specific_keys: language_keys,
    }
}

fn generate_golden_data(source: &str, language: Language, test_name: &str) -> GoldenData {
    // Parse the source
    let ast = TestUtils::parse_source(source, language)
        .expect("Failed to parse source for golden data generation");
    
    // Extract entities
    let mut context = TestUtils::create_test_context(language, source, &format!("{}.test", test_name));
    let mut entity_extractor = EntityExtractor::new();
    let entities = entity_extractor.visit_ast(&ast, &mut context)
        .expect("Failed to extract entities");
    
    // Extract relations
    let mut context = TestUtils::create_test_context(language, source, &format!("{}.test", test_name));
    let mut relation_extractor = RelationExtractor::new();
    let relations = relation_extractor.visit_ast(&ast, &mut context)
        .expect("Failed to extract relations");
    
    // Collect metadata
    let mut context = TestUtils::create_test_context(language, source, &format!("{}.test", test_name));
    let mut metadata_collector = MetadataCollector::new();
    let (metrics, _entity_metadata) = metadata_collector.visit_ast(&ast, &mut context)
        .expect("Failed to collect metadata");
    
    // Create summaries
    let ast_summary = create_ast_summary(&ast);
    
    let mut entity_summaries: Vec<_> = entities.iter().map(create_entity_summary).collect();
    entity_summaries.sort_by(|a, b| a.name.cmp(&b.name)); // Deterministic ordering
    
    let mut relation_summaries: Vec<_> = relations.iter().map(create_relation_summary).collect();
    relation_summaries.sort_by(|a, b| {
        a.from_entity.cmp(&b.from_entity).then(a.to_entity.cmp(&b.to_entity))
    }); // Deterministic ordering
    
    let metrics_summary = create_metrics_summary(&metrics);
    
    GoldenData {
        source_hash: hash_source(source),
        ast_summary,
        entities: entity_summaries,
        relations: relation_summaries,
        metrics: metrics_summary,
    }
}

fn load_or_create_golden_data(
    source: &str,
    language: Language,
    test_name: &str,
    update_golden: bool
) -> GoldenData {
    let golden_path = get_golden_file_path(test_name);
    
    if update_golden || !golden_path.exists() {
        println!("Generating golden data for test: {}", test_name);
        let golden_data = generate_golden_data(source, language, test_name);
        
        // Ensure directory exists
        if let Some(parent) = golden_path.parent() {
            fs::create_dir_all(parent).expect("Failed to create golden directory");
        }
        
        // Write golden data
        let json = serde_json::to_string_pretty(&golden_data)
            .expect("Failed to serialize golden data");
        fs::write(&golden_path, json)
            .expect("Failed to write golden file");
        
        golden_data
    } else {
        // Load existing golden data
        let json = fs::read_to_string(&golden_path)
            .expect("Failed to read golden file");
        serde_json::from_str(&json)
            .expect("Failed to deserialize golden data")
    }
}

fn verify_against_golden(source: &str, language: Language, test_name: &str) {
    let golden_data = load_or_create_golden_data(source, language, test_name, false);
    let current_data = generate_golden_data(source, language, test_name);
    
    // Check source hash first
    assert_eq!(
        current_data.source_hash,
        golden_data.source_hash,
        "Source code has changed - this might invalidate the golden test"
    );
    
    // Compare AST structure
    assert_eq!(
        current_data.ast_summary,
        golden_data.ast_summary,
        "AST structure has changed for test: {}",
        test_name
    );
    
    // Compare entities
    assert_eq!(
        current_data.entities.len(),
        golden_data.entities.len(),
        "Number of entities has changed for test: {}",
        test_name
    );
    
    for (current, golden) in current_data.entities.iter().zip(golden_data.entities.iter()) {
        assert_eq!(
            current,
            golden,
            "Entity mismatch for test: {} - entity: {}",
            test_name,
            current.name
        );
    }
    
    // Compare relations
    assert_eq!(
        current_data.relations.len(),
        golden_data.relations.len(),
        "Number of relations has changed for test: {}",
        test_name
    );
    
    for (current, golden) in current_data.relations.iter().zip(golden_data.relations.iter()) {
        assert_eq!(
            current,
            golden,
            "Relation mismatch for test: {} - relation: {} -> {}",
            test_name,
            current.from_entity,
            current.to_entity
        );
    }
    
    // Compare metrics (allowing small floating point differences)
    assert_eq!(current_data.metrics.total_lines, golden_data.metrics.total_lines);
    assert_eq!(current_data.metrics.classes_count, golden_data.metrics.classes_count);
    assert_eq!(current_data.metrics.functions_count, golden_data.metrics.functions_count);
    
    let complexity_diff = (current_data.metrics.complexity_score - golden_data.metrics.complexity_score).abs();
    assert!(
        complexity_diff < 0.01,
        "Complexity score has changed significantly: {} vs {} (diff: {})",
        current_data.metrics.complexity_score,
        golden_data.metrics.complexity_score,
        complexity_diff
    );
    
    assert_eq!(
        current_data.metrics.language_specific_keys,
        golden_data.metrics.language_specific_keys,
        "Language-specific metadata keys have changed for test: {}",
        test_name
    );
}

// Golden file tests

#[test]
fn test_java_complex_inheritance_golden() {
    let fixture_path = get_fixture_path("java/complex_inheritance.java");
    let source = fs::read_to_string(&fixture_path)
        .expect("Failed to read Java fixture");
    
    verify_against_golden(&source, Language::Java, "java_complex_inheritance");
}

#[test]
fn test_python_decorators_golden() {
    let fixture_path = get_fixture_path("python/decorators_example.py");
    let source = fs::read_to_string(&fixture_path)
        .expect("Failed to read Python fixture");
    
    verify_against_golden(&source, Language::Python, "python_decorators");
}

#[test]
fn test_javascript_modern_es6_golden() {
    let fixture_path = get_fixture_path("javascript/modern_es6.js");
    let source = fs::read_to_string(&fixture_path)
        .expect("Failed to read JavaScript fixture");
    
    verify_against_golden(&source, Language::JavaScript, "javascript_modern_es6");
}

#[test]
fn test_kotlin_coroutines_golden() {
    let fixture_path = get_fixture_path("kotlin/coroutines_example.kt");
    let source = fs::read_to_string(&fixture_path)
        .expect("Failed to read Kotlin fixture");
    
    verify_against_golden(&source, Language::Kotlin, "kotlin_coroutines");
}

// Simple golden tests for basic patterns

#[test]
fn test_simple_java_class_golden() {
    let source = r#"
public class SimpleClass {
    private int value;
    
    public SimpleClass(int value) {
        this.value = value;
    }
    
    public int getValue() {
        return value;
    }
    
    public void setValue(int value) {
        this.value = value;
    }
}
"#;
    
    verify_against_golden(source, Language::Java, "java_simple_class");
}

#[test]
fn test_simple_python_class_golden() {
    let source = r#"
class SimpleClass:
    def __init__(self, value):
        self.value = value
    
    def get_value(self):
        return self.value
    
    def set_value(self, value):
        self.value = value
        
    @property
    def formatted_value(self):
        return f"Value: {self.value}"
"#;
    
    verify_against_golden(source, Language::Python, "python_simple_class");
}

#[test]
fn test_simple_javascript_class_golden() {
    let source = r#"
class SimpleClass {
    constructor(value) {
        this.value = value;
    }
    
    getValue() {
        return this.value;
    }
    
    setValue(value) {
        this.value = value;
    }
    
    async fetchData() {
        const response = await fetch('/api/data');
        return response.json();
    }
}
"#;
    
    verify_against_golden(source, Language::JavaScript, "javascript_simple_class");
}

#[test]
fn test_simple_kotlin_class_golden() {
    let source = r#"
data class SimpleClass(private var value: Int) {
    fun getValue(): Int = value
    
    fun setValue(newValue: Int) {
        value = newValue
    }
    
    suspend fun fetchData(): String {
        return "data"
    }
    
    companion object {
        fun create(value: Int): SimpleClass = SimpleClass(value)
    }
}
"#;
    
    verify_against_golden(source, Language::Kotlin, "kotlin_simple_class");
}

// Test for updating golden files (normally disabled)
#[test]
#[ignore] // Remove #[ignore] to update golden files
fn test_update_all_golden_files() {
    let test_cases = vec![
        ("java/complex_inheritance.java", Language::Java, "java_complex_inheritance"),
        ("python/decorators_example.py", Language::Python, "python_decorators"),
        ("javascript/modern_es6.js", Language::JavaScript, "javascript_modern_es6"),
        ("kotlin/coroutines_example.kt", Language::Kotlin, "kotlin_coroutines"),
    ];
    
    for (fixture, language, test_name) in test_cases {
        let fixture_path = get_fixture_path(fixture);
        let source = fs::read_to_string(&fixture_path)
            .expect(&format!("Failed to read fixture: {}", fixture));
        
        println!("Updating golden file for: {}", test_name);
        load_or_create_golden_data(&source, language, test_name, true);
    }
}

// Utility test to verify golden data integrity
#[test]
fn test_golden_data_serialization_roundtrip() {
    let source = "class Test { void method() {} }";
    let golden_data = generate_golden_data(source, Language::Java, "test");
    
    // Serialize and deserialize
    let json = serde_json::to_string_pretty(&golden_data)
        .expect("Failed to serialize");
    
    let deserialized: GoldenData = serde_json::from_str(&json)
        .expect("Failed to deserialize");
    
    assert_eq!(golden_data, deserialized, "Golden data should survive serialization roundtrip");
}