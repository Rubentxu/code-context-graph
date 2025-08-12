use code_context_graph_parser::test_utils::TestUtils;
use code_context_graph_core::Language;
use code_context_graph_graph::GraphBuilder;

fn contains_all(haystack: &str, needles: &[&str]) {
    for n in needles {
        assert!(haystack.contains(n), "expected to find '{}', got: {}", n, haystack);
    }

}

#[test]
fn builds_basic_python_class_graph_queries() {
    let source = r#"
class Foo:
    def bar(self):
        pass
"#;
    let ast = TestUtils::parse_source(source, Language::Python).unwrap();
    let builder = GraphBuilder::new("code_graph");
    let queries = builder.build_queries(&ast, "src/foo.py");
    let all = queries.join("\n");
    contains_all(&all, &[
        "MERGE (f:File",
        "{ path: 'src/foo.py' }",
        "MERGE (cls:Class",
        "{ name: 'Foo' }",
        "MERGE (f)-[:CONTAINS]->(cls)",
    ]);
}

#[test]
fn builds_basic_python_function_graph_queries() {
    let source = r#"
def foo(x):
    return x
"#;
    let ast = TestUtils::parse_source(source, Language::Python).unwrap();
    let builder = GraphBuilder::new("code_graph");
    let queries = builder.build_queries(&ast, "src/main.py");
    let all = queries.join("\n");
    // Expect MERGE for File and Function, and CONTAINS relation
    contains_all(&all, &[
        "MERGE (f:File",
        "{ path: 'src/main.py' }",
        "MERGE (fn:Function",
        "{ name: 'foo' }",
        "MERGE (f)-[:CONTAINS]->(fn)",
    ]);
}

#[test]
fn includes_import_relationships_for_python() {
    let source = r#"
import os
def foo():
    return os.getcwd()
"#;
    let ast = TestUtils::parse_source(source, Language::Python).unwrap();
    let builder = GraphBuilder::new("code_graph");
    let queries = builder.build_queries(&ast, "app/util.py");
    let all = queries.join("\n");
    contains_all(&all, &[
        "MERGE (m:Module { name: 'os' })",
        "MERGE (f:File { path: 'app/util.py' })",
        "MERGE (f)-[:IMPORTS]->(m)",
    ]);
}
