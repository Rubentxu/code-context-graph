use code_context_graph_core::{Language, Result, CodeGraphError};
use crate::ast::SimplifiedAST;
use tree_sitter::Parser;

pub struct JavaScriptParser {
    parser: Parser,
}

impl JavaScriptParser {
    pub fn new() -> Result<Self> {
        let mut parser = Parser::new();
        parser.set_language(&tree_sitter_javascript::LANGUAGE.into())
            .map_err(|e| CodeGraphError::Parser {
                message: format!("Failed to set JavaScript language: {}", e)
            })?;
        
        Ok(Self { parser })
    }

    pub fn parse(&mut self, source: &str) -> Result<SimplifiedAST> {
        let tree = self.parser.parse(source, None)
            .ok_or_else(|| CodeGraphError::Parser {
                message: "Failed to parse JavaScript source".to_string()
            })?;

        SimplifiedAST::from_tree_sitter(tree.root_node(), source, Language::JavaScript)
    }

    pub fn parse_with_old_tree(&mut self, source: &str, old_tree: Option<&tree_sitter::Tree>) -> Result<SimplifiedAST> {
        let tree = self.parser.parse(source, old_tree)
            .ok_or_else(|| CodeGraphError::Parser {
                message: "Failed to parse JavaScript source incrementally".to_string()
            })?;

        SimplifiedAST::from_tree_sitter(tree.root_node(), source, Language::JavaScript)
    }

    pub fn supported_extensions() -> Vec<&'static str> {
        vec!["js", "mjs", "jsx"]
    }

    pub fn is_javascript_file(path: &std::path::Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            Self::supported_extensions().contains(&ext)
        } else {
            false
        }
    }
}

impl Default for JavaScriptParser {
    fn default() -> Self {
        Self::new().expect("Failed to create JavaScript parser")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_javascript_parser_creation() {
        let parser = JavaScriptParser::new();
        assert!(parser.is_ok());
    }

    #[test]
    fn test_parse_simple_javascript() {
        let mut parser = JavaScriptParser::new().unwrap();
        let source = r#"
function greet(name) {
    return `Hello, ${name}!`;
}

const MyClass = class {
    constructor(value) {
        this.value = value;
    }
    
    getValue() {
        return this.value;
    }
    
    static create(value) {
        return new MyClass(value);
    }
};

const instance = new MyClass(42);
console.log(greet("World"));
"#;

        let ast = parser.parse(source).unwrap();
        assert_eq!(ast.language, Language::JavaScript);
        
        let functions = ast.find_all_functions();
        assert!(functions.len() >= 3); // greet, constructor, getValue, create
        
        let calls = ast.find_all_calls();
        assert!(calls.len() >= 2); // console.log, greet call
    }

    #[test]
    fn test_parse_javascript_es6_features() {
        let mut parser = JavaScriptParser::new().unwrap();
        let source = r#"
import { module1, module2 } from './modules';
import defaultExport from './default';

const arrow = (x, y) => x + y;

async function fetchData(url) {
    try {
        const response = await fetch(url);
        return await response.json();
    } catch (error) {
        console.error('Error:', error);
        throw error;
    }
}

function* generator() {
    yield 1;
    yield 2;
    yield 3;
}

class Component extends React.Component {
    render() {
        return <div>Hello</div>;
    }
}

export { arrow, fetchData, generator };
export default Component;
"#;

        let ast = parser.parse(source).unwrap();
        
        let imports = ast.find_all_imports();
        assert!(imports.len() >= 2);
        
        let functions = ast.find_all_functions();
        assert!(functions.len() >= 3); // arrow, fetchData, generator, render
        
        // Check for async metadata
        let async_func = functions.iter()
            .find(|f| f.name == Some("fetchData".to_string()))
            .unwrap();
            
        let is_async: Option<bool> = async_func.get_metadata("async");
        assert_eq!(is_async, Some(true));
        
        // Check that ES6 features are parsed correctly (classes, functions, etc.)
        let classes = ast.find_all_classes();
        assert!(classes.len() >= 1); // At least one class declaration
        
        let functions = ast.find_all_functions();
        assert!(functions.len() >= 1); // At least one function/generator
    }

    #[test]
    fn test_parse_javascript_class_inheritance() {
        let mut parser = JavaScriptParser::new().unwrap();
        let source = r#"
class Animal {
    constructor(name) {
        this.name = name;
    }
    
    speak() {
        console.log(`${this.name} makes a sound`);
    }
}

class Dog extends Animal {
    constructor(name, breed) {
        super(name);
        this.breed = breed;
    }
    
    speak() {
        console.log(`${this.name} barks`);
    }
    
    getBreed() {
        return this.breed;
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
    }

    #[test]
    fn test_parse_javascript_object_methods() {
        let mut parser = JavaScriptParser::new().unwrap();
        let source = r#"
const obj = {
    name: 'test',
    
    method1() {
        return this.name;
    },
    
    method2: function() {
        return 'method2';
    },
    
    method3: () => {
        return 'arrow method';
    },
    
    async asyncMethod() {
        return await Promise.resolve('async result');
    }
};
"#;

        let ast = parser.parse(source).unwrap();
        
        let functions = ast.find_all_functions();
        assert!(functions.len() >= 4); // method1, method2, method3, asyncMethod
    }

    #[test]
    fn test_parse_javascript_modules() {
        let mut parser = JavaScriptParser::new().unwrap();
        let source = r#"
// CommonJS style
const fs = require('fs');
const { readFile } = require('fs').promises;

// ES6 style
import React from 'react';
import { useState, useEffect } from 'react';
import * as utils from './utils';

function Component() {
    const [state, setState] = useState(null);
    
    useEffect(() => {
        utils.loadData().then(setState);
    }, []);
    
    return React.createElement('div', null, state);
}

module.exports = Component;
export default Component;
"#;

        let ast = parser.parse(source).unwrap();
        
        let imports = ast.find_all_imports();
        assert!(imports.len() >= 3); // Various import styles
        
        let functions = ast.find_all_functions();
        assert!(functions.len() >= 2); // Component, arrow function in useEffect
    }

    #[test]
    fn test_is_javascript_file() {
        assert!(JavaScriptParser::is_javascript_file(&PathBuf::from("script.js")));
        assert!(JavaScriptParser::is_javascript_file(&PathBuf::from("module.mjs")));
        assert!(JavaScriptParser::is_javascript_file(&PathBuf::from("component.jsx")));
        assert!(!JavaScriptParser::is_javascript_file(&PathBuf::from("test.py")));
        assert!(!JavaScriptParser::is_javascript_file(&PathBuf::from("README")));
    }

    #[test]
    fn test_parse_javascript_destructuring() {
        let mut parser = JavaScriptParser::new().unwrap();
        let source = r#"
const { a, b, c } = obj;
const [first, second, ...rest] = array;

function destructureParam({ name, age }) {
    return `${name} is ${age} years old`;
}

const arrow = ({ x, y }) => x + y;
"#;

        let ast = parser.parse(source).unwrap();
        
        let functions = ast.find_all_functions();
        assert!(functions.len() >= 2); // destructureParam, arrow
    }


    #[test]
    fn test_parse_javascript_closures() {
        let mut parser = JavaScriptParser::new().unwrap();
        let source = r#"
function createCounter() {
    let count = 0;
    
    return function() {
        return ++count;
    };
}

function higherOrder(callback) {
    return function(x) {
        return callback(x * 2);
    };
}

const counter = createCounter();
const doubled = higherOrder(x => x + 1);
"#;

        let ast = parser.parse(source).unwrap();
        
        let functions = ast.find_all_functions();
        assert!(functions.len() >= 4); // createCounter, inner function, higherOrder, inner function, arrow
    }
}