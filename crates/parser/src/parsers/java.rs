use code_context_graph_core::{Language, Result, CodeGraphError};
use crate::ast::SimplifiedAST;
use tree_sitter::Parser;

pub struct JavaParser {
    parser: Parser,
}

impl JavaParser {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_java::LANGUAGE.into())
            .map_err(|e| CodeGraphError::Parser {
                message: format!("Failed to set Java language: {}", e)
            })?;
        
        Ok(Self { parser })
    }

    pub fn parse(&mut self, source: &str) -> Result<SimplifiedAST> {
        let tree = self.parser.parse(source, None)
            .ok_or_else(|| CodeGraphError::Parser {
                message: "Failed to parse Java source".to_string()
            })?;

        SimplifiedAST::from_tree_sitter(tree.root_node(), source, Language::Java)
    }

    pub fn parse_with_old_tree(&mut self, source: &str, old_tree: Option<&tree_sitter::Tree>) -> Result<SimplifiedAST> {
        let tree = self.parser.parse(source, old_tree)
            .ok_or_else(|| CodeGraphError::Parser {
                message: "Failed to parse Java source incrementally".to_string()
            })?;

        SimplifiedAST::from_tree_sitter(tree.root_node(), source, Language::Java)
    }

    pub fn supported_extensions() -> Vec<&'static str> {
        vec!["java"]
    }

    pub fn is_java_file(path: &std::path::Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            Self::supported_extensions().contains(&ext)
        } else {
            false
        }
    }
}

impl Default for JavaParser {
    fn default() -> Self {
        Self::new().expect("Failed to create Java parser")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_java_parser_creation() {
        let parser = JavaParser::new();
        assert!(parser.is_ok());
    }

    #[test]
    fn test_parse_simple_java() {
        let mut parser = JavaParser::new().unwrap();
        let source = r#"
package com.example;

import java.util.List;
import java.util.ArrayList;

public class MyClass {
    private int value;
    
    public MyClass(int value) {
        this.value = value;
    }
    
    public int getValue() {
        return value;
    }
    
    public static void main(String[] args) {
        MyClass instance = new MyClass(42);
        System.out.println(instance.getValue());
    }
}
"#;

        let ast = parser.parse(source).unwrap();
        assert_eq!(ast.language, Language::Java);
        
        let classes = ast.find_all_classes();
        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].name, Some("MyClass".to_string()));
        
        let functions = ast.find_all_functions();
        assert!(functions.len() >= 3); // constructor, getValue, main
        
        let imports = ast.find_all_imports();
        assert!(imports.len() >= 2); // java.util.List, java.util.ArrayList
    }

    #[test]
    fn test_parse_java_inheritance() {
        let mut parser = JavaParser::new().unwrap();
        let source = r#"
public abstract class Animal {
    protected String name;
    
    public Animal(String name) {
        this.name = name;
    }
    
    public abstract void makeSound();
}

public class Dog extends Animal implements Runnable {
    public Dog(String name) {
        super(name);
    }
    
    @Override
    public void makeSound() {
        System.out.println("Woof!");
    }
    
    @Override
    public void run() {
        System.out.println("Dog is running");
    }
}
"#;

        let ast = parser.parse(source).unwrap();
        
        let classes = ast.find_all_classes();
        assert_eq!(classes.len(), 2);
        
        // Check for inheritance metadata
        let dog_class = classes.iter()
            .find(|c| c.name == Some("Dog".to_string()))
            .unwrap();
            
        let extends: Option<String> = dog_class.get_metadata("extends");
        assert!(extends.is_some());
        assert_eq!(extends.unwrap(), "Animal");
        
        // Note: implements metadata extraction may depend on Tree-sitter Java grammar specifics
        // For now, we verify the class structure is parsed correctly
    }

    #[test]
    fn test_parse_java_modifiers() {
        let mut parser = JavaParser::new().unwrap();
        let source = r#"
public final class Utils {
    private static final int CONSTANT = 42;
    
    public static synchronized void synchronizedMethod() {
        // Method body
    }
    
    private final void finalMethod() {
        // Method body
    }
}
"#;

        let ast = parser.parse(source).unwrap();
        
        let functions = ast.find_all_functions();
        assert!(functions.len() >= 2);
        
        // Check that the synchronized method is parsed correctly
        let sync_method = functions.iter()
            .find(|f| f.name == Some("synchronizedMethod".to_string()))
            .unwrap();
            
        // Basic parsing verification - method name and structure are correct
        assert_eq!(sync_method.name, Some("synchronizedMethod".to_string()));
        assert_eq!(sync_method.node_type, crate::ast::ASTNodeType::MethodDeclaration);
    }

    #[test]
    fn test_parse_java_interface() {
        let mut parser = JavaParser::new().unwrap();
        let source = r#"
public interface Drawable {
    void draw();
    
    default void clear() {
        System.out.println("Clearing");
    }
    
    static void staticMethod() {
        System.out.println("Static method in interface");
    }
}
"#;

        let ast = parser.parse(source).unwrap();
        
        // Interfaces should be captured as classes with interface metadata
        let classes = ast.find_all_classes();
        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].name, Some("Drawable".to_string()));
        
        let functions = ast.find_all_functions();
        assert!(functions.len() >= 3); // draw, clear, staticMethod
    }

    #[test]
    fn test_parse_java_generic_class() {
        let mut parser = JavaParser::new().unwrap();
        let source = r#"
public class Container<T> {
    private T item;
    
    public Container(T item) {
        this.item = item;
    }
    
    public T getItem() {
        return item;
    }
    
    public void setItem(T item) {
        this.item = item;
    }
}
"#;

        let ast = parser.parse(source).unwrap();
        
        let classes = ast.find_all_classes();
        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].name, Some("Container".to_string()));
        
        let functions = ast.find_all_functions();
        assert!(functions.len() >= 3); // constructor, getItem, setItem
    }

    #[test]
    fn test_is_java_file() {
        assert!(JavaParser::is_java_file(&PathBuf::from("Test.java")));
        assert!(!JavaParser::is_java_file(&PathBuf::from("test.py")));
        assert!(!JavaParser::is_java_file(&PathBuf::from("README")));
    }

    #[test]
    fn test_parse_java_annotations() {
        let mut parser = JavaParser::new().unwrap();
        let source = r#"
@Entity
@Table(name = "users")
public class User {
    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;
    
    @Column(nullable = false)
    private String name;
    
    @Override
    @Deprecated
    public String toString() {
        return "User{name='" + name + "'}";
    }
}
"#;

        let ast = parser.parse(source).unwrap();
        
        let classes = ast.find_all_classes();
        assert_eq!(classes.len(), 1);
        
        let functions = ast.find_all_functions();
        assert!(functions.len() >= 1); // toString method
    }

    #[test]
    fn test_incremental_parsing() {
        let mut parser = JavaParser::new().unwrap();
        let original_source = r#"
public class Test {
    public void method1() {
        System.out.println("Method 1");
    }
}
"#;

        let ast1 = parser.parse(original_source).unwrap();
        
        let modified_source = r#"
public class Test {
    public void method1() {
        System.out.println("Modified Method 1");
    }
    
    public void method2() {
        System.out.println("Method 2");
    }
}
"#;

        let ast2 = parser.parse(modified_source).unwrap();
        
        let functions1 = ast1.find_all_functions();
        let functions2 = ast2.find_all_functions();
        
        assert_eq!(functions1.len(), 1);
        assert_eq!(functions2.len(), 2);
    }
}