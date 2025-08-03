use crate::ast::{ASTNode, SimplifiedAST};
use code_context_graph_core::{Result, Language, Hash};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct VisitorContext {
    pub language: Language,
    pub source: String,
    pub file_path: std::path::PathBuf,
    pub current_scope: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl VisitorContext {
    pub fn new(language: Language, source: String, file_path: std::path::PathBuf) -> Self {
        Self {
            language,
            source,
            file_path,
            current_scope: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn push_scope(&mut self, scope_name: String) {
        self.current_scope.push(scope_name);
    }

    pub fn pop_scope(&mut self) -> Option<String> {
        self.current_scope.pop()
    }

    pub fn current_scope_path(&self) -> String {
        self.current_scope.join("::")
    }

    pub fn add_metadata<T: serde::Serialize>(&mut self, key: &str, value: T) {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.metadata.insert(key.to_string(), json_value);
        }
    }

    pub fn get_metadata<T>(&self, key: &str) -> Option<T> 
    where 
        T: for<'de> serde::Deserialize<'de>
    {
        self.metadata.get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VisitResult {
    Continue,
    Skip,
    Stop,
}

pub trait ASTVisitor {
    type Output;

    fn visit_ast(&mut self, ast: &SimplifiedAST, context: &mut VisitorContext) -> Result<Self::Output>;
    
    fn visit_node(&mut self, node: &ASTNode, context: &mut VisitorContext) -> Result<VisitResult>;
    
    fn visit_children(&mut self, node: &ASTNode, context: &mut VisitorContext) -> Result<VisitResult> {
        for child in &node.children {
            match self.visit_node(child, context)? {
                VisitResult::Continue => continue,
                VisitResult::Skip => continue, // Skip this child's subtree but continue with siblings
                VisitResult::Stop => return Ok(VisitResult::Stop),
            }
        }
        Ok(VisitResult::Continue)
    }
}

pub struct WalkingVisitor<V: ASTVisitor> {
    visitor: V,
}

impl<V: ASTVisitor> WalkingVisitor<V> {
    pub fn new(visitor: V) -> Self {
        Self { visitor }
    }

    pub fn walk(&mut self, ast: &SimplifiedAST, context: &mut VisitorContext) -> Result<V::Output> {
        self.visitor.visit_ast(ast, context)
    }

    pub fn walk_node(&mut self, node: &ASTNode, context: &mut VisitorContext) -> Result<()> {
        self.walk_node_recursive(node, context)?;
        Ok(())
    }

    fn walk_node_recursive(&mut self, node: &ASTNode, context: &mut VisitorContext) -> Result<VisitResult> {
        // Visit current node
        match self.visitor.visit_node(node, context)? {
            VisitResult::Continue => {
                // Continue to children
                self.visitor.visit_children(node, context)
            },
            VisitResult::Skip => Ok(VisitResult::Continue), // Skip this subtree but continue
            VisitResult::Stop => Ok(VisitResult::Stop), // Stop entire traversal
        }
    }
}

// Utility trait for visitor composition
pub trait VisitorComposer {
    fn compose<V1, V2>(visitor1: V1, visitor2: V2) -> CompositeVisitor<V1, V2>
    where
        V1: ASTVisitor,
        V2: ASTVisitor;
}

pub struct CompositeVisitor<V1, V2> {
    visitor1: V1,
    visitor2: V2,
}

impl<V1, V2> CompositeVisitor<V1, V2>
where
    V1: ASTVisitor,
    V2: ASTVisitor,
{
    pub fn new(visitor1: V1, visitor2: V2) -> Self {
        Self { visitor1, visitor2 }
    }
}

impl<V1, V2> ASTVisitor for CompositeVisitor<V1, V2>
where
    V1: ASTVisitor,
    V2: ASTVisitor,
{
    type Output = V1::Output;

    fn visit_ast(&mut self, ast: &SimplifiedAST, context: &mut VisitorContext) -> Result<Self::Output> {
        let result1 = self.visitor1.visit_ast(ast, context)?;
        let _result2 = self.visitor2.visit_ast(ast, context)?;
        // TODO: Implement proper result combination based on specific output types
        Ok(result1)
    }

    fn visit_node(&mut self, node: &ASTNode, context: &mut VisitorContext) -> Result<VisitResult> {
        // Visit with first visitor
        let result1 = self.visitor1.visit_node(node, context)?;
        if result1 == VisitResult::Stop {
            return Ok(VisitResult::Stop);
        }

        // Visit with second visitor
        let result2 = self.visitor2.visit_node(node, context)?;
        
        // Combine results (Stop takes precedence, then Skip, then Continue)
        match (result1, result2) {
            (VisitResult::Stop, _) | (_, VisitResult::Stop) => Ok(VisitResult::Stop),
            (VisitResult::Skip, _) | (_, VisitResult::Skip) => Ok(VisitResult::Skip),
            (VisitResult::Continue, VisitResult::Continue) => Ok(VisitResult::Continue),
        }
    }
}

// Predefined visitor for filtering nodes
pub struct FilterVisitor<F> {
    filter: F,
    collected: Vec<ASTNode>,
}

impl<F> FilterVisitor<F>
where
    F: Fn(&ASTNode, &VisitorContext) -> bool,
{
    pub fn new(filter: F) -> Self {
        Self {
            filter,
            collected: Vec::new(),
        }
    }
}

impl<F> ASTVisitor for FilterVisitor<F>
where
    F: Fn(&ASTNode, &VisitorContext) -> bool,
{
    type Output = Vec<ASTNode>;

    fn visit_ast(&mut self, ast: &SimplifiedAST, context: &mut VisitorContext) -> Result<Self::Output> {
        self.visit_node(&ast.root, context)?;
        Ok(self.collected.clone())
    }

    fn visit_node(&mut self, node: &ASTNode, context: &mut VisitorContext) -> Result<VisitResult> {
        if (self.filter)(node, context) {
            self.collected.push(node.clone());
        }
        
        // Continue visiting children
        self.visit_children(node, context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{ASTNodeType, NodeLocation};

    struct TestVisitor {
        visited_nodes: Vec<String>,
    }

    impl ASTVisitor for TestVisitor {
        type Output = Vec<String>;

        fn visit_ast(&mut self, ast: &SimplifiedAST, context: &mut VisitorContext) -> Result<Self::Output> {
            self.visit_node(&ast.root, context)?;
            Ok(self.visited_nodes.clone())
        }

        fn visit_node(&mut self, node: &ASTNode, context: &mut VisitorContext) -> Result<VisitResult> {
            self.visited_nodes.push(format!("{:?}", node.node_type));
            self.visit_children(node, context)
        }
    }

    #[test]
    fn test_visitor_context() {
        let mut context = VisitorContext::new(
            Language::Python, 
            "test source".to_string(), 
            std::path::PathBuf::from("test.py")
        );

        context.push_scope("module".to_string());
        context.push_scope("class".to_string());
        
        assert_eq!(context.current_scope_path(), "module::class");
        
        assert_eq!(context.pop_scope(), Some("class".to_string()));
        assert_eq!(context.current_scope_path(), "module");
    }

    #[test]
    fn test_filter_visitor() {
        use crate::ast::SimplifiedAST;
        
        let location = NodeLocation::new(1, 0, 10, 0, 0, 100);
        let mut root = ASTNode::new(ASTNodeType::Program, None, location.clone());
        
        let func1 = ASTNode::new(ASTNodeType::FunctionDeclaration, Some("func1".to_string()), location.clone());
        let func2 = ASTNode::new(ASTNodeType::FunctionDeclaration, Some("func2".to_string()), location.clone());
        let class1 = ASTNode::new(ASTNodeType::ClassDeclaration, Some("Class1".to_string()), location.clone());
        
        root.add_child(func1);
        root.add_child(func2);
        root.add_child(class1);
        
        let ast = SimplifiedAST::new(root, Language::Python, "test");
        let mut context = VisitorContext::new(Language::Python, "test".to_string(), std::path::PathBuf::from("test.py"));
        
        let mut filter_visitor = FilterVisitor::new(|node, _| {
            matches!(node.node_type, ASTNodeType::FunctionDeclaration)
        });
        
        let functions = filter_visitor.visit_ast(&ast, &mut context).unwrap();
        assert_eq!(functions.len(), 2);
    }
}