# Troubleshooting and FAQ

## Common Issues and Solutions

### Installation and Setup

#### Issue: Compilation fails with "tree-sitter" errors

**Symptoms:**
```
error: failed to run custom build command for `tree-sitter-python v0.20.4`
```

**Solution:**
1. Ensure you have a C compiler installed:
   ```bash
   # Ubuntu/Debian
   sudo apt install build-essential
   
   # CentOS/RHEL
   sudo yum groupinstall "Development Tools"
   
   # macOS
   xcode-select --install
   ```

2. Update Rust to the latest version:
   ```bash
   rustup update
   ```

3. Clean and rebuild:
   ```bash
   cargo clean
   cargo build
   ```

#### Issue: FalkorDB connection fails

**Symptoms:**
```
Error: Failed to connect to FalkorDB: Connection refused (os error 61)
```

**Solution:**
1. Verify FalkorDB is running:
   ```bash
   docker ps | grep falkordb
   ```

2. Start FalkorDB if not running:
   ```bash
   docker run -p 6379:6379 falkordb/falkordb:latest
   ```

3. Test connection manually:
   ```bash
   redis-cli -p 6379 ping
   ```

4. Check configuration:
   ```toml
   [falkordb]
   url = "redis://localhost:6379"  # Correct format
   # Not: url = "localhost:6379"   # Missing protocol
   ```

#### Issue: Permission denied for storage directories

**Symptoms:**
```
Error: Permission denied (os error 13) when creating ./cas_store
```

**Solution:**
1. Create directories with proper permissions:
   ```bash
   mkdir -p ./cas_store ./context
   chmod 755 ./cas_store ./context
   ```

2. Or use a different location:
   ```toml
   [cas]
   storage_path = "/tmp/ccg_cas"
   
   [aase]
   context_path = "/tmp/ccg_context"
   ```

### Parsing Issues

#### Issue: Parser fails on large files

**Symptoms:**
```
Error: Parser timeout after 30 seconds for file large_file.py
Warning: File large_file.py (5MB) exceeds recommended size
```

**Solution:**
1. Increase parser limits:
   ```toml
   [parser]
   max_file_size_kb = 10240  # 10MB instead of default 1MB
   timeout_seconds = 60      # 60s instead of default 30s
   ```

2. Or exclude large files:
   ```toml
   [parser]
   ignore_patterns = [
       "*.min.js",
       "large_generated_file.py",
       "**/*_generated.py"
   ]
   ```

#### Issue: Parsing fails for specific language constructs

**Symptoms:**
```
Error: Failed to parse decorators in python file
Warning: Unknown AST node type 'async_with_stmt'
```

**Solution:**
1. Update Tree-sitter parsers:
   ```bash
   cargo update
   ```

2. Check if the language feature is supported:
   ```bash
   ccg parser info --language python
   ```

3. Report unsupported constructs or use ignore patterns for files with problematic syntax.

#### Issue: Parser runs out of memory

**Symptoms:**
```
Error: Memory allocation failed
Process killed (signal 9)
```

**Solution:**
1. Reduce parallel workers:
   ```toml
   [parser]
   parallel_workers = 2  # Instead of auto-detected CPU count
   ```

2. Enable streaming for large files:
   ```toml
   [parser]
   enable_streaming = true
   max_memory_mb = 512
   ```

3. Exclude large directories:
   ```toml
   [parser]
   ignore_patterns = ["node_modules/**", "target/**", ".git/**"]
   ```

### File Watching Issues

#### Issue: File watcher doesn't detect changes

**Symptoms:**
- Changes to files don't trigger analysis updates
- API shows stale data after file modifications

**Solution:**
1. Check if file watcher is enabled:
   ```toml
   [file_watcher]
   enabled = true
   ```

2. Verify file permissions:
   ```bash
   # Files should be readable by the user running CCG
   ls -la ./src
   ```

3. Increase debounce time for network filesystems:
   ```toml
   [file_watcher]
   debounce_ms = 1000  # Increase from default 100ms
   ```

4. Check if path is being watched:
   ```bash
   ccg watch status
   ```

#### Issue: Too many file events causing performance issues

**Symptoms:**
```
Warning: File watcher queue overflow, some events may be lost
CPU usage consistently high during development
```

**Solution:**
1. Increase batch threshold:
   ```toml
   [file_watcher]
   batch_threshold = 200  # Process more files per batch
   debounce_ms = 500      # Wait longer before processing
   ```

2. Add more ignore patterns:
   ```toml
   [file_watcher]
   ignore_patterns = [
       ".git/**",
       "node_modules/**",
       "target/**",
       "**/*.tmp",
       "**/*.log",
       ".DS_Store"
   ]
   ```

3. Limit maximum events per second:
   ```toml
   [file_watcher]
   max_events_per_second = 100
   ```

### Storage Issues

#### Issue: CAS storage grows too large

**Symptoms:**
- Disk space running out
- `./cas_store` directory is very large

**Solution:**
1. Enable garbage collection:
   ```toml
   [versioning]
   garbage_collection_enabled = true
   gc_interval_hours = 6
   retention_days = 7  # Keep only 1 week of history
   ```

2. Run manual garbage collection:
   ```bash
   ccg storage gc --aggressive
   ```

3. Enable compression:
   ```toml
   [cas]
   compression = "zstd"
   compression_level = 6
   ```

4. Reduce deduplication threshold to store fewer similar files:
   ```toml
   [cas]
   dedup_threshold = 0.95  # Only dedupe if 95% similar
   ```

#### Issue: Merkle tree corruption

**Symptoms:**
```
Error: Invalid Merkle tree node hash
Error: Failed to verify tree integrity
```

**Solution:**
1. Verify storage integrity:
   ```bash
   ccg storage verify --repair
   ```

2. Rebuild from scratch if corruption is extensive:
   ```bash
   ccg storage rebuild --backup-corrupted
   ```

3. Check for hardware issues:
   ```bash
   # Run filesystem check
   fsck /dev/sda1
   
   # Check memory
   memtest86+
   ```

### API Issues

#### Issue: API requests timeout

**Symptoms:**
```
Error: Request timeout after 30 seconds
504 Gateway Timeout
```

**Solution:**
1. Increase API timeout:
   ```toml
   [api]
   request_timeout_seconds = 60
   ```

2. Optimize queries by adding filters:
   ```json
   {
     "question": "Show payment functions",
     "filters": {
       "file_patterns": ["src/payment/**"],
       "max_hops": 2
     }
   }
   ```

3. Use pagination for large results:
   ```bash
   curl "http://localhost:8080/api/v1/graph/nodes?limit=50&offset=0"
   ```

#### Issue: High memory usage during API queries

**Symptoms:**
- API server memory usage keeps growing
- Out of memory errors during complex queries

**Solution:**
1. Limit query depth:
   ```toml
   [security]
   max_query_depth = 5
   ```

2. Enable result streaming:
   ```toml
   [api]
   enable_streaming = true
   max_response_size_mb = 50
   ```

3. Implement query result caching:
   ```toml
   [api.caching]
   enabled = true
   cache_ttl_seconds = 300
   max_cache_size_mb = 100
   ```

### Quality Analysis Issues

#### Issue: Connascence analysis takes too long

**Symptoms:**
- Analysis hangs on connascence detection
- Very slow progress on large codebases

**Solution:**
1. Disable expensive connascence types:
   ```toml
   [connascence.types]
   timing = { enabled = false }  # Most expensive
   execution = { enabled = false }
   ```

2. Increase thresholds to reduce candidates:
   ```toml
   [connascence]
   strength_threshold = 0.8  # Only strong connascence
   degree_threshold = 10     # At least 10 entities involved
   ```

3. Limit analysis scope:
   ```bash
   ccg connascence analyze ./src/core --max-files 100
   ```

#### Issue: False positive connascence detections

**Symptoms:**
- Many irrelevant connascence warnings
- Common naming patterns flagged as problems

**Solution:**
1. Adjust detection sensitivity:
   ```toml
   [connascence.types]
   name = { enabled = true, weight = 0.5 }  # Reduce weight
   ```

2. Add exclusion patterns:
   ```toml
   [connascence]
   exclude_patterns = [
       "test*",      # Exclude test files
       "*_test.py",
       "common_*"    # Exclude common utilities
   ]
   ```

3. Use allow-lists for common patterns:
   ```toml
   [connascence.allow_lists]
   common_names = ["id", "name", "value", "data", "result"]
   ```

### AASE Context Issues

#### Issue: Context generation fails with LLM errors

**Symptoms:**
```
Error: LLM API call failed: Rate limit exceeded
Error: Context generation timeout
```

**Solution:**
1. Configure retry settings:
   ```toml
   [aase.generation]
   max_retries = 3
   retry_delay_seconds = 5
   ```

2. Use caching to reduce API calls:
   ```toml
   [aase.generation]
   enable_caching = true
   cache_ttl_hours = 24
   ```

3. Implement fallback to simpler prompts:
   ```toml
   [aase.generation]
   fallback_to_simple = true
   simple_prompt_template = "basic_context"
   ```

#### Issue: Generated contexts have low quality scores

**Symptoms:**
- Context validation fails
- Low completeness/consistency scores

**Solution:**
1. Adjust quality thresholds:
   ```toml
   [aase.quality]
   min_completeness = 0.6    # Lower from 0.8
   min_consistency = 0.7     # Lower from 0.9
   ```

2. Enable iterative improvement:
   ```toml
   [aase.quality]
   auto_improve = true
   max_improvement_iterations = 3
   ```

3. Provide more input context:
   ```bash
   ccg aase generate-context --domain payment \
     --include-examples \
     --include-documentation \
     --analyze ./src/payment
   ```

## Performance Optimization

### General Performance Tips

1. **Use appropriate hardware:**
   - SSD storage for better I/O performance
   - Sufficient RAM (4GB+ for large codebases)
   - Multi-core CPU for parallel processing

2. **Optimize configuration:**
   ```toml
   [parser]
   parallel_workers = 8      # Match CPU cores
   enable_incremental = true # Only reparse changed files
   
   [cas]
   compression = "lz4"       # Faster than zstd
   
   [file_watcher]
   debounce_ms = 200         # Balance responsiveness vs load
   ```

3. **Use selective analysis:**
   ```bash
   # Analyze only specific directories
   ccg analyze ./src/core --languages python,rust
   
   # Skip expensive analysis
   ccg analyze ./src --no-connascence --no-aase
   ```

### Memory Usage Optimization

1. **Reduce memory footprint:**
   ```toml
   [parser]
   max_file_size_kb = 512    # Limit large files
   streaming_threshold_kb = 100
   
   [api]
   max_response_size_mb = 10 # Limit API responses
   
   [cas.cache]
   max_entries = 1000        # Reduce cache size
   ```

2. **Monitor memory usage:**
   ```bash
   # Enable memory profiling
   ccg analyze ./src --profile-memory
   
   # Check current usage
   ccg status --memory
   ```

### Disk Usage Optimization

1. **Enable compression:**
   ```toml
   [cas]
   compression = "zstd"
   compression_level = 6
   
   [versioning]
   compression_enabled = true
   ```

2. **Aggressive garbage collection:**
   ```toml
   [versioning.gc]
   retention_days = 7
   min_versions_to_keep = 5
   enable_compaction = true
   ```

3. **Regular cleanup:**
   ```bash
   # Clean old versions
   ccg storage gc --keep-days 7
   
   # Compact storage
   ccg storage compact
   ```

## Debugging

### Enable Debug Logging

```toml
[logging]
level = "debug"
modules = [
    "code_context_graph::parser" = "debug",
    "code_context_graph::storage" = "info",
    "code_context_graph::api" = "debug"
]
```

Or use environment variables:
```bash
export RUST_LOG=debug
export CCG_LOG_LEVEL=debug
ccg analyze ./src
```

### Diagnostic Commands

```bash
# Check system status
ccg status --verbose

# Verify configuration
ccg config validate

# Test individual components
ccg parser test ./src/example.py
ccg storage verify
ccg api health-check

# Export diagnostic information
ccg diagnostics export --output diagnostics.json
```

### Common Debug Scenarios

1. **Parser issues:**
   ```bash
   # Test parsing specific file
   ccg parser parse ./problematic_file.py --debug
   
   # Show AST for debugging
   ccg parser ast ./file.py --pretty
   ```

2. **Storage issues:**
   ```bash
   # Verify storage integrity
   ccg storage verify --verbose
   
   # Show storage statistics
   ccg storage stats
   ```

3. **API issues:**
   ```bash
   # Test API endpoints
   curl -v http://localhost:8080/health
   
   # Enable API debug logging
   ccg api start --log-level debug
   ```

## FAQ

### Q: Can I use Code Context Graph with non-UTF8 files?

**A:** Currently, Code Context Graph assumes UTF-8 encoding. For other encodings:
1. Convert files to UTF-8 before analysis
2. Use encoding detection and conversion in preprocessing
3. Add encoding configuration (planned for future releases)

### Q: How much disk space will Code Context Graph use?

**A:** Disk usage depends on:
- Codebase size: ~10-50% of original size with compression
- History retention: Configure `retention_days` to control
- Deduplication: Significant savings for repeated code patterns

Example for 1GB codebase:
- Without compression: ~1.2GB (with metadata)
- With compression: ~400-600MB
- With 30 days history: ~2-5GB (depending on change frequency)

### Q: Can I analyze multiple programming languages simultaneously?

**A:** Yes, Code Context Graph supports multi-language analysis:
```toml
[engine]
languages = ["python", "javascript", "java", "kotlin", "rust"]
```

Cross-language relationships (like REST API calls) are detected where possible.

### Q: How do I backup and restore Code Context Graph data?

**A:** 
1. **Backup:**
   ```bash
   # Backup storage
   tar -czf ccg_backup.tar.gz ./cas_store ./context ./versions
   
   # Export graph data
   ccg export --format json --output backup.json
   ```

2. **Restore:**
   ```bash
   # Restore storage
   tar -xzf ccg_backup.tar.gz
   
   # Import graph data
   ccg import --format json --input backup.json
   ```

### Q: Can I run Code Context Graph in CI/CD pipelines?

**A:** Yes, Code Context Graph is designed for CI/CD:
```yaml
# Example GitHub Actions
- name: Analyze Code Quality
  run: |
    ccg analyze ./src --format json --output analysis.json
    ccg quality check --fail-on-threshold 0.7
    ccg connascence analyze --max-strength 0.8
```

### Q: How do I contribute new language support?

**A:** To add a new language:
1. Add Tree-sitter parser dependency
2. Implement language-specific AST simplifier
3. Add language detection rules
4. Create test cases
5. Submit pull request

See `docs/parser.md` for detailed instructions.

### Q: What's the difference between Code Context Graph and other code analysis tools?

**A:** Code Context Graph focuses on:
- **Semantic relationships:** Not just syntax, but how code components interact
- **LLM integration:** Optimized for AI-assisted development workflows
- **Versioning:** Complete history with efficient storage
- **Connascence analysis:** Advanced coupling detection beyond traditional metrics
- **Context engineering:** Rich metadata for better AI understanding

### Q: Can I run Code Context Graph on large codebases (>1M LOC)?

**A:** Yes, with proper configuration:
```toml
[parser]
parallel_workers = 16
max_file_size_kb = 1024

[cas]
compression = "zstd"
compression_level = 6

[versioning.gc]
retention_days = 14
enable_compaction = true
```

Performance considerations:
- Initial analysis: ~10-30 minutes for 1M LOC
- Incremental updates: ~1-5 seconds
- Memory usage: ~2-4GB during analysis
- Storage: ~500MB-2GB depending on complexity

### Q: How do I integrate with existing development tools?

**A:** Code Context Graph provides multiple integration options:
- **REST API:** For custom integrations
- **CLI:** For scripts and automation
- **WebSocket:** For real-time updates
- **File exports:** JSON, GraphQL schema, etc.

Example integrations:
- VS Code extension (community developed)
- GitHub Actions workflow
- Jenkins pipeline integration
- Slack notifications for quality changes

For specific integration questions, check the API documentation or create an issue on GitHub.

## Getting Help

### Support Channels

1. **Documentation:** Start with the comprehensive docs in `/docs`
2. **GitHub Issues:** Report bugs and request features
3. **Community:** Join discussions in GitHub Discussions
4. **Stack Overflow:** Tag questions with `code-context-graph`

### Reporting Bugs

When reporting issues, please include:
1. Code Context Graph version: `ccg --version`
2. Operating system and version
3. Configuration file (remove sensitive data)
4. Steps to reproduce
5. Expected vs actual behavior
6. Log output with debug enabled
7. Sample code that triggers the issue (if applicable)

### Feature Requests

For feature requests, please describe:
1. Use case and motivation
2. Proposed solution or API
3. Alternative solutions considered
4. Impact on existing functionality
5. Implementation complexity estimate

This troubleshooting guide should help resolve most common issues. For complex problems, don't hesitate to reach out to the community or maintainers.