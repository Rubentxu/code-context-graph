use code_context_graph_graph::GraphClient;

fn falkor_url() -> String {
    std::env::var("FALKORDB_URL").unwrap_or_else(|_| "redis://localhost:6379".to_string())
}

#[test]
#[ignore]
fn live_falkordb_persist_and_query() {
    let url = falkor_url();
    let graph = format!("ccg_test_{}", std::process::id());
    let client = match GraphClient::new_with_redis(&url, &graph) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Skipping live_falkordb_persist_and_query: cannot connect to {}", url);
            return;
        }
    };

    // Write some data
    let writes = vec![
        "MERGE (a:TestNode { id: 1 })".to_string(),
        "MERGE (b:TestNode { id: 2 })".to_string(),
        "MERGE (a)-[:LINKS_TO]->(b)".to_string(),
    ];
    client.persist_queries(&writes).expect("persist should succeed");

    // Read back count
    let res = client.execute("MATCH (n:TestNode) RETURN count(n)").expect("query ok");
    // We don't depend on exact wire format; just ensure it's not Nil/Null/Empty
    match res {
        redis::Value::Nil => panic!("unexpected Nil result"),
        redis::Value::Bulk(ref arr) if arr.is_empty() => panic!("empty result"),
        _ => {}
    }
}
