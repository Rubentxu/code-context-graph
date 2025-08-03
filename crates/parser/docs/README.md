# Code Context Graph Parser

## Español | [English](#english)

### Resumen

El módulo **Code Context Graph Parser** es un analizador sintáctico robusto y multi-lenguaje construido sobre Tree-sitter que convierte código fuente en Árboles de Sintaxis Abstracta (AST) simplificados y extraíbles. Diseñado para análisis de código a gran escala, refactorización automática y generación de grafos de contexto.

### Características Principales

- **🌍 Multi-lenguaje**: Soporte nativo para Python, Java, JavaScript y Kotlin
- **🚀 Alto Rendimiento**: Parsing incremental y caching inteligente
- **🏗️ Arquitectura Abstracta**: Capa de abstracción que unifica diferentes sintaxis
- **🔍 Visitor Pattern**: Sistema extensible para análisis y extracción de datos
- **⚡ Tree-sitter**: Basado en la biblioteca de parsing más rápida y precisa
- **📊 Metadata Rica**: Extracción automática de información semántica
- **🧪 Testing Completo**: 164+ tests con 100% de coverage crítico

### Arquitectura del Sistema

```
┌─────────────────────────────────────────────────────────────────┐
│                    Code Context Graph Parser                     │
├─────────────────────────────────────────────────────────────────┤
│  Language Detection  │  Parser Registry  │  Incremental Cache   │
├─────────────────────────────────────────────────────────────────┤
│           Tree-sitter Integration Layer                         │
│  ┌─────────────┬─────────────┬─────────────┬─────────────┐      │
│  │   Python    │    Java     │ JavaScript  │   Kotlin    │      │
│  │   Parser    │   Parser    │   Parser    │   Parser    │      │
│  └─────────────┴─────────────┴─────────────┴─────────────┘      │
├─────────────────────────────────────────────────────────────────┤
│                    AST Abstraction Layer                        │
│  ┌─────────────────────────────────────────────────────────┐    │
│  │  SimplifiedAST  │  ASTNode  │  NodeLocation  │ Metadata │    │
│  └─────────────────────────────────────────────────────────┘    │
├─────────────────────────────────────────────────────────────────┤
│                     Visitor Pattern System                      │
│  ┌─────────────────┬─────────────────┬─────────────────────┐    │
│  │ EntityExtractor │ RelationExtractor│ MetadataCollector │    │
│  └─────────────────┴─────────────────┴─────────────────────┘    │
└─────────────────────────────────────────────────────────────────┘
```

### Uso Básico

```rust
use code_context_graph_parser::{ParserRegistry, Language};

// Inicializar el registry de parsers
let registry = ParserRegistry::new();

// Parsear código fuente
let source_code = r#"
class Calculator:
    def add(self, a, b):
        return a + b
"#;

let ast = registry.parse(source_code, Language::Python)?;

// Extraer información
let classes = ast.find_all_classes();
let functions = ast.find_all_functions();

println!("Encontradas {} clases y {} funciones", 
         classes.len(), functions.len());
```

### Lenguajes Soportados

| Lenguaje   | Estado | Características Soportadas |
|------------|--------|-----------------------------|
| **Python** | ✅ | Clases, funciones, decoradores, imports, async/await |
| **Java** | ✅ | Clases, interfaces, enums, generics, annotations |
| **JavaScript** | ✅ | Clases ES6, funciones arrow, destructuring, modules |
| **Kotlin** | ✅ | Data classes, objetos, corrutinas, extensiones |

### Componentes del Sistema

#### 1. Language Detection (`src/language/detector.rs`)
- Detecta automáticamente el lenguaje basado en extensión y contenido
- Heurísticas inteligentes para casos ambiguos
- Soporte para patrones específicos de cada lenguaje

#### 2. Parser Registry (`src/language/registry.rs`)
- Gestión centralizada de parsers Tree-sitter
- Pool de parsers para rendimiento óptimo
- Interfaz unificada para todos los lenguajes

#### 3. AST Abstraction (`src/ast/`)
- **SimplifiedAST**: Representación simplificada y uniforme
- **ASTNode**: Nodos con metadata y ubicación precisa
- **NodeLocation**: Información de posición en código fuente

#### 4. Visitor Pattern (`src/visitor/`)
- **EntityExtractor**: Extrae clases, funciones, variables
- **RelationExtractor**: Encuentra relaciones entre entidades
- **MetadataCollector**: Recopila información semántica

#### 5. Incremental Parsing (`src/incremental/`)
- Cache inteligente basado en hashes de contenido
- Parsing solo de archivos modificados
- Optimización para proyectos grandes

### Tree-sitter Integration

Tree-sitter es un parser incremental que genera ASTs concretos con las siguientes ventajas:

#### Ventajas de Tree-sitter
- **Error Recovery**: Continúa parseando incluso con errores de sintaxis
- **Incremental**: Re-parsea solo las partes modificadas
- **Precisión**: Mantiene toda la información del código fuente
- **Velocidad**: Parsing en tiempo lineal O(n)
- **Streaming**: Procesa archivos grandes sin cargar todo en memoria

#### Integración en el Proyecto
```rust
// Tree-sitter grammar específico por lenguaje
use tree_sitter_python::language as python_language;
use tree_sitter_java::language as java_language;
use tree_sitter_javascript::language as javascript_language;

// Conversión de Tree-sitter AST a nuestro SimplifiedAST
impl From<&Node> for ASTNode {
    fn from(node: &Node) -> Self {
        let node_type = map_tree_sitter_kind(node.kind(), language);
        // ... conversión completa
    }
}
```

### Testing y Calidad

El módulo incluye una suite de testing completa:

- **60 Unit Tests**: Tests específicos por parser y componente
- **104 Integration Tests**: Tests end-to-end y casos reales
- **Property-based Tests**: Verifican invariantes del sistema
- **Golden Tests**: Regression testing con archivos de referencia
- **Performance Benchmarks**: Medición de rendimiento con Criterion

```bash
# Ejecutar todos los tests
cargo test

# Tests específicos
cargo test --test integration
cargo test language::detector

# Benchmarks
cargo bench
```

---

## English

### Overview

The **Code Context Graph Parser** module is a robust multi-language parser built on Tree-sitter that converts source code into simplified, extractable Abstract Syntax Trees (AST). Designed for large-scale code analysis, automated refactoring, and context graph generation.

### Key Features

- **🌍 Multi-language**: Native support for Python, Java, JavaScript, and Kotlin
- **🚀 High Performance**: Incremental parsing and intelligent caching
- **🏗️ Abstract Architecture**: Abstraction layer that unifies different syntaxes
- **🔍 Visitor Pattern**: Extensible system for analysis and data extraction
- **⚡ Tree-sitter**: Built on the fastest and most accurate parsing library
- **📊 Rich Metadata**: Automatic extraction of semantic information
- **🧪 Comprehensive Testing**: 164+ tests with 100% critical coverage

### System Architecture

The parser follows a layered architecture with clear separation of concerns:

1. **Language Detection Layer**: Automatic language identification
2. **Tree-sitter Integration**: Direct integration with language-specific parsers
3. **AST Abstraction**: Unified representation across languages
4. **Visitor System**: Extensible analysis and extraction framework

### Basic Usage

```rust
use code_context_graph_parser::{ParserRegistry, Language};

// Initialize parser registry
let registry = ParserRegistry::new();

// Parse source code
let source_code = r#"
public class Calculator {
    public int add(int a, int b) {
        return a + b;
    }
}
"#;

let ast = registry.parse(source_code, Language::Java)?;

// Extract information
let classes = ast.find_all_classes();
let methods = ast.find_all_methods();

println!("Found {} classes and {} methods", 
         classes.len(), methods.len());
```

### Supported Languages

| Language   | Status | Supported Features |
|------------|--------|--------------------|
| **Python** | ✅ | Classes, functions, decorators, imports, async/await |
| **Java** | ✅ | Classes, interfaces, enums, generics, annotations |
| **JavaScript** | ✅ | ES6 classes, arrow functions, destructuring, modules |
| **Kotlin** | ✅ | Data classes, objects, coroutines, extensions |

### Testing and Quality

The module includes a comprehensive testing suite ensuring reliability and correctness across all supported languages and use cases.

### Documentation Structure

- [`README.md`](README.md) - This overview document
- [`tree-sitter-integration.md`](tree-sitter-integration.md) - Tree-sitter integration details
- [`ast-abstraction.md`](ast-abstraction.md) - AST architecture documentation
- [`visitor-pattern.md`](visitor-pattern.md) - Visitor system guide
- [`examples/`](examples/) - Usage examples and tutorials

### Contributing

When contributing to this module, please ensure:
1. All tests pass (`cargo test`)
2. New features include comprehensive tests
3. Documentation is updated accordingly
4. Code follows project conventions