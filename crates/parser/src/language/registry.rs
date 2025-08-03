use code_context_graph_core::{Language, Result, CodeGraphError};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tree_sitter::{Parser, Node};
use crate::ast::SimplifiedAST;

pub type ParseResult = Result<SimplifiedAST>;
pub type ParserFunction = Box<dyn Fn(&str) -> ParseResult + Send + Sync>;

pub struct ParserRegistry {
    parsers: HashMap<Language, ParserFunction>,
    parser_pool: Arc<Mutex<HashMap<Language, Vec<Parser>>>>,
}

impl ParserRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            parsers: HashMap::new(),
            parser_pool: Arc::new(Mutex::new(HashMap::new())),
        };
        
        registry.register_builtin_parsers();
        registry
    }

    fn register_builtin_parsers(&mut self) {
        // Python parser
        self.parsers.insert(
            Language::Python,
            Box::new(|source| {
                let mut parser = Parser::new();
                parser.set_language(&tree_sitter_python::LANGUAGE.into())
                    .map_err(|e| CodeGraphError::Parser { 
                        message: format!("Failed to set Python language: {}", e) 
                    })?;
                
                let tree = parser.parse(source, None)
                    .ok_or_else(|| CodeGraphError::Parser { 
                        message: "Failed to parse Python source".to_string() 
                    })?;
                
                SimplifiedAST::from_tree_sitter(tree.root_node(), source, Language::Python)
            })
        );

        // Java parser
        self.parsers.insert(
            Language::Java,
            Box::new(|source| {
                let mut parser = Parser::new();
                parser.set_language(&tree_sitter_java::LANGUAGE.into())
                    .map_err(|e| CodeGraphError::Parser { 
                        message: format!("Failed to set Java language: {}", e) 
                    })?;
                
                let tree = parser.parse(source, None)
                    .ok_or_else(|| CodeGraphError::Parser { 
                        message: "Failed to parse Java source".to_string() 
                    })?;
                
                SimplifiedAST::from_tree_sitter(tree.root_node(), source, Language::Java)
            })
        );

        // JavaScript parser
        self.parsers.insert(
            Language::JavaScript,
            Box::new(|source| {
                let mut parser = Parser::new();
                parser.set_language(&tree_sitter_javascript::LANGUAGE.into())
                    .map_err(|e| CodeGraphError::Parser { 
                        message: format!("Failed to set JavaScript language: {}", e) 
                    })?;
                
                let tree = parser.parse(source, None)
                    .ok_or_else(|| CodeGraphError::Parser { 
                        message: "Failed to parse JavaScript source".to_string() 
                    })?;
                
                SimplifiedAST::from_tree_sitter(tree.root_node(), source, Language::JavaScript)
            })
        );

        // Kotlin parser
        self.parsers.insert(
            Language::Kotlin,
            Box::new(|source| {
                let mut parser = Parser::new();
                parser.set_language(&tree_sitter_kotlin_ng::LANGUAGE.into())
                    .map_err(|e| CodeGraphError::Parser { 
                        message: format!("Failed to set Kotlin language: {}", e) 
                    })?;
                
                let tree = parser.parse(source, None)
                    .ok_or_else(|| CodeGraphError::Parser { 
                        message: "Failed to parse Kotlin source".to_string() 
                    })?;
                
                SimplifiedAST::from_tree_sitter(tree.root_node(), source, Language::Kotlin)
            })
        );
    }

    pub fn parse(&self, source: &str, language: Language) -> ParseResult {
        let parser_fn = self.parsers.get(&language)
            .ok_or_else(|| CodeGraphError::Parser {
                message: format!("No parser registered for language: {:?}", language)
            })?;
        
        parser_fn(source)
    }

    pub fn supports_language(&self, language: &Language) -> bool {
        self.parsers.contains_key(language)
    }

    pub fn supported_languages(&self) -> Vec<Language> {
        self.parsers.keys().cloned().collect()
    }

    pub fn register_custom_parser(&mut self, language: Language, parser: ParserFunction) {
        self.parsers.insert(language, parser);
    }
}

impl Default for ParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = ParserRegistry::new();
        
        assert!(registry.supports_language(&Language::Python));
        assert!(registry.supports_language(&Language::Java));
        assert!(registry.supports_language(&Language::JavaScript));
        assert!(registry.supports_language(&Language::Kotlin));
        assert!(!registry.supports_language(&Language::Unknown));
    }

    #[test]
    fn test_supported_languages() {
        let registry = ParserRegistry::new();
        let languages = registry.supported_languages();
        
        assert!(languages.contains(&Language::Python));
        assert!(languages.contains(&Language::Java));
        assert!(languages.contains(&Language::JavaScript));
        assert!(languages.contains(&Language::Kotlin));
    }

    #[test]
    fn test_unsupported_language() {
        let registry = ParserRegistry::new();
        let result = registry.parse("some code", Language::Unknown);
        
        assert!(result.is_err());
    }
}