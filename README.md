# Code Context Graph

A multi-language code analysis tool that generates semantic graphs with LLM integration, Content-Addressable Storage (CAS), and real-time file watching capabilities.

## 🚀 Features

- **Multi-language support**: Python, Java, Kotlin, JavaScript
- **Semantic graph generation**: AST-based code relationships
- **Real-time updates**: File watching with incremental analysis
- **Content-Addressable Storage**: Efficient deduplication and versioning
- **Connascence analysis**: Coupling and cohesion metrics
- **LLM integration**: Optimized context for AI assistants
- **AASE framework**: Automated context engineering

## 📋 Requirements

- Rust 1.75+
- FalkorDB (Redis-compatible graph database)

## 🛠️ Installation

```bash
git clone https://github.com/rubentxu/code-context-graph
cd code-context-graph
cargo build --release
```

## 🎯 Quick Start

```bash
# Analyze a codebase
./target/release/ccg analyze --path /path/to/your/project

# Start real-time watching
./target/release/ccg watch --path /path/to/your/project

# Query the graph
./target/release/ccg query --question "What functions call authenticate?"

# Analyze code quality
./target/release/ccg quality --path /path/to/your/project

# Start API server
./target/release/ccg serve --port 8080
```

## 📁 Project Structure

```
code-context-graph/
├── crates/
│   ├── core/           # Core types and domain logic
│   ├── parser/         # Tree-sitter parsing
│   ├── graph/          # Graph operations (FalkorDB)
│   ├── storage/        # CAS + Merkle tree storage
│   ├── watcher/        # File system monitoring
│   ├── api/            # REST API server
│   ├── connascence/    # Coupling analysis
│   ├── aase/           # Context engineering
│   └── cli/            # Command-line interface
├── config.toml         # Default configuration
└── docs/               # Documentation
```

## ⚙️ Configuration

Create a `config.toml` file or use the default configuration:

```toml
[engine]
name = "my-project"
languages = ["python", "javascript", "java", "kotlin"]

[falkordb]
url = "redis://localhost:6379"
graph_name = "code_graph"

[cas]
enabled = true
storage_path = "./cas_store"

[file_watcher]
enabled = true
debounce_ms = 100
```

## 🔧 Development

```bash
# Run tests
cargo test

# Run with logging
RUST_LOG=debug cargo run -- analyze --path ./examples/python

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy
```

## 📊 Architecture

The system follows **Hexagonal Architecture** with clear separation between:

- **Domain Layer**: Core types and business logic
- **Application Layer**: Use cases and orchestration  
- **Infrastructure Layer**: Database, file system, external services

## 🧪 Testing

```bash
# Unit tests
cargo test --lib

# Integration tests  
cargo test --test integration

# Property-based tests
cargo test proptest
```

## 📈 Performance

- **Analysis**: <10s for 500 files
- **Real-time updates**: <100ms for file changes
- **Storage efficiency**: >85% deduplication with CAS
- **Memory usage**: <2GB for 500k LOC projects

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Follow conventional commits
4. Add tests for new functionality
5. Submit a pull request

## 📄 License

MIT OR Apache-2.0

---

**Status**: 🚧 Under Development

This is the initial structure implementation. Core functionality is being developed incrementally following the roadmap in the PRD.