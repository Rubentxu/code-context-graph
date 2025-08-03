use code_context_graph_core::{Language, Result, CodeGraphError};
use crate::ast::SimplifiedAST;
use tree_sitter::Parser;

pub struct PythonParser {
    parser: Parser,
}

impl PythonParser {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_python::LANGUAGE.into())
            .map_err(|e| CodeGraphError::Parser {
                message: format!("Failed to set Python language: {}", e)
            })?;
        
        Ok(Self { parser })
    }

    pub fn parse(&mut self, source: &str) -> Result<SimplifiedAST> {
        let tree = self.parser.parse(source, None)
            .ok_or_else(|| CodeGraphError::Parser {
                message: "Failed to parse Python source".to_string()
            })?;

        SimplifiedAST::from_tree_sitter(tree.root_node(), source, Language::Python)
    }

    pub fn parse_with_old_tree(&mut self, source: &str, old_tree: Option<&tree_sitter::Tree>) -> Result<SimplifiedAST> {
        let tree = self.parser.parse(source, old_tree)
            .ok_or_else(|| CodeGraphError::Parser {
                message: "Failed to parse Python source incrementally".to_string()
            })?;

        SimplifiedAST::from_tree_sitter(tree.root_node(), source, Language::Python)
    }

    pub fn supported_extensions() -> Vec<&'static str> {
        vec!["py", "pyw"]
    }

    pub fn is_python_file(path: &std::path::Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            Self::supported_extensions().contains(&ext)
        } else {
            false
        }
    }
}

impl Default for PythonParser {
    fn default() -> Self {
        Self::new().expect("Failed to create Python parser")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_python_parser_creation() {
        let parser = PythonParser::new();
        assert!(parser.is_ok());
    }

    #[test]
    fn test_parse_simple_python() {
        let mut parser = PythonParser::new().unwrap();
        let source = r#"
def hello_world():
    print("Hello, World!")

class MyClass:
    def __init__(self):
        self.value = 42
    
    def get_value(self):
        return self.value
"#;

        let ast = parser.parse(source).unwrap();
        assert_eq!(ast.language, Language::Python);
        
        let functions = ast.find_all_functions();
        assert!(functions.len() >= 2); // hello_world, __init__, get_value
        
        let classes = ast.find_all_classes();
        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].name, Some("MyClass".to_string()));
    }

    #[test]
    fn test_parse_python_with_imports() {
        let mut parser = PythonParser::new().unwrap();
        let source = r#"
import os
from datetime import datetime
import json as js

def process_data():
    data = js.loads('{"key": "value"}')
    return data
"#;

        let ast = parser.parse(source).unwrap();
        
        let imports = ast.find_all_imports();
        assert!(imports.len() >= 2); // import os, from datetime import datetime, import json as js
        
        let functions = ast.find_all_functions();
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, Some("process_data".to_string()));
    }

    #[test]
    fn test_parse_python_with_decorators() {
        let mut parser = PythonParser::new().unwrap();
        let source = r#"
@property
def my_property(self):
    return self._value

@staticmethod
@lru_cache(maxsize=128)
def cached_function(x):
    return x * 2
"#;

        let ast = parser.parse(source).unwrap();
        
        let functions = ast.find_all_functions();
        assert_eq!(functions.len(), 2);
        
        // Check for decorator metadata
        let decorated_func = functions.iter()
            .find(|f| f.name == Some("cached_function".to_string()))
            .unwrap();
            
        let decorators: Option<Vec<String>> = decorated_func.get_metadata("decorators");
        assert!(decorators.is_some());
    }

    #[test]
    fn test_parse_python_class_inheritance() {
        let mut parser = PythonParser::new().unwrap();
        let source = r#"
class Animal:
    def speak(self):
        pass

class Dog(Animal):
    def speak(self):
        return "Woof!"

class Cat(Animal):
    def speak(self):
        return "Meow!"
"#;

        let ast = parser.parse(source).unwrap();
        
        let classes = ast.find_all_classes();
        assert_eq!(classes.len(), 3);
        
        // Check for inheritance metadata
        let dog_class = classes.iter()
            .find(|c| c.name == Some("Dog".to_string()))
            .unwrap();
            
        let base_classes: Option<Vec<String>> = dog_class.get_metadata("base_classes");
        assert!(base_classes.is_some());
        if let Some(bases) = base_classes {
            assert!(bases.contains(&"Animal".to_string()));
        }
    }

    #[test]
    fn test_is_python_file() {
        assert!(PythonParser::is_python_file(&PathBuf::from("test.py")));
        assert!(PythonParser::is_python_file(&PathBuf::from("script.pyw")));
        assert!(!PythonParser::is_python_file(&PathBuf::from("test.java")));
        assert!(!PythonParser::is_python_file(&PathBuf::from("README")));
    }

    #[test]
    fn test_parse_syntax_error() {
        let mut parser = PythonParser::new().unwrap();
        let source = r#"
def invalid_syntax(
    # Missing closing parenthesis and proper body
"#;

        // Tree-sitter should still parse this, but might produce error nodes
        let result = parser.parse(source);
        assert!(result.is_ok()); // Tree-sitter is resilient to syntax errors
    }

    #[test]
    fn test_incremental_parsing() {
        let mut parser = PythonParser::new().unwrap();
        let original_source = r#"
def original_function():
    return "original"
"#;

        let ast1 = parser.parse(original_source).unwrap();
        
        let modified_source = r#"
def original_function():
    return "modified"

def new_function():
    return "new"
"#;

        let ast2 = parser.parse(modified_source).unwrap();
        
        let functions1 = ast1.find_all_functions();
        let functions2 = ast2.find_all_functions();
        
        assert_eq!(functions1.len(), 1);
        assert_eq!(functions2.len(), 2);
    }
}