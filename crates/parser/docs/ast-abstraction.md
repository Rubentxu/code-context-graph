# AST Abstraction Architecture

## Español | [English](#english)

### Visión General

La capa de abstracción AST del Code Context Graph Parser proporciona una representación unificada y simplificada de los Árboles de Sintaxis Abstracta (AST) generados por Tree-sitter. Esta abstracción permite trabajar con código de diferentes lenguajes de manera consistente, ocultando las diferencias específicas de cada parser.

### Arquitectura de la Abstracción

```
┌─────────────────────────────────────────────────────────┐
│                   Tree-sitter Layer                     │
│  ┌─────────────┬─────────────┬─────────────────────┐   │
│  │ Python AST  │  Java AST   │ JavaScript/Kotlin   │   │
│  │  Concrete   │  Concrete   │    AST Concrete     │   │
│  └─────────────┴─────────────┴─────────────────────┘   │
└─────────────────────────────────────────────────────────┘
                            ↓ Abstraction Layer
┌─────────────────────────────────────────────────────────┐
│                SimplifiedAST Unified Layer              │
│  ┌─────────────────────────────────────────────────┐   │
│  │  ASTNode (Abstract)  │  NodeLocation  │ Metadata│   │
│  │  - node_type         │  - start_line  │ - k/v   │   │
│  │  - name              │  - end_line    │ - typed │   │
│  │  - children          │  - start_byte  │ - rich  │   │
│  │  - location          │  - end_byte    │         │   │
│  │  - metadata          │                │         │   │
│  └─────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### Componentes Principales

#### 1. SimplifiedAST (`src/ast/simplified.rs`)

La estructura principal que encapsula todo el árbol sintáctico:

```rust
pub struct SimplifiedAST {
    pub root: ASTNode,
    pub language: Language,
    pub source_name: String,
}

impl SimplifiedAST {
    /// Crear nueva instancia con nodo raíz
    pub fn new(root: ASTNode, language: Language, source_name: &str) -> Self {
        Self {
            root,
            language,
            source_name: source_name.to_string(),
        }
    }
    
    /// Encontrar todas las clases en el AST
    pub fn find_all_classes(&self) -> Vec<&ASTNode> {
        let mut result = self.root.find_children_by_type(&ASTNodeType::ClassDeclaration);
        result.extend(self.root.find_children_by_type(&ASTNodeType::InterfaceDeclaration));
        result
    }
    
    /// Encontrar todas las funciones/métodos
    pub fn find_all_functions(&self) -> Vec<&ASTNode> {
        let mut result = self.root.find_children_by_type(&ASTNodeType::FunctionDeclaration);
        result.extend(self.root.find_children_by_type(&ASTNodeType::MethodDeclaration));
        result
    }
    
    /// Buscar nodos por nombre
    pub fn find_nodes_by_name(&self, name: &str) -> Vec<&ASTNode> {
        self.root.find_nodes_by_name(name)
    }
}
```

#### 2. ASTNode (`src/ast/node.rs`)

El componente fundamental que representa cada nodo del árbol:

```rust
pub struct ASTNode {
    pub node_type: ASTNodeType,
    pub name: Option<String>,
    pub location: NodeLocation,
    pub metadata: HashMap<String, serde_json::Value>,
    pub children: Vec<ASTNode>,
}

impl ASTNode {
    /// Crear nuevo nodo
    pub fn new(
        node_type: ASTNodeType,
        name: Option<String>,
        location: NodeLocation,
    ) -> Self {
        Self {
            node_type,
            name,
            location,
            metadata: HashMap::new(),
            children: Vec::new(),
        }
    }
    
    /// Añadir metadata tipada
    pub fn add_metadata<T: serde::Serialize>(&mut self, key: &str, value: T) {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.metadata.insert(key.to_string(), json_value);
        }
    }
    
    /// Obtener metadata tipada
    pub fn get_metadata<T>(&self, key: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.metadata.get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }
    
    /// Buscar hijos por tipo
    pub fn find_children_by_type(&self, target_type: &ASTNodeType) -> Vec<&ASTNode> {
        let mut result = Vec::new();
        self.collect_children_by_type(target_type, &mut result);
        result
    }
    
    /// Búsqueda recursiva por tipo
    fn collect_children_by_type(&self, target_type: &ASTNodeType, result: &mut Vec<&ASTNode>) {
        if &self.node_type == target_type {
            result.push(self);
        }
        for child in &self.children {
            child.collect_children_by_type(target_type, result);
        }
    }
}
```

#### 3. ASTNodeType (`src/ast/node.rs`)

Enumeración que unifica los tipos de nodos de todos los lenguajes:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ASTNodeType {
    // Nodos estructurales
    Program,
    Module,
    Block,
    
    // Declaraciones
    ClassDeclaration,
    InterfaceDeclaration,
    EnumDeclaration,
    FunctionDeclaration,
    MethodDeclaration,
    VariableDeclaration,
    FieldDeclaration,
    ParameterDeclaration,
    
    // Imports y módulos
    ImportDeclaration,
    ExportDeclaration,
    FromImport,
    
    // Expresiones
    CallExpression,
    AssignmentExpression,
    BinaryExpression,
    UnaryExpression,
    LiteralExpression,
    IdentifierExpression,
    
    // Statements
    IfStatement,
    ForStatement,
    WhileStatement,
    ReturnStatement,
    BreakStatement,
    ContinueStatement,
    
    // Especiales
    Comment,
    Decorator,
    Annotation,
    
    // Fallback para nodos no reconocidos
    Unknown(String),
}
```

#### 4. NodeLocation (`src/ast/node.rs`)

Información precisa de ubicación en el código fuente:

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeLocation {
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub start_byte: u32,
    pub end_byte: u32,
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
    
    /// Verificar si esta ubicación contiene otra
    pub fn contains(&self, other: &NodeLocation) -> bool {
        self.start_byte <= other.start_byte && self.end_byte >= other.end_byte
    }
    
    /// Verificar si dos ubicaciones se solapan
    pub fn overlaps(&self, other: &NodeLocation) -> bool {
        self.start_byte < other.end_byte && other.start_byte < self.end_byte
    }
    
    /// Obtener el rango de líneas
    pub fn line_range(&self) -> std::ops::RangeInclusive<u32> {
        self.start_line..=self.end_line
    }
}
```

### Conversión desde Tree-sitter

El proceso de conversión es el corazón de la abstracción:

```rust
impl ASTNode {
    pub fn from_tree_sitter_node(
        node: &tree_sitter::Node,
        source: &str,
        language: Language,
    ) -> Result<Self> {
        // 1. Mapear tipo de nodo específico del lenguaje a tipo abstracto
        let node_type = Self::map_tree_sitter_kind(node.kind(), language);
        
        // 2. Extraer nombre del nodo si es aplicable
        let name = Self::extract_node_name(node, source);
        
        // 3. Crear ubicación precisa
        let location = NodeLocation::from_tree_sitter_node(node);
        
        // 4. Crear nodo base
        let mut ast_node = ASTNode::new(node_type, name, location);
        
        // 5. Extraer metadata específica del lenguaje y tipo de nodo
        ast_node.metadata = Self::extract_metadata(node, source, language);
        
        // 6. Convertir nodos hijos recursivamente
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if Self::should_include_child(&child, language) {
                let child_node = Self::from_tree_sitter_node(&child, source, language)?;
                ast_node.children.push(child_node);
            }
        }
        
        Ok(ast_node)
    }
}
```

### Mapeo de Tipos Multi-lenguaje

Uno de los aspectos más importantes es el mapeo consistente de tipos:

```rust
fn map_tree_sitter_kind(kind: &str, language: Language) -> ASTNodeType {
    match (kind, language) {
        // Clases - diferentes nombres, mismo concepto
        ("class_definition", Language::Python) |
        ("class_declaration", Language::Java) |
        ("class_declaration", Language::JavaScript) |
        ("class_declaration", Language::Kotlin) |
        ("object_declaration", Language::Kotlin) => ASTNodeType::ClassDeclaration,
        
        // Funciones - variaciones por lenguaje
        ("function_definition", Language::Python) |
        ("function_declaration", _) |
        ("function", Language::JavaScript) |
        ("arrow_function", Language::JavaScript) => ASTNodeType::FunctionDeclaration,
        
        // Métodos - específicos de clases
        ("method_declaration", Language::Java) |
        ("method_declaration", Language::Kotlin) |
        ("function_definition", Language::Python) => {
            // Lógica adicional para determinar si es método o función
            ASTNodeType::MethodDeclaration
        },
        
        // Interfaces - Java/Kotlin específico
        ("interface_declaration", Language::Java) |
        ("interface_declaration", Language::Kotlin) => ASTNodeType::InterfaceDeclaration,
        
        // Imports - diferentes sintaxis
        ("import_statement", Language::Python) |
        ("import_declaration", Language::Java) |
        ("import_statement", Language::JavaScript) => ASTNodeType::ImportDeclaration,
        
        // Decoradores/Anotaciones
        ("decorator", Language::Python) => ASTNodeType::Decorator,
        ("annotation", Language::Java) => ASTNodeType::Annotation,
        
        // Nodo raíz del programa
        ("module", Language::Python) |
        ("program", _) => ASTNodeType::Program,
        
        // Fallback para tipos no reconocidos
        _ => ASTNodeType::Unknown(kind.to_string()),
    }
}
```

### Extracción de Metadata Rica

La metadata captura información semántica específica del lenguaje:

```rust
fn extract_metadata(
    node: &tree_sitter::Node,
    source: &str,
    language: Language,
) -> HashMap<String, serde_json::Value> {
    let mut metadata = HashMap::new();
    
    match language {
        Language::Python => extract_python_metadata(node, source, &mut metadata),
        Language::Java => extract_java_metadata(node, source, &mut metadata),
        Language::JavaScript => extract_javascript_metadata(node, source, &mut metadata),
        Language::Kotlin => extract_kotlin_metadata(node, source, &mut metadata),
        _ => {}
    }
    
    metadata
}

fn extract_python_metadata(
    node: &tree_sitter::Node,
    source: &str,
    metadata: &mut HashMap<String, serde_json::Value>,
) {
    match node.kind() {
        "function_definition" => {
            // Decoradores
            let decorators = extract_python_decorators(node, source);
            if !decorators.is_empty() {
                metadata.insert("decorators".to_string(), json!(decorators));
            }
            
            // Función async
            if is_async_function(node) {
                metadata.insert("is_async".to_string(), json!(true));
            }
            
            // Parámetros con tipos
            if let Some(type_hints) = extract_type_hints(node, source) {
                metadata.insert("type_hints".to_string(), json!(type_hints));
            }
        },
        
        "class_definition" => {
            // Clases base para herencia
            let base_classes = extract_base_classes(node, source);
            if !base_classes.is_empty() {
                metadata.insert("base_classes".to_string(), json!(base_classes));
            }
        },
        
        _ => {}
    }
}
```

### Utilidades de Búsqueda y Análisis

La abstracción proporciona métodos de búsqueda potentes:

```rust
impl SimplifiedAST {
    /// Buscar entidades por patrón
    pub fn find_entities_matching<F>(&self, predicate: F) -> Vec<&ASTNode>
    where
        F: Fn(&ASTNode) -> bool,
    {
        let mut results = Vec::new();
        self.root.collect_matching_nodes(&predicate, &mut results);
        results
    }
    
    /// Encontrar todas las definiciones de variables/campos
    pub fn find_all_variables(&self) -> Vec<&ASTNode> {
        self.find_entities_matching(|node| {
            matches!(node.node_type, 
                    ASTNodeType::VariableDeclaration | 
                    ASTNodeType::FieldDeclaration |
                    ASTNodeType::ParameterDeclaration)
        })
    }
    
    /// Encontrar imports/exports
    pub fn find_all_imports(&self) -> Vec<&ASTNode> {
        let mut result = self.root.find_children_by_type(&ASTNodeType::ImportDeclaration);
        result.extend(self.root.find_children_by_type(&ASTNodeType::FromImport));
        result
    }
    
    /// Analizar complejidad del código
    pub fn calculate_complexity(&self) -> CodeComplexity {
        let mut complexity = CodeComplexity::new();
        
        complexity.total_nodes = self.count_all_nodes();
        complexity.total_classes = self.find_all_classes().len();
        complexity.total_functions = self.find_all_functions().len();
        complexity.max_depth = self.calculate_max_depth();
        
        complexity
    }
    
    /// Extraer estadísticas del AST
    pub fn get_statistics(&self) -> ASTStatistics {
        ASTStatistics {
            language: self.language,
            total_nodes: self.count_all_nodes(),
            node_type_distribution: self.get_node_type_distribution(),
            average_children_per_node: self.calculate_average_children(),
            max_depth: self.calculate_max_depth(),
            lines_of_code: self.calculate_lines_of_code(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeComplexity {
    pub total_nodes: usize,
    pub total_classes: usize,
    pub total_functions: usize,
    pub max_depth: usize,
    pub cyclomatic_complexity: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ASTStatistics {
    pub language: Language,
    pub total_nodes: usize,
    pub node_type_distribution: HashMap<String, usize>,
    pub average_children_per_node: f64,
    pub max_depth: usize,
    pub lines_of_code: u32,
}
```

### Serialización y Persistencia

La abstracción es completamente serializable:

```rust
impl SimplifiedAST {
    /// Serializar a JSON
    pub fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
    
    /// Deserializar desde JSON
    pub fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }
    
    /// Guardar en archivo
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let json = self.to_json()?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    /// Cargar desde archivo
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let json = std::fs::read_to_string(path)?;
        Self::from_json(&json)
    }
}
```

### Testing de la Abstracción

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ast_abstraction_consistency() {
        // Mismo concepto en diferentes lenguajes
        let python_code = "class Calculator:\n    def add(self, a, b):\n        return a + b";
        let java_code = "public class Calculator {\n    public int add(int a, int b) {\n        return a + b;\n    }\n}";
        
        let registry = ParserRegistry::new();
        let python_ast = registry.parse(python_code, Language::Python).unwrap();
        let java_ast = registry.parse(java_code, Language::Java).unwrap();
        
        // Ambos deben tener 1 clase y 1 función
        assert_eq!(python_ast.find_all_classes().len(), 1);
        assert_eq!(java_ast.find_all_classes().len(), 1);
        
        assert_eq!(python_ast.find_all_functions().len(), 1);
        assert_eq!(java_ast.find_all_functions().len(), 1);
        
        // Los nombres deben ser consistentes
        assert_eq!(python_ast.find_all_classes()[0].name.as_ref().unwrap(), "Calculator");
        assert_eq!(java_ast.find_all_classes()[0].name.as_ref().unwrap(), "Calculator");
    }
    
    #[test]
    fn test_metadata_extraction() {
        let decorated_python = r#"
@staticmethod
@property
def method():
    pass
        "#;
        
        let ast = ParserRegistry::new().parse(decorated_python, Language::Python).unwrap();
        let functions = ast.find_all_functions();
        
        assert_eq!(functions.len(), 1);
        
        let decorators: Vec<String> = functions[0].get_metadata("decorators").unwrap();
        assert_eq!(decorators.len(), 2);
        assert!(decorators.contains(&"@staticmethod".to_string()));
        assert!(decorators.contains(&"@property".to_string()));
    }
}
```

---

## English

### Overview

The AST abstraction layer of the Code Context Graph Parser provides a unified and simplified representation of Abstract Syntax Trees (AST) generated by Tree-sitter. This abstraction allows working with code from different languages consistently, hiding language-specific parser differences.

### Architecture Benefits

1. **Language Unification**: Same interface for all supported languages
2. **Simplified Navigation**: Easy traversal and searching methods
3. **Rich Metadata**: Language-specific semantic information preserved
4. **Serializable**: Full JSON serialization support for persistence
5. **Extensible**: Easy to add new node types and metadata
6. **Type Safety**: Strong typing with Rust's type system
7. **Performance**: Efficient tree operations with minimal overhead

### Testing and Quality

The AST abstraction is thoroughly tested with comprehensive test suites that verify consistency across languages, metadata extraction accuracy, and serialization roundtrip integrity.