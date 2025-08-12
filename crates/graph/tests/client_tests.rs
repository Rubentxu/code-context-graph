use code_context_graph_graph::{GraphBuilder, GraphClient, GraphExecutor};
use code_context_graph_parser::test_utils::TestUtils;
use code_context_graph_core::Language;
use std::sync::{Arc, Mutex};

struct MockExec(Arc<Mutex<Vec<(String, String)>>>);

impl MockExec {
    fn new() -> Self { Self(Arc::new(Mutex::new(Vec::new()))) }
    fn recorded(&self) -> Vec<(String,String)> { self.0.lock().unwrap().clone() }
}

impl GraphExecutor for MockExec {
    fn query(&self, graph: &str, cypher: &str) -> anyhow::Result<redis::Value> {
        self.0.lock().unwrap().push((graph.to_string(), cypher.to_string()));
        Ok(redis::Value::Okay)
    }
}

#[test]
fn persist_executes_all_queries_in_order() {
    let exec = MockExec::new();
    let client = GraphClient::with_executor("code_graph", Box::new(exec ));
    let queries = vec![
        "MERGE (f:File { path: 'a.py' })".to_string(),
        "MERGE (x:X)".to_string(),
    ];
    client.persist_queries(&queries).unwrap();
    let recorded = client.recorded_for_tests();
    assert_eq!(recorded.len(), 2);
    assert_eq!(recorded[0].0, "code_graph");
    assert_eq!(recorded[0].1, "MERGE (f:File { path: 'a.py' })");
    assert_eq!(recorded[1].1, "MERGE (x:X)");
}

#[test]
fn persist_ast_builds_and_executes() {
    let source = r#"
import os

def foo(x):
    return os.getcwd()
"#;
    let ast = TestUtils::parse_source(source, Language::Python).unwrap();
    let builder = GraphBuilder::new("code_graph");
    let exec = MockExec::new();
    let client = GraphClient::with_executor("code_graph", Box::new(exec));
    let queries = builder.build_queries(&ast, "src/app.py");
    client.persist_queries(&queries).unwrap();
    let recorded = client.recorded_for_tests();
    let all = recorded.iter().map(|(_,q)| q.clone()).collect::<Vec<_>>().join("\n");
    assert!(all.contains("MERGE (fn:Function { name: 'foo' })"));
    assert!(all.contains("MERGE (m:Module { name: 'os' })"));
}
