use code_context_graph_core::{Language, Result, CodeGraphError};
use crate::ast::{ASTNode, ASTNodeType, NodeLocation};
use tree_sitter::Node;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplifiedAST {
    pub root: ASTNode,
    pub language: Language,
    pub source_hash: code_context_graph_core::Hash,
}

impl SimplifiedAST {
    pub fn new(root: ASTNode, language: Language, source: &str) -> Self {
        let source_hash = code_context_graph_core::Hash::from_string(source);
        
        Self {
            root,
            language,
            source_hash,
        }
    }

    pub fn from_tree_sitter(node: Node, source: &str, language: Language) -> Result<Self> {
        let root = Self::convert_node(node, source, language)?;
        Ok(Self::new(root, language, source))
    }

    fn convert_node(node: Node, source: &str, language: Language) -> Result<ASTNode> {
        let node_type = ASTNodeType::from_tree_sitter_kind(node.kind(), language);
        let location = NodeLocation::from_tree_sitter(node);
        
        // Extract name for named nodes
        let name = Self::extract_node_name(&node, source, &node_type);
        
        let mut ast_node = ASTNode::new(node_type, name, location);
        
        // Add language-specific metadata
        Self::add_node_metadata(&mut ast_node, &node, source, language);
        
        // Convert children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if Self::should_include_node(&child, language) {
                let child_ast = Self::convert_node(child, source, language)?;
                ast_node.add_child(child_ast);
            }
        }
        
        Ok(ast_node)
    }

    fn extract_node_name(node: &Node, source: &str, node_type: &ASTNodeType) -> Option<String> {
        match node_type {
            ASTNodeType::ClassDeclaration |
            ASTNodeType::FunctionDeclaration |
            ASTNodeType::MethodDeclaration |
            ASTNodeType::VariableDeclaration |
            ASTNodeType::InterfaceDeclaration |
            ASTNodeType::EnumDeclaration => {
                Self::find_name_child(node, source)
            },
            ASTNodeType::Identifier => {
                Some(node.utf8_text(source.as_bytes()).unwrap_or("").to_string())
            },
            ASTNodeType::ImportDeclaration => {
                Self::extract_import_name(node, source)
            },
            _ => None,
        }
    }

    fn find_name_child(node: &Node, source: &str) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if matches!(child.kind(), "identifier" | "name" | "type_identifier" | "simple_identifier") {
                if let Ok(text) = child.utf8_text(source.as_bytes()) {
                    return Some(text.to_string());
                }
            }
            // For nested structures, look deeper
            if let Some(nested_name) = Self::find_name_child(&child, source) {
                return Some(nested_name);
            }
        }
        None
    }

    fn extract_import_name(node: &Node, source: &str) -> Option<String> {
        // Try to get the main imported module/package name
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "dotted_name" | "module_name" | "identifier" | "string_literal" => {
                    if let Ok(text) = child.utf8_text(source.as_bytes()) {
                        // Remove quotes from string literals
                        let clean_text = text.trim_matches('"').trim_matches('\'');
                        return Some(clean_text.to_string());
                    }
                },
                "scoped_identifier" => {
                    // Handle JavaScript style imports
                    if let Some(name) = Self::find_name_child(&child, source) {
                        return Some(name);
                    }
                },
                _ => {}
            }
        }
        None
    }

    fn add_node_metadata(ast_node: &mut ASTNode, node: &Node, source: &str, language: Language) {
        // Add common metadata
        ast_node.add_metadata("kind", node.kind());
        ast_node.add_metadata("is_named", node.is_named());
        ast_node.add_metadata("language", language);
        
        // Add text content for small nodes
        if let Ok(text) = node.utf8_text(source.as_bytes()) {
            if text.len() < 100 { // Only for small nodes to avoid large metadata
                ast_node.add_metadata("text", text);
            }
        }
        
        // Language-specific metadata
        match language {
            Language::Python => Self::add_python_metadata(ast_node, node, source),
            Language::Java => Self::add_java_metadata(ast_node, node, source),
            Language::JavaScript => Self::add_javascript_metadata(ast_node, node, source),
            Language::Kotlin => Self::add_kotlin_metadata(ast_node, node, source),
            _ => {}
        }
    }

    fn add_python_metadata(ast_node: &mut ASTNode, node: &Node, source: &str) {
        match node.kind() {
            "function_definition" => {
                // Check for decorators - they might be children or siblings
                let decorators = Self::extract_python_decorators(node, source);
                if !decorators.is_empty() {
                    ast_node.add_metadata("decorators", decorators);
                }
            },
            "class_definition" => {
                // Check for base classes
                if let Some(base_classes) = Self::extract_python_base_classes(node, source) {
                    ast_node.add_metadata("base_classes", base_classes);
                }
            },
            _ => {}
        }
    }

    fn add_java_metadata(ast_node: &mut ASTNode, node: &Node, source: &str) {
        match node.kind() {
            "method_declaration" | "constructor_declaration" => {
                // Extract modifiers
                let modifiers = Self::extract_java_modifiers(node, source);
                if !modifiers.is_empty() {
                    ast_node.add_metadata("modifiers", modifiers);
                }
            },
            "class_declaration" => {
                // Extract extends and implements
                if let Some(extends) = Self::extract_java_extends(node, source) {
                    ast_node.add_metadata("extends", extends);
                }
                if let Some(implements) = Self::extract_java_implements(node, source) {
                    ast_node.add_metadata("implements", implements);
                }
            },
            _ => {}
        }
    }

    fn add_javascript_metadata(ast_node: &mut ASTNode, node: &Node, source: &str) {
        match node.kind() {
            "function_declaration" | "method_definition" => {
                // Check if async
                let mut cursor = node.walk();
                let is_async = node.children(&mut cursor)
                    .any(|child| child.kind() == "async");
                
                if is_async {
                    ast_node.add_metadata("async", true);
                }
            
                // Check if generator
                let is_generator = node.children(&mut cursor)
                    .any(|child| child.kind() == "*");
                
                if is_generator {
                    ast_node.add_metadata("generator", true);
                }
            },
            "class_declaration" => {
                // Extract extends
                if let Some(extends) = Self::extract_js_extends(node, source) {
                    ast_node.add_metadata("extends", extends);
                }
            },
            _ => {}
        }
    }

    fn add_kotlin_metadata(ast_node: &mut ASTNode, node: &Node, source: &str) {
        match node.kind() {
            "function_declaration" => {
                // Extract modifiers
                let modifiers = Self::extract_kotlin_modifiers(node, source);
                if !modifiers.is_empty() {
                    ast_node.add_metadata("modifiers", modifiers);
                }
            },
            "class_declaration" | "object_declaration" => {
                // Extract modifiers for classes/objects
                let modifiers = Self::extract_kotlin_modifiers(node, source);
                if !modifiers.is_empty() {
                    ast_node.add_metadata("modifiers", modifiers);
                }
                
                // Extract parent types
                if let Some(parents) = Self::extract_kotlin_parents(node, source) {
                    ast_node.add_metadata("parents", parents);
                }
            },
            _ => {}
        }
    }

    fn should_include_node(node: &Node, language: Language) -> bool {
        // Filter out noise nodes but keep important structural elements
        let kind = node.kind();
        
        // Always exclude pure punctuation and comments
        if matches!(kind, 
            "(" | ")" | "[" | "]" | "{" | "}" | ";" | "," | "." | ":" |
            "comment" | "line_comment" | "block_comment" | "multiline_comment"
        ) {
            return false;
        }
        
        // Include all named nodes (they usually contain important information)
        if node.is_named() {
            return true;
        }
        
        // Language-specific filtering for unnamed nodes
        match language {
            Language::Python => {
                // Include important keywords and operators
                matches!(kind, "def" | "class" | "import" | "from" | "as" | "async" | "await")
            },
            Language::Java => {
                matches!(kind, "public" | "private" | "protected" | "static" | "final" | "abstract" | "class" | "interface" | "enum")
            },
            Language::JavaScript => {
                matches!(kind, "class" | "function" | "async" | "await" | "import" | "export" | "const" | "let" | "var")
            },
            Language::Kotlin => {
                matches!(kind, "class" | "fun" | "val" | "var" | "object" | "interface" | "enum" | "data" | "sealed")
            },
            _ => true,
        }
    }

    // Helper methods for extracting language-specific information
    fn extract_python_decorators(node: &Node, source: &str) -> Vec<String> {
        let mut decorators = Vec::new();
        
        // First, check direct children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "decorator" {
                if let Ok(text) = child.utf8_text(source.as_bytes()) {
                    decorators.push(text.to_string());
                }
            }
        }
        
        // If no direct children decorators found, check parent for decorated_definition
        if decorators.is_empty() {
            if let Some(parent) = node.parent() {
                if parent.kind() == "decorated_definition" {
                    let mut parent_cursor = parent.walk();
                    for sibling in parent.children(&mut parent_cursor) {
                        if sibling.kind() == "decorator" {
                            if let Ok(text) = sibling.utf8_text(source.as_bytes()) {
                                decorators.push(text.to_string());
                            }
                        }
                    }
                }
            }
        }
        
        decorators
    }
    
    fn extract_python_base_classes(node: &Node, source: &str) -> Option<Vec<String>> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "argument_list" {
                let mut base_classes = Vec::new();
                let mut child_cursor = child.walk();
                for grandchild in child.children(&mut child_cursor) {
                    if grandchild.kind() == "identifier" {
                        if let Ok(text) = grandchild.utf8_text(source.as_bytes()) {
                            base_classes.push(text.to_string());
                        }
                    }
                }
                if !base_classes.is_empty() {
                    return Some(base_classes);
                }
            }
        }
        None
    }

    fn extract_java_modifiers(node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if matches!(child.kind(), "public" | "private" | "protected" | "static" | "final" | "abstract" | "synchronized") {
                if let Ok(text) = child.utf8_text(source.as_bytes()) {
                    modifiers.push(text.to_string());
                }
            }
        }
        modifiers
    }

    fn extract_java_extends(node: &Node, source: &str) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "superclass" {
                // Look for identifier within superclass
                let mut child_cursor = child.walk();
                for grandchild in child.children(&mut child_cursor) {
                    if grandchild.kind() == "type_identifier" || grandchild.kind() == "identifier" {
                        if let Ok(text) = grandchild.utf8_text(source.as_bytes()) {
                            return Some(text.to_string());
                        }
                    }
                }
                // Fallback to full text if no identifier found
                if let Ok(text) = child.utf8_text(source.as_bytes()) {
                    let clean_text = text.trim().replace("extends ", "");
                    if !clean_text.is_empty() {
                        return Some(clean_text);
                    }
                }
            }
        }
        None
    }

    fn extract_java_implements(node: &Node, source: &str) -> Option<Vec<String>> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "super_interfaces" {
                let mut interfaces = Vec::new();
                let mut child_cursor = child.walk();
                for grandchild in child.children(&mut child_cursor) {
                    if grandchild.kind() == "type_identifier" || grandchild.kind() == "identifier" {
                        if let Ok(text) = grandchild.utf8_text(source.as_bytes()) {
                            interfaces.push(text.to_string());
                        }
                    }
                }
                if !interfaces.is_empty() {
                    return Some(interfaces);
                }
            }
        }
        None
    }

    fn extract_js_extends(node: &Node, source: &str) -> Option<String> {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "class_heritage" {
                if let Ok(text) = child.utf8_text(source.as_bytes()) {
                    return Some(text.to_string());
                }
            }
        }
        None
    }

    fn extract_kotlin_modifiers(node: &Node, source: &str) -> Vec<String> {
        let mut modifiers = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            // Check for modifier nodes or direct modifier keywords
            if matches!(child.kind(), 
                "visibility_modifier" | "inheritance_modifier" | "function_modifier" |
                "modifiers" | "modifier"
            ) {
                if let Ok(text) = child.utf8_text(source.as_bytes()) {
                    modifiers.push(text.to_string());
                }
            } else if matches!(child.kind(),
                "suspend" | "inline" | "private" | "public" | "protected" | "internal" |
                "abstract" | "final" | "open" | "override" | "data" | "sealed" |
                "inner" | "enum" | "annotation" | "companion" | "lateinit" | "vararg"
            ) {
                if let Ok(text) = child.utf8_text(source.as_bytes()) {
                    modifiers.push(text.to_string());
                }
            }
            
            // Also check within modifier lists
            if child.kind() == "modifiers" {
                let mut child_cursor = child.walk();
                for grandchild in child.children(&mut child_cursor) {
                    if let Ok(text) = grandchild.utf8_text(source.as_bytes()) {
                        modifiers.push(text.to_string());
                    }
                }
            }
        }
        modifiers
    }

    fn extract_kotlin_parents(node: &Node, source: &str) -> Option<Vec<String>> {
        let mut parents = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            // Check for inheritance specification
            if matches!(child.kind(), "delegation_specifiers" | "supertype_list" | "superclass") {
                let mut child_cursor = child.walk();
                for grandchild in child.children(&mut child_cursor) {
                    if matches!(grandchild.kind(), 
                        "constructor_invocation" | "user_type" | "type_identifier" | 
                        "simple_identifier" | "identifier" | "supertype"
                    ) {
                        if let Ok(text) = grandchild.utf8_text(source.as_bytes()) {
                            // Clean up the text (remove constructor call syntax)
                            let clean_text = text.split('(').next().unwrap_or(text).trim();
                            if !clean_text.is_empty() {
                                parents.push(clean_text.to_string());
                            }
                        }
                    }
                    
                    // Also check nested identifiers within these nodes
                    if matches!(grandchild.kind(), "constructor_invocation" | "user_type") {
                        let mut nested_cursor = grandchild.walk();
                        for nested_child in grandchild.children(&mut nested_cursor) {
                            if matches!(nested_child.kind(), "type_identifier" | "simple_identifier") {
                                if let Ok(text) = nested_child.utf8_text(source.as_bytes()) {
                                    parents.push(text.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        if !parents.is_empty() {
            Some(parents)
        } else {
            None
        }
    }

    // Public API for AST traversal
    pub fn find_all_functions(&self) -> Vec<&ASTNode> {
        let mut result = self.root.find_children_by_type(&ASTNodeType::FunctionDeclaration);
        result.extend(self.root.find_children_by_type(&ASTNodeType::MethodDeclaration));
        result
    }

    pub fn find_all_classes(&self) -> Vec<&ASTNode> {
        let mut result = self.root.find_children_by_type(&ASTNodeType::ClassDeclaration);
        result.extend(self.root.find_children_by_type(&ASTNodeType::InterfaceDeclaration));
        result
    }

    pub fn find_all_imports(&self) -> Vec<&ASTNode> {
        self.root.find_children_by_type(&ASTNodeType::ImportDeclaration)
    }

    pub fn find_all_calls(&self) -> Vec<&ASTNode> {
        self.root.find_children_by_type(&ASTNodeType::CallExpression)
    }
    
    pub fn find_all_interfaces(&self) -> Vec<&ASTNode> {
        self.root.find_children_by_type(&ASTNodeType::InterfaceDeclaration)
    }
    
    pub fn find_all_enums(&self) -> Vec<&ASTNode> {
        self.root.find_children_by_type(&ASTNodeType::EnumDeclaration)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simplified_ast_creation() {
        let location = NodeLocation::new(1, 0, 10, 0, 0, 100);
        let root = ASTNode::new(ASTNodeType::Program, None, location);
        let source = "test source";
        
        let ast = SimplifiedAST::new(root, Language::Python, source);
        
        assert_eq!(ast.language, Language::Python);
        assert_eq!(ast.source_hash, code_context_graph_core::Hash::from_string(source));
    }

    #[test]
    fn test_find_functions() {
        let location = NodeLocation::new(1, 0, 10, 0, 0, 100);
        let mut root = ASTNode::new(ASTNodeType::Program, None, location.clone());
        
        let func1 = ASTNode::new(ASTNodeType::FunctionDeclaration, Some("func1".to_string()), location.clone());
        let func2 = ASTNode::new(ASTNodeType::FunctionDeclaration, Some("func2".to_string()), location.clone());
        
        root.add_child(func1);
        root.add_child(func2);
        
        let ast = SimplifiedAST::new(root, Language::Python, "test");
        let functions = ast.find_all_functions();
        
        assert_eq!(functions.len(), 2);
    }
}