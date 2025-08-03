use code_context_graph_core::{Language, Hash};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ASTNode {
    pub id: Hash,
    pub node_type: ASTNodeType,
    pub name: Option<String>,
    pub location: NodeLocation,
    pub children: Vec<ASTNode>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ASTNodeType {
    // File level
    Program,
    Module,
    
    // Declarations
    ClassDeclaration,
    FunctionDeclaration,
    MethodDeclaration,
    VariableDeclaration,
    ImportDeclaration,
    TypeDeclaration,
    InterfaceDeclaration,
    EnumDeclaration,
    
    // Expressions
    CallExpression,
    MemberExpression,
    Identifier,
    Literal,
    AssignmentExpression,
    BinaryExpression,
    UnaryExpression,
    
    // Statements
    ExpressionStatement,
    ReturnStatement,
    IfStatement,
    ForStatement,
    WhileStatement,
    BlockStatement,
    
    // Language specific
    Decorator,       // Python
    Annotation,      // Java/Kotlin
    Lambda,          // Python/Java/Kotlin
    Comprehension,   // Python
    
    // Generic
    Other(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NodeLocation {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub start_byte: u32,
    pub end_byte: u32,
}

impl ASTNode {
    pub fn new(
        node_type: ASTNodeType,
        name: Option<String>,
        location: NodeLocation,
    ) -> Self {
        let content = format!("{:?}:{}:{}:{}", 
            node_type, 
            name.as_deref().unwrap_or(""), 
            location.start_line, 
            location.start_column
        );
        let id = Hash::from_string(&content);

        Self {
            id,
            node_type,
            name,
            location,
            children: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn add_child(&mut self, child: ASTNode) {
        self.children.push(child);
    }

    pub fn add_metadata<T: Serialize>(&mut self, key: &str, value: T) {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.metadata.insert(key.to_string(), json_value);
        }
    }

    pub fn get_metadata<T>(&self, key: &str) -> Option<T> 
    where 
        T: for<'de> Deserialize<'de>
    {
        self.metadata.get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    pub fn find_children_by_type(&self, node_type: &ASTNodeType) -> Vec<&ASTNode> {
        let mut result = Vec::new();
        
        for child in &self.children {
            if &child.node_type == node_type {
                result.push(child);
            }
            result.extend(child.find_children_by_type(node_type));
        }
        
        result
    }

    pub fn find_child_by_name(&self, name: &str) -> Option<&ASTNode> {
        for child in &self.children {
            if child.name.as_deref() == Some(name) {
                return Some(child);
            }
            if let Some(found) = child.find_child_by_name(name) {
                return Some(found);
            }
        }
        None
    }

    pub fn get_text_content<'a>(&self, source: &'a str) -> &'a str {
        let start = self.location.start_byte as usize;
        let end = self.location.end_byte as usize;
        
        if start < source.len() && end <= source.len() && start <= end {
            &source[start..end]
        } else {
            ""
        }
    }

    pub fn is_declaration(&self) -> bool {
        matches!(self.node_type, 
            ASTNodeType::ClassDeclaration |
            ASTNodeType::FunctionDeclaration |
            ASTNodeType::MethodDeclaration |
            ASTNodeType::VariableDeclaration |
            ASTNodeType::TypeDeclaration |
            ASTNodeType::InterfaceDeclaration |
            ASTNodeType::EnumDeclaration
        )
    }

    pub fn is_expression(&self) -> bool {
        matches!(self.node_type,
            ASTNodeType::CallExpression |
            ASTNodeType::MemberExpression |
            ASTNodeType::Identifier |
            ASTNodeType::Literal |
            ASTNodeType::AssignmentExpression |
            ASTNodeType::BinaryExpression |
            ASTNodeType::UnaryExpression
        )
    }
}

impl NodeLocation {
    pub fn new(
        start_line: u32,
        start_column: u32,
        end_line: u32,
        end_column: u32,
        start_byte: u32,
        end_byte: u32,
    ) -> Self {
        Self {
            start_line,
            start_column,
            end_line,
            end_column,
            start_byte,
            end_byte,
        }
    }

    pub fn from_tree_sitter(node: tree_sitter::Node) -> Self {
        let start_pos = node.start_position();
        let end_pos = node.end_position();
        
        Self::new(
            start_pos.row as u32 + 1,  // Convert to 1-based line numbers
            start_pos.column as u32,
            end_pos.row as u32 + 1,
            end_pos.column as u32,
            node.start_byte() as u32,
            node.end_byte() as u32,
        )
    }

    pub fn contains(&self, other: &NodeLocation) -> bool {
        self.start_byte <= other.start_byte && other.end_byte <= self.end_byte
    }

    pub fn overlaps(&self, other: &NodeLocation) -> bool {
        !(self.end_byte <= other.start_byte || other.end_byte <= self.start_byte)
    }
}

impl ASTNodeType {
    pub fn from_tree_sitter_kind(kind: &str, language: Language) -> Self {
        match (kind, language) {
            // Common across languages
            ("program", _) | ("source_file", _) => ASTNodeType::Program,
            ("module", _) => ASTNodeType::Module,
            
            // Class declarations
            ("class_definition", Language::Python) |
            ("class_declaration", Language::Java) |
            ("class_declaration", Language::Kotlin) |
            ("object_declaration", Language::Kotlin) |
            ("class_declaration", Language::JavaScript) => ASTNodeType::ClassDeclaration,
            
            // Function declarations
            ("function_definition", Language::Python) |
            ("function_declaration", _) |
            ("function", Language::JavaScript) |
            ("arrow_function", Language::JavaScript) => ASTNodeType::FunctionDeclaration,
            
            // Method declarations  
            ("method_declaration", _) |
            ("method_definition", _) |
            ("constructor_declaration", _) => ASTNodeType::MethodDeclaration,
            
            // Variable declarations
            ("assignment", Language::Python) |
            ("variable_declaration", _) |
            ("local_variable_declaration", Language::Java) => ASTNodeType::VariableDeclaration,
            
            // Import statements
            ("import_statement", _) |
            ("import_from_statement", Language::Python) |
            ("import_declaration", _) => ASTNodeType::ImportDeclaration,
            
            // Call expressions
            ("call", Language::Python) |
            ("call_expression", _) |
            ("method_invocation", Language::Java) => ASTNodeType::CallExpression,
            
            // Member access
            ("attribute", Language::Python) |
            ("member_expression", _) |
            ("field_access", Language::Java) => ASTNodeType::MemberExpression,
            
            // Identifiers and literals
            ("identifier", _) => ASTNodeType::Identifier,
            ("string", _) | ("integer", _) | ("float", _) | ("boolean", _) => ASTNodeType::Literal,
            
            // Control flow
            ("if_statement", _) => ASTNodeType::IfStatement,
            ("for_statement", _) | ("for_in_statement", _) => ASTNodeType::ForStatement,
            ("while_statement", _) => ASTNodeType::WhileStatement,
            ("return_statement", _) => ASTNodeType::ReturnStatement,
            
            // Blocks
            ("block", _) | ("suite", Language::Python) => ASTNodeType::BlockStatement,
            
            // Interfaces and Enums
            ("interface_declaration", _) => ASTNodeType::InterfaceDeclaration,
            ("enum_declaration", _) |
            ("enum_class_declaration", Language::Kotlin) => ASTNodeType::EnumDeclaration,
            
            // Annotations
            ("annotation", _) |
            ("modifiers", _) => ASTNodeType::Annotation,
            
            // Language specific
            ("decorator", Language::Python) => ASTNodeType::Decorator,
            ("lambda", _) => ASTNodeType::Lambda,
            
            // Default
            _ => ASTNodeType::Other(kind.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_node_creation() {
        let location = NodeLocation::new(1, 0, 1, 10, 0, 10);
        let node = ASTNode::new(
            ASTNodeType::FunctionDeclaration,
            Some("test_func".to_string()),
            location,
        );

        assert_eq!(node.node_type, ASTNodeType::FunctionDeclaration);
        assert_eq!(node.name, Some("test_func".to_string()));
        assert!(node.children.is_empty());
        assert!(node.metadata.is_empty());
    }

    #[test]
    fn test_node_metadata() {
        let location = NodeLocation::new(1, 0, 1, 10, 0, 10);
        let mut node = ASTNode::new(
            ASTNodeType::FunctionDeclaration,
            Some("test_func".to_string()),
            location,
        );

        node.add_metadata("visibility", "public");
        node.add_metadata("parameters", vec!["a", "b"]);

        assert_eq!(node.get_metadata::<String>("visibility"), Some("public".to_string()));
        assert_eq!(node.get_metadata::<Vec<String>>("parameters"), Some(vec!["a".to_string(), "b".to_string()]));
    }

    #[test]
    fn test_location_contains() {
        let outer = NodeLocation::new(1, 0, 10, 0, 0, 100);
        let inner = NodeLocation::new(2, 0, 5, 0, 10, 50);
        let outside = NodeLocation::new(11, 0, 15, 0, 101, 150);

        assert!(outer.contains(&inner));
        assert!(!outer.contains(&outside));
        assert!(!inner.contains(&outer));
    }

    #[test]
    fn test_find_children_by_type() {
        let location = NodeLocation::new(1, 0, 10, 0, 0, 100);
        let mut root = ASTNode::new(ASTNodeType::Program, None, location.clone());
        
        let func1 = ASTNode::new(ASTNodeType::FunctionDeclaration, Some("func1".to_string()), location.clone());
        let func2 = ASTNode::new(ASTNodeType::FunctionDeclaration, Some("func2".to_string()), location.clone());
        let class1 = ASTNode::new(ASTNodeType::ClassDeclaration, Some("Class1".to_string()), location.clone());
        
        root.add_child(func1);
        root.add_child(func2);
        root.add_child(class1);

        let functions = root.find_children_by_type(&ASTNodeType::FunctionDeclaration);
        assert_eq!(functions.len(), 2);
        
        let classes = root.find_children_by_type(&ASTNodeType::ClassDeclaration);
        assert_eq!(classes.len(), 1);
    }
}