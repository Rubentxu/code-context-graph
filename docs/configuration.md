# Configuration Reference

## Overview

Code Context Graph uses a hierarchical configuration system supporting multiple sources: configuration files, environment variables, and command-line arguments. The system follows the principle of "convention over configuration" with sensible defaults.

## Configuration Sources (Priority Order)

1. **Command-line arguments** (highest priority)
2. **Environment variables**
3. **Configuration file** (config.toml)
4. **Default values** (lowest priority)

## Configuration File Structure

### Main Configuration File (config.toml)

```toml
[engine]
name = "my-project"
version = "1.0.0"
languages = ["python", "javascript", "java", "kotlin"]
root_path = "./src"
output_format = "json"
log_level = "info"

[parser]
max_file_size_kb = 1024
max_files = 10000
timeout_seconds = 30
ignore_patterns = [
    "*_test.py",
    "*.min.js",
    "node_modules/**",
    "target/**",
    ".git/**"
]
include_patterns = [
    "src/**/*.py",
    "src/**/*.js",
    "src/**/*.java",
    "src/**/*.kt"
]
enable_incremental = true
parallel_workers = 0  # 0 = auto-detect CPU cores

[falkordb]
url = "redis://localhost:6379"
database = 0
graph_name = "code_graph"
connection_pool_size = 10
timeout_ms = 5000
enable_tls = false
username = ""
password = ""

[cas]
enabled = true
storage_path = "./cas_store"
hash_algorithm = "blake3"
compression = "zstd"
compression_level = 3
dedup_threshold = 0.85
max_content_size_mb = 100
cleanup_interval_hours = 24
retention_days = 30

[file_watcher]
enabled = true
debounce_ms = 100
batch_threshold = 50
max_events_per_second = 1000
ignore_patterns = [
    ".git/**",
    "node_modules/**",
    "target/**",
    "__pycache__/**",
    "*.tmp",
    "*.swp",
    ".DS_Store"
]
recursive = true
follow_symlinks = false

[versioning]
enabled = true
max_versions = 1000
auto_snapshot_interval_seconds = 3600
merkle_fanout = 16
compression_enabled = true
garbage_collection_enabled = true
gc_interval_hours = 6

[connascence]
enabled = true
detect_static = true
detect_dynamic = true
strength_threshold = 0.7
locality_threshold = 0.5
degree_threshold = 5
auto_suggest_refactoring = true
export_metrics = true

[connascence.types]
name = { enabled = true, weight = 1.0 }
type = { enabled = true, weight = 1.2 }
meaning = { enabled = true, weight = 1.4 }
position = { enabled = true, weight = 1.6 }
algorithm = { enabled = true, weight = 1.8 }
execution = { enabled = true, weight = 2.0 }
timing = { enabled = false, weight = 2.2 }  # Experimental
values = { enabled = true, weight = 2.4 }
identity = { enabled = true, weight = 2.6 }

[aase]
enabled = true
context_path = "./context"
naming_convention = "strict"
auto_propagate = true
human_review_threshold = 0.8
artifact_versioning = true
context_chain_depth = 5
template_path = "./templates"

[aase.artifact_types]
context = { prefix = "CTX", enabled = true }
model = { prefix = "MDL", enabled = true }
use_case = { prefix = "UCS", enabled = true }
prompt = { prefix = "PRM", enabled = true }
specification = { prefix = "SPC", enabled = true }

[quality_metrics]
calculate_cohesion = true
calculate_coupling = true
calculate_complexity = true
maintainability_threshold = 65
complexity_warning = 10
coupling_warning = 0.8
cohesion_warning = 0.3
export_to_prometheus = false

[api]
host = "0.0.0.0"
port = 8080
max_connections = 1000
request_timeout_seconds = 30
max_request_size_mb = 10
enable_cors = true
cors_origins = ["*"]
enable_compression = true
api_key_required = false
api_key = ""

[api.rate_limiting]
enabled = true
requests_per_minute = 100
burst_size = 20
key_strategy = "ip"  # "ip", "api_key", "user"

[api.endpoints]
query = { enabled = true, path = "/api/v1/query" }
graph = { enabled = true, path = "/api/v1/graph" }
versions = { enabled = true, path = "/api/v1/versions" }
quality = { enabled = true, path = "/api/v1/quality" }
aase = { enabled = true, path = "/api/v1/aase" }
websocket = { enabled = true, path = "/ws/updates" }

[logging]
level = "info"  # trace, debug, info, warn, error
format = "json"  # json, pretty, compact
output = "stdout"  # stdout, stderr, file
file_path = "./logs/ccg.log"
rotation = "daily"  # never, hourly, daily, weekly
max_files = 7
structured = true

[logging.modules]
"code_context_graph::parser" = "debug"
"code_context_graph::graph" = "info"
"code_context_graph::storage" = "warn"

[monitoring]
enabled = false
prometheus_port = 9090
health_check_port = 8081
metrics_interval_seconds = 60
trace_sampling_rate = 0.1

[security]
enable_tls = false
cert_file = ""
key_file = ""
ca_file = ""
allowed_hosts = []
max_query_depth = 10
sanitize_inputs = true
```

## Environment Variables

All configuration options can be overridden using environment variables with the `CCG_` prefix:

### Engine Configuration
```bash
export CCG_ENGINE_NAME="production-analyzer"
export CCG_ENGINE_LANGUAGES="python,javascript,java"
export CCG_ENGINE_ROOT_PATH="/app/src"
export CCG_ENGINE_LOG_LEVEL="debug"
```

### Parser Configuration
```bash
export CCG_PARSER_MAX_FILE_SIZE_KB=2048
export CCG_PARSER_TIMEOUT_SECONDS=60
export CCG_PARSER_PARALLEL_WORKERS=8
export CCG_PARSER_ENABLE_INCREMENTAL=true
```

### Database Configuration
```bash
export CCG_FALKORDB_URL="redis://redis.example.com:6379"
export CCG_FALKORDB_DATABASE=1
export CCG_FALKORDB_PASSWORD="your-password"
export CCG_FALKORDB_ENABLE_TLS=true
```

### CAS Configuration
```bash
export CCG_CAS_ENABLED=true
export CCG_CAS_STORAGE_PATH="/data/cas"
export CCG_CAS_HASH_ALGORITHM="blake3"
export CCG_CAS_COMPRESSION="lz4"
```

### File Watcher Configuration
```bash
export CCG_FILE_WATCHER_ENABLED=true
export CCG_FILE_WATCHER_DEBOUNCE_MS=200
export CCG_FILE_WATCHER_BATCH_THRESHOLD=100
```

### API Configuration
```bash
export CCG_API_HOST="0.0.0.0"
export CCG_API_PORT=8080
export CCG_API_API_KEY="your-secret-key"
export CCG_API_ENABLE_CORS=true
```

## Command Line Arguments

### Global Options
```bash
# Configuration file
--config, -c <PATH>          Path to configuration file
--log-level, -l <LEVEL>      Log level (trace|debug|info|warn|error)
--verbose, -v                Enable verbose output
--quiet, -q                  Suppress non-error output

# Engine options
--root-path <PATH>           Root path to analyze
--languages <LANGS>          Comma-separated list of languages
--output-format <FORMAT>     Output format (json|yaml|pretty)
```

### Analyze Command
```bash
ccg analyze [OPTIONS] <PATH>

OPTIONS:
    --languages <LANGS>           Languages to analyze [default: auto-detect]
    --max-files <N>              Maximum number of files to process
    --include <PATTERNS>          Include patterns (glob)
    --exclude <PATTERNS>          Exclude patterns (glob)
    --output <PATH>              Output file path
    --format <FORMAT>            Output format [default: json]
    --parallel <N>               Number of parallel workers
    --incremental                Enable incremental analysis
    --no-cache                   Disable CAS caching
    --quality-metrics            Include quality metrics
    --connascence                Include connascence analysis
    --aase                       Generate AASE artifacts
```

### Watch Command
```bash
ccg watch [OPTIONS] <PATH>

OPTIONS:
    --config <PATH>              Configuration file
    --debounce <MS>              Debounce interval in milliseconds
    --batch-size <N>             Batch size for bulk changes
    --ignore <PATTERNS>          Additional ignore patterns
    --api-server                 Start API server alongside watcher
    --port <PORT>                API server port
```

### Query Command
```bash
ccg query [OPTIONS] <QUESTION>

OPTIONS:
    --endpoint <URL>             API endpoint URL
    --api-key <KEY>              API key for authentication
    --max-hops <N>               Maximum relationship hops
    --include-code               Include source code in results
    --format <FORMAT>            Output format [default: pretty]
    --timeout <SECONDS>          Query timeout
```

### Version Command
```bash
ccg version [OPTIONS]

SUBCOMMANDS:
    list                         List all versions
    show <VERSION>               Show version details
    compare <V1> <V2>            Compare two versions
    checkout <VERSION>           Checkout specific version
    gc                           Run garbage collection
```

## Configuration Validation

### Schema Validation

The configuration is validated against a JSON schema:

```rust
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Config {
    #[validate(nested)]
    pub engine: EngineConfig,
    
    #[validate(nested)]
    pub parser: ParserConfig,
    
    #[validate(nested)]
    pub falkordb: FalkorDBConfig,
    
    #[validate(nested)]
    pub cas: CASConfig,
    
    #[validate(nested)]
    pub file_watcher: FileWatcherConfig,
    
    #[validate(nested)]
    pub api: ApiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct EngineConfig {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    
    #[validate(length(min = 1))]
    pub languages: Vec<String>,
    
    #[validate(custom = "validate_path_exists")]
    pub root_path: Option<PathBuf>,
}

fn validate_path_exists(path: &PathBuf) -> Result<(), ValidationError> {
    if path.exists() {
        Ok(())
    } else {
        Err(ValidationError::new("path_not_found"))
    }
}
```

### Runtime Validation

Configuration is validated at startup:

```rust
impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        let config = Self::from_sources()?;
        config.validate()?;
        config.post_process()
    }
    
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate language support
        for lang in &self.engine.languages {
            if !SUPPORTED_LANGUAGES.contains(&lang.as_str()) {
                return Err(ConfigError::UnsupportedLanguage(lang.clone()));
            }
        }
        
        // Validate storage paths
        if self.cas.enabled {
            let cas_path = Path::new(&self.cas.storage_path);
            if let Some(parent) = cas_path.parent() {
                if !parent.exists() {
                    return Err(ConfigError::StoragePathInvalid(cas_path.to_path_buf()));
                }
            }
        }
        
        // Validate connection parameters
        if let Err(e) = url::Url::parse(&self.falkordb.url) {
            return Err(ConfigError::InvalidDatabaseUrl(e));
        }
        
        Ok(())
    }
    
    fn post_process(mut self) -> Result<Self, ConfigError> {
        // Set default parallel workers to CPU count
        if self.parser.parallel_workers == 0 {
            self.parser.parallel_workers = num_cpus::get();
        }
        
        // Ensure output directory exists
        if let Some(output_dir) = &self.engine.output_path {
            std::fs::create_dir_all(output_dir)?;
        }
        
        // Expand environment variables in paths
        self.cas.storage_path = shellexpand::tilde(&self.cas.storage_path).to_string();
        self.aase.context_path = shellexpand::tilde(&self.aase.context_path).to_string();
        
        Ok(self)
    }
}
```

## Configuration Profiles

### Development Profile (dev.toml)
```toml
[engine]
log_level = "debug"

[parser]
parallel_workers = 2
timeout_seconds = 10

[file_watcher]
debounce_ms = 50

[api]
enable_cors = true
cors_origins = ["http://localhost:3000"]

[logging]
level = "debug"
format = "pretty"
```

### Production Profile (prod.toml)
```toml
[engine]
log_level = "info"

[parser]
parallel_workers = 16
timeout_seconds = 60

[file_watcher]
debounce_ms = 500
batch_threshold = 200

[api]
enable_cors = false
api_key_required = true

[monitoring]
enabled = true
prometheus_port = 9090

[logging]
level = "info"
format = "json"
output = "file"
file_path = "/var/log/ccg/app.log"
```

### Testing Profile (test.toml)
```toml
[engine]
log_level = "warn"

[falkordb]
database = 15  # Use separate test database

[cas]
storage_path = "./test_cas"

[file_watcher]
enabled = false

[versioning]
max_versions = 10

[logging]
level = "warn"
output = "stdout"
```

## Dynamic Configuration

### Hot Reload

Some configuration can be updated without restart:

```rust
pub struct ConfigManager {
    config: Arc<RwLock<Config>>,
    watchers: Vec<notify::RecommendedWatcher>,
}

impl ConfigManager {
    pub async fn reload(&self) -> Result<(), ConfigError> {
        let new_config = Config::load()?;
        
        // Only allow hot-reloadable changes
        let mut config = self.config.write().await;
        config.logging = new_config.logging;
        config.api.rate_limiting = new_config.api.rate_limiting;
        config.monitoring = new_config.monitoring;
        
        info!("Configuration reloaded successfully");
        Ok(())
    }
    
    pub fn get_config(&self) -> Arc<RwLock<Config>> {
        Arc::clone(&self.config)
    }
}
```

### Remote Configuration

Configuration can be fetched from remote sources:

```toml
[remote_config]
enabled = true
url = "https://config.example.com/ccg/config.toml"
polling_interval_seconds = 300
auth_token = "${CONFIG_AUTH_TOKEN}"
```

## Configuration Examples

### Minimal Configuration
```toml
[engine]
name = "my-project"
root_path = "./src"

[falkordb]
url = "redis://localhost:6379"
```

### Enterprise Configuration
```toml
[engine]
name = "enterprise-analyzer"
languages = ["java", "kotlin", "javascript", "typescript"]
root_path = "/app/enterprise-codebase"

[parser]
max_file_size_kb = 5120
parallel_workers = 32
timeout_seconds = 120

[falkordb]
url = "redis://redis-cluster.internal:6379"
connection_pool_size = 50
enable_tls = true
username = "ccg-service"
password = "${REDIS_PASSWORD}"

[cas]
storage_path = "/data/cas"
compression = "zstd"
compression_level = 9
max_content_size_mb = 500

[api]
host = "0.0.0.0"
port = 8080
api_key_required = true
api_key = "${CCG_API_KEY}"

[api.rate_limiting]
enabled = true
requests_per_minute = 500
key_strategy = "api_key"

[monitoring]
enabled = true
prometheus_port = 9090
health_check_port = 8081

[security]
enable_tls = true
cert_file = "/etc/ssl/certs/ccg.crt"
key_file = "/etc/ssl/private/ccg.key"
allowed_hosts = ["ccg.company.com"]

[logging]
level = "info"
format = "json"
output = "file"
file_path = "/var/log/ccg/app.log"
rotation = "daily"
```

### Multi-Language Configuration
```toml
[engine]
languages = ["python", "javascript", "java", "kotlin", "rust", "go"]

[parser.language_specific.python]
ignore_patterns = ["*_test.py", "test_*.py", "**/tests/**"]
max_complexity = 15

[parser.language_specific.javascript]
ignore_patterns = ["*.min.js", "*.bundle.js", "node_modules/**"]
parse_jsx = true
parse_typescript = true

[parser.language_specific.java]
source_version = "11"
ignore_patterns = ["**/target/**", "**/*Test.java"]

[parser.language_specific.kotlin]
ignore_patterns = ["**/build/**", "**/*Test.kt"]
experimental_features = true
```

## Troubleshooting Configuration

### Common Issues

1. **File Permission Errors**
```bash
# Ensure storage directories are writable
chmod -R 755 ./cas_store
chmod -R 755 ./context
```

2. **Database Connection Issues**
```bash
# Test FalkorDB connection
redis-cli -u redis://localhost:6379 ping
```

3. **Memory Issues with Large Codebases**
```toml
[parser]
max_file_size_kb = 512  # Reduce max file size
parallel_workers = 4    # Reduce parallel workers

[cas]
max_content_size_mb = 50  # Reduce CAS cache size
```

4. **Performance Tuning**
```toml
[file_watcher]
debounce_ms = 1000      # Increase debounce for busy filesystems
batch_threshold = 500   # Higher batch size for bulk operations

[versioning]
merkle_fanout = 32      # Increase fanout for better performance
compression_enabled = false  # Disable compression for speed
```

### Configuration Validation Tool

```bash
# Validate configuration
ccg config validate --config config.toml

# Show effective configuration
ccg config show --profile production

# Test configuration with dry-run
ccg analyze --dry-run --config config.toml ./src
```