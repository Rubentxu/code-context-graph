# Tree-sitter Integration Guide

## Español | [English](#english)

### Qué es Tree-sitter

Tree-sitter es una biblioteca de parsing creada por GitHub que genera parsers incrementales para lenguajes de programación. A diferencia de los parsers tradicionales, Tree-sitter está diseñado específicamente para editores de código y herramientas de análisis estático.

### Ventajas de Tree-sitter

#### 1. **Parsing Incremental**
- Solo re-parsea las partes del código que han cambiado
- Ideal para editores de código y análisis en tiempo real
- Mantiene el rendimiento constante incluso en archivos grandes

#### 2. **Error Recovery Robusto**
- Continúa parseando incluso cuando encuentra errores de sintaxis
- Genera un AST parcial pero útil para análisis
- Esencial para herramientas de desarrollo que trabajan con código incompleto

#### 3. **Precisión Total**
- Conserva todos los tokens del código fuente original
- Permite reconstruir el código exacto desde el AST
- Mantiene información de whitespace y comentarios

#### 4. **Alto Rendimiento**
- Parsing en tiempo lineal O(n)
- Uso eficiente de memoria
- Optimizado para archivos grandes

### Arquitectura de la Integración

```rust
// Estructura de la integración Tree-sitter
┌─────────────────────────────────────────────────────────────┐
│                  Parser Registry                            │
│  ┌─────────────────────────────────────────────────────┐    │
│  │  Language-specific Tree-sitter Parsers             │    │
│  │  ┌─────────────┬─────────────┬─────────────────┐   │    │
│  │  │tree-sitter- │tree-sitter- │tree-sitter-     │   │    │
│  │  │python       │java         │javascript       │   │    │
│  │  └─────────────┴─────────────┴─────────────────┘   │    │
│  └─────────────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────────┤
│                 AST Conversion Layer                        │
│  Tree-sitter Node -> ASTNode -> SimplifiedAST              │
└─────────────────────────────────────────────────────────────┘
```

### Implementación Detallada

#### 1. Configuración de Parsers por Lenguaje

```rust
// src/language/registry.rs
use tree_sitter::{Language, Parser};

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
        
        // Registrar parsers Tree-sitter
        registry.register_parser(Language::Python, |source| {
            parse_with_tree_sitter(source, tree_sitter_python::language())
        });
        
        registry.register_parser(Language::Java, |source| {
            parse_with_tree_sitter(source, tree_sitter_java::language())
        });
        
        // ... más lenguajes
        
        registry
    }
}
```

#### 2. Conversión de Tree-sitter a AST Abstracto

El proceso de conversión sigue estos pasos:

```rust
// src/ast/node.rs
impl ASTNode {
    pub fn from_tree_sitter_node(
        node: &tree_sitter::Node,
        source: &str,
        language: Language
    ) -> Result<Self> {
        // 1. Mapear tipo de nodo Tree-sitter a ASTNodeType
        let node_type = Self::map_node_type(node.kind(), language);
        
        // 2. Extraer ubicación precisa
        let location = NodeLocation::from_tree_sitter_node(node);
        
        // 3. Extraer nombre si es aplicable
        let name = Self::extract_node_name(node, source);
        
        // 4. Extraer metadata específica del lenguaje
        let metadata = Self::extract_metadata(node, source, language);
        
        // 5. Convertir nodos hijos recursivamente
        let children = node.children(&mut node.walk())
            .map(|child| Self::from_tree_sitter_node(&child, source, language))
            .collect::<Result<Vec<_>>>()?;
        
        Ok(ASTNode {
            node_type,
            name,
            location,
            metadata,
            children,
        })
    }
}
```

#### 3. Mapeo de Tipos de Nodos

Cada lenguaje tiene diferentes nombres para conceptos similares. Nuestro sistema unifica estos tipos:

```rust
fn map_node_type(tree_sitter_kind: &str, language: Language) -> ASTNodeType {
    match (tree_sitter_kind, language) {
        // Declaraciones de clases
        ("class_definition", Language::Python) |
        ("class_declaration", Language::Java) |
        ("class_declaration", Language::JavaScript) |
        ("class_declaration", Language::Kotlin) => ASTNodeType::ClassDeclaration,
        
        // Declaraciones de funciones
        ("function_definition", Language::Python) |
        ("method_declaration", Language::Java) |
        ("function_declaration", Language::JavaScript) |
        ("function_declaration", Language::Kotlin) => ASTNodeType::FunctionDeclaration,
        
        // Interfaces (Java/Kotlin específico)
        ("interface_declaration", Language::Java) |
        ("interface_declaration", Language::Kotlin) => ASTNodeType::InterfaceDeclaration,
        
        // Imports/módulos
        ("import_statement", Language::Python) |
        ("import_declaration", Language::Java) |
        ("import_statement", Language::JavaScript) => ASTNodeType::ImportDeclaration,
        
        // Fallback para nodos no reconocidos
        _ => ASTNodeType::Unknown(tree_sitter_kind.to_string()),
    }
}
```

### Extracción de Metadata Específica

Cada lenguaje tiene características únicas que se capturan como metadata:

#### Python
```rust
fn extract_python_metadata(node: &Node, source: &str) -> HashMap<String, serde_json::Value> {
    let mut metadata = HashMap::new();
    
    match node.kind() {
        "function_definition" => {
            // Decoradores
            if let Some(decorators) = extract_decorators(node, source) {
                metadata.insert("decorators".to_string(), json!(decorators));
            }
            
            // Funciones async
            if is_async_function(node) {
                metadata.insert("is_async".to_string(), json!(true));
            }
        },
        "class_definition" => {
            // Herencia múltiple
            if let Some(bases) = extract_base_classes(node, source) {
                metadata.insert("base_classes".to_string(), json!(bases));
            }
        },
        _ => {}
    }
    
    metadata
}
```

#### Java
```rust
fn extract_java_metadata(node: &Node, source: &str) -> HashMap<String, serde_json::Value> {
    let mut metadata = HashMap::new();
    
    match node.kind() {
        "class_declaration" | "interface_declaration" => {
            // Modificadores de acceso
            let modifiers = extract_modifiers(node, source);
            metadata.insert("modifiers".to_string(), json!(modifiers));
            
            // Generics
            if let Some(type_params) = extract_type_parameters(node, source) {
                metadata.insert("type_parameters".to_string(), json!(type_params));
            }
            
            // Herencia e implementación
            if let Some(extends) = extract_extends_clause(node, source) {
                metadata.insert("extends".to_string(), json!(extends));
            }
            
            if let Some(implements) = extract_implements_clause(node, source) {
                metadata.insert("implements".to_string(), json!(implements));
            }
        },
        _ => {}
    }
    
    metadata
}
```

### Manejo de Errores de Sintaxis

Tree-sitter es robusto ante errores de sintaxis. Nuestro sistema maneja estos casos:

```rust
pub fn parse_with_error_recovery(source: &str, language: Language) -> Result<SimplifiedAST> {
    let mut parser = get_parser_for_language(language)?;
    let tree = parser.parse(source, None).ok_or_else(|| {
        CodeGraphError::ParseError("Failed to parse source code".to_string())
    })?;
    
    let root_node = tree.root_node();
    
    // Verificar si hay errores de sintaxis
    if root_node.has_error() {
        log::warn!("Syntax errors detected, but continuing with partial AST");
        
        // Recopilar información sobre errores para debugging
        let errors = collect_error_nodes(&root_node, source);
        log::debug!("Found {} syntax errors: {:?}", errors.len(), errors);
    }
    
    // Convertir a nuestro AST abstracto incluso con errores
    let ast_root = ASTNode::from_tree_sitter_node(&root_node, source, language)?;
    
    Ok(SimplifiedAST::new(ast_root, language, "parsed"))
}

fn collect_error_nodes(node: &Node, source: &str) -> Vec<String> {
    let mut errors = Vec::new();
    
    if node.is_error() {
        if let Ok(text) = node.utf8_text(source.as_bytes()) {
            errors.push(format!("Error at {}:{} - '{}'", 
                               node.start_position().row + 1,
                               node.start_position().column + 1,
                               text));
        }
    }
    
    // Recursivamente buscar errores en nodos hijos
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        errors.extend(collect_error_nodes(&child, source));
    }
    
    errors
}
```

### Optimizaciones de Rendimiento

#### 1. Pool de Parsers
```rust
impl ParserRegistry {
    fn get_parser(&self, language: Language) -> Result<Parser> {
        let mut pool = self.parser_pool.lock().unwrap();
        
        if let Some(parsers) = pool.get_mut(&language) {
            if let Some(parser) = parsers.pop() {
                return Ok(parser);
            }
        }
        
        // Crear nuevo parser si el pool está vacío
        let mut parser = Parser::new();
        parser.set_language(get_tree_sitter_language(language))?;
        Ok(parser)
    }
    
    fn return_parser(&self, language: Language, parser: Parser) {
        let mut pool = self.parser_pool.lock().unwrap();
        pool.entry(language)
            .or_insert_with(Vec::new)
            .push(parser);
    }
}
```

#### 2. Parsing Incremental
```rust
pub fn parse_incremental(&mut self, 
                        source: &str, 
                        old_tree: Option<&Tree>, 
                        language: Language) -> Result<SimplifiedAST> {
    let mut parser = self.get_parser(language)?;
    
    // Usar árbol anterior para parsing incremental
    let tree = parser.parse(source, old_tree)
        .ok_or_else(|| CodeGraphError::ParseError("Incremental parse failed".to_string()))?;
    
    // Solo convertir nodos que han cambiado
    let ast_root = if let Some(old) = old_tree {
        ASTNode::from_tree_sitter_incremental(&tree.root_node(), old, source, language)?
    } else {
        ASTNode::from_tree_sitter_node(&tree.root_node(), source, language)?
    };
    
    self.return_parser(language, parser);
    
    Ok(SimplifiedAST::new(ast_root, language, "incremental"))
}
```

### Testing de la Integración Tree-sitter

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tree_sitter_python_integration() {
        let source = r#"
class Calculator:
    @staticmethod
    def add(a, b):
        return a + b
        "#;
        
        let registry = ParserRegistry::new();
        let ast = registry.parse(source, Language::Python).unwrap();
        
        // Verificar que Tree-sitter detectó correctamente la estructura
        assert_eq!(ast.root.node_type, ASTNodeType::Program);
        
        let classes = ast.find_all_classes();
        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].name.as_ref().unwrap(), "Calculator");
        
        let functions = ast.find_all_functions();
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name.as_ref().unwrap(), "add");
        
        // Verificar metadata de decoradores
        let decorators: Vec<String> = functions[0].get_metadata("decorators").unwrap();
        assert_eq!(decorators, vec!["@staticmethod"]);
    }
    
    #[test]
    fn test_error_recovery() {
        let malformed_code = r#"
class BrokenClass {
    def method_without_parentheses
        return "this should still be parseable"
}
        "#;
        
        let registry = ParserRegistry::new();
        let result = registry.parse(malformed_code, Language::Python);
        
        // Debe parsear exitosamente incluso con errores
        assert!(result.is_ok());
        
        let ast = result.unwrap();
        // Debe encontrar al menos la clase
        let classes = ast.find_all_classes();
        assert!(!classes.is_empty());
    }
}
```

---

## English

### What is Tree-sitter

Tree-sitter is a parsing library created by GitHub that generates incremental parsers for programming languages. Unlike traditional parsers, Tree-sitter is specifically designed for code editors and static analysis tools.

### Tree-sitter Advantages

#### 1. **Incremental Parsing**
- Only re-parses parts of code that have changed
- Ideal for code editors and real-time analysis
- Maintains consistent performance even with large files

#### 2. **Robust Error Recovery**
- Continues parsing even when it encounters syntax errors
- Generates partial but useful AST for analysis
- Essential for development tools working with incomplete code

#### 3. **Total Precision**
- Preserves all tokens from original source code
- Allows exact code reconstruction from AST
- Maintains whitespace and comment information

#### 4. **High Performance**
- Linear time parsing O(n)
- Efficient memory usage
- Optimized for large files

### Integration Architecture

The integration follows a layered approach where Tree-sitter parsers are wrapped in our abstraction layer, providing a unified interface across all supported languages while preserving the performance and accuracy benefits of Tree-sitter.

### Testing Tree-sitter Integration

The integration is thoroughly tested with comprehensive test suites that verify both correct parsing of valid code and robust error handling for malformed code, ensuring reliability across all supported languages.