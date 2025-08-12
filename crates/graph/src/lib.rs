use anyhow::Result;
use code_context_graph_core::Language;
use code_context_graph_parser::ast::{ASTNode, ASTNodeType, SimplifiedAST};
use redis::{Client as RedisClient, Value as RedisValue};
use std::sync::{Arc, Mutex};

pub struct GraphBuilder {
    graph_name: String,
}

impl GraphBuilder {
    pub fn new(graph_name: &str) -> Self {
        Self { graph_name: graph_name.to_string() }
    }

    pub fn build_queries(&self, ast: &SimplifiedAST, file_path: &str) -> Vec<String> {
        let mut queries = Vec::new();
        // File node
        queries.push(format!("MERGE (f:File {{ path: '{}' }})", file_path.replace('\\', "/")));
        // Walk AST for simple entities
        self.walk(&ast.root, ast.language, &mut queries);
        queries
    }

    fn walk(&self, node: &ASTNode, language: Language, queries: &mut Vec<String>) {
        match node.node_type {
            ASTNodeType::ClassDeclaration => {
                if let Some(name) = &node.name {
                    queries.push(format!("MERGE (cls:Class {{ name: '{}' }})", escape(name)));
                    queries.push("MERGE (f)-[:CONTAINS]->(cls)".to_string());
                }
            }
            ASTNodeType::FunctionDeclaration | ASTNodeType::MethodDeclaration => {
                if let Some(name) = &node.name {
                    queries.push(format!("MERGE (fn:Function {{ name: '{}' }})", escape(name)));
                    queries.push("MERGE (f)-[:CONTAINS]->(fn)".to_string());
                }
            }
            ASTNodeType::ImportDeclaration => {
                if let Some(name) = &node.name {
                    queries.push(format!("MERGE (m:Module {{ name: '{}' }})", escape(name)));
                    queries.push("MERGE (f)-[:IMPORTS]->(m)".to_string());
                }
            }
            _ => {}
        }
        for child in &node.children {
            self.walk(child, language, queries);
        }
    }
}

fn escape(s: &str) -> String { s.replace("'", "\\'") }

pub trait GraphExecutor: Send + Sync {
    fn query(&self, graph: &str, cypher: &str) -> anyhow::Result<RedisValue>;
}

pub struct RedisExecutor {
    client: RedisClient,
}

impl RedisExecutor {
    pub fn new(url: &str) -> anyhow::Result<Self> {
        Ok(Self { client: RedisClient::open(url)? })
    }
}

impl GraphExecutor for RedisExecutor {
    fn query(&self, graph: &str, cypher: &str) -> anyhow::Result<RedisValue> {
        let mut conn = self.client.get_connection()?;
        // Execute FalkorDB/RedisGraph query
        let val: redis::Value = redis::cmd("GRAPH.QUERY")
            .arg(graph)
            .arg(cypher)
            .query(&mut conn)?;
        Ok(val)
    }
}

pub struct GraphClient {
    graph_name: String,
    exec: Box<dyn GraphExecutor>,
    recorded: Arc<Mutex<Vec<(String, String)>>>,
}

impl GraphClient {
    pub fn new_with_redis(url: &str, graph_name: &str) -> anyhow::Result<Self> {
        let exec = Box::new(RedisExecutor::new(url)?);
        Ok(Self::with_executor(graph_name, exec))
    }

    pub fn with_executor(graph_name: &str, exec: Box<dyn GraphExecutor>) -> Self {
        Self {
            graph_name: graph_name.to_string(),
            exec,
            recorded: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn persist_queries(&self, queries: &[String]) -> anyhow::Result<()> {
        for q in queries {
            self.recorded.lock().unwrap().push((self.graph_name.clone(), q.clone()));
            let _ = self.exec.query(&self.graph_name, q)?;
        }
        Ok(())
    }

    pub fn execute(&self, cypher: &str) -> anyhow::Result<redis::Value> {
        self.recorded
            .lock()
            .unwrap()
            .push((self.graph_name.clone(), cypher.to_string()));
        self.exec.query(&self.graph_name, cypher)
    }

    // Test-only helper
    pub fn recorded_for_tests(&self) -> Vec<(String, String)> {
        self.recorded.lock().unwrap().clone()
    }
}