# Development Guide

## Getting Started

### Prerequisites
- Rust 1.75+
- FalkorDB (Docker recommended)
- Git

### Setup Development Environment

```bash
# Clone repository
git clone https://github.com/rubentxu/code-context-graph
cd code-context-graph

# Start FalkorDB
docker run -p 6379:6379 falkordb/falkordb:latest

# Build project
cargo build

# Run tests
cargo test
```

## Development Workflow

### 1. Test-Driven Development (TDD)

Following the required TDD workflow:

1. **Red**: Write a failing test
2. **Green**: Write minimal code to pass
3. **Refactor**: Improve while keeping tests green
4. **Commit**: Use conventional commit format

### 2. Code Quality Standards

- **SOLID Principles**: Applied throughout architecture
- **Clean Code**: Self-documenting, readable code
- **Documentation**: All public APIs documented in English
- **No Code Duplication**: Reuse existing functionality

### 3. Conventional Commits

All commits must follow this format:
```
type(scope): description

feat(parser): add support for Kotlin language
fix(storage): resolve hash collision edge case
docs(api): update REST endpoint documentation
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

## Project Structure Guidelines

### Crate Organization
- **core**: Domain types, no external dependencies
- **parser**: Tree-sitter integration, language-specific
- **storage**: CAS and Merkle tree implementation
- **graph**: FalkorDB operations and queries
- **api**: REST endpoints and serialization
- **cli**: Command-line interface

### Dependency Management
- All dependencies in `Cargo.toml` workspace
- Use `libs.versions.toml` for version centralization
- Minimize external dependencies per crate

## Testing Strategy

### Unit Tests
```bash
# Run all unit tests
cargo test --lib

# Test specific crate
cargo test -p code-context-graph-core

# Test with coverage
cargo tarpaulin --out Html
```

### Integration Tests
```bash
# Run integration tests
cargo test --test integration

# Test with embedded FalkorDB
cargo test test_graph_operations
```

### Property-Based Testing
Using `proptest` for edge case discovery:

```rust
proptest! {
    #[test]
    fn hash_consistency(content in any::<Vec<u8>>()) {
        let hash1 = Hash::new(&content);
        let hash2 = Hash::new(&content);
        prop_assert_eq!(hash1, hash2);
    }
}
```

## Code Style

### Rust Conventions
- Use `cargo fmt` for formatting
- Follow `cargo clippy` suggestions
- Prefer `anyhow::Result` for errors
- Use `tracing` for logging, not `println!`

### Error Handling
```rust
// Good: Use domain-specific errors
fn parse_file(path: &Path) -> Result<AST> {
    let content = fs::read_to_string(path)
        .map_err(|e| CodeGraphError::Parser { 
            message: format!("Failed to read {}: {}", path.display(), e) 
        })?;
    // ...
}

// Avoid: Generic errors
fn parse_file(path: &Path) -> anyhow::Result<AST> {
    let content = fs::read_to_string(path)?; // Too generic
    // ...
}
```

## Architecture Patterns

### Hexagonal Architecture
```
Domain (core) -> Application (use cases) -> Infrastructure (adapters)
```

### Dependency Injection
```rust
// Use trait objects for testability
trait GraphStore {
    async fn store_node(&self, node: CodeNode) -> Result<()>;
}

struct GraphService<S: GraphStore> {
    store: S,
}
```

### Event-Driven Processing
```rust
// File watcher events
enum FileEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Deleted(PathBuf),
}

// Handler chain
async fn handle_file_event(event: FileEvent) -> Result<()> {
    match event {
        FileEvent::Modified(path) => {
            let changes = detect_changes(&path).await?;
            update_graph(changes).await?;
            notify_subscribers(path).await?;
        }
        // ...
    }
}
```

## Performance Guidelines

### Memory Efficiency
- Use streaming for large files
- Implement `Drop` for cleanup
- Profile memory usage regularly

### Async Best Practices
- Use `tokio::spawn` for CPU-bound tasks
- Prefer `async/await` over futures combinators
- Handle backpressure with bounded channels

### Database Operations
- Batch graph updates when possible
- Use prepared statements/queries
- Implement connection pooling

## Debugging

### Logging
```rust
use tracing::{info, debug, warn, error};

// Structured logging
info!(
    file_path = %path.display(),
    node_count = nodes.len(),
    "Parsed file successfully"
);
```

### Development Tools
```bash
# Debug build with full symbols
cargo build --features debug-symbols

# Run with tracing
RUST_LOG=debug cargo run -- analyze --path ./test-project

# Profile performance
cargo flamegraph --bin ccg -- analyze --path ./large-project
```

## Contributing Checklist

- [ ] Tests written and passing
- [ ] Documentation updated
- [ ] Code formatted (`cargo fmt`)
- [ ] No clippy warnings
- [ ] Conventional commit message
- [ ] No breaking changes (or documented)
- [ ] Performance impact assessed

## Release Process

### Version Management
- Use semantic versioning (v0.1.0, v0.2.0, etc.)
- Tag releases with `git tag v0.1.0`
- Update `CHANGELOG.md` with conventional commits

### Build and Test
```bash
# Full test suite
cargo test --all-features

# Release build
cargo build --release

# Documentation
cargo doc --no-deps --open
```

## IDE Setup

### VS Code Extensions
- `rust-analyzer`: Rust language support
- `crates`: Cargo.toml management
- `Error Lens`: Inline error display

### Settings
```json
{
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.check.command": "clippy"
}
```