use clap::{Parser, Subcommand};
use code_context_graph_core::{Config, Result, SnapshotMeta, FileEntry};
use std::path::{Path, PathBuf};
use tracing::{info, Level};
use tracing_subscriber::util::SubscriberInitExt;
use std::net::SocketAddr;
use code_context_graph_api as api;
use tokio::net::TcpListener;
use axum::serve;
use code_context_graph_storage::cas::{CasConfig, CasStore};
use std::io;
use code_context_graph_storage::merkle::MerkleBuilder;
use std::fs;
use serde_json;
use code_context_graph_parser::language::{LanguageDetector, ParserRegistry};
use code_context_graph_graph::{GraphBuilder, GraphClient, GraphExecutor};
use code_context_graph_viz::mermaid::ClassDiagramExporter;
use std::env;
use serde::Deserialize;

#[derive(Parser)]
#[command(name = "ccg")]
#[command(about = "Code Context Graph - A multi-language code analysis tool with semantic graph generation")]
#[command(version)]
pub struct Cli {
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum VizCommands {
    #[command(about = "Generate Mermaid class diagram from source path")]
    Class {
        #[arg(long, value_name = "PATH")]
        path: PathBuf,
        #[arg(long, value_name = "FILE")] 
        out: PathBuf,
        #[arg(long, value_name = "FORMAT", default_value = "md")]
        format: String,
        #[arg(long = "filter-class", value_name = "CLASS", value_delimiter = ',')]
        filter_class: Option<Vec<String>>,
    },
}

// Allow minimal config files by merging provided TOML over defaults
fn load_config_with_defaults(path: &PathBuf) -> Result<Config> {
    let content = std::fs::read_to_string(path)?;
    let partial: PartialConfig = toml::from_str(&content)
        .map_err(|e| code_context_graph_core::CodeGraphError::Config { message: format!("Failed to parse config: {}", e) })?;
    let mut cfg = Config::default();
    if let Some(engine) = partial.engine {
        if let Some(name) = engine.name { cfg.engine.name = name; }
        if let Some(langs) = engine.languages { cfg.engine.languages = langs; }
    }
    if let Some(parser) = partial.parser {
        if let Some(max_kb) = parser.max_file_size_kb { cfg.parser.max_file_size_kb = max_kb; }
        if let Some(ign) = parser.ignore_patterns { cfg.parser.ignore_patterns = ign; }
    }
    if let Some(falkor) = partial.falkordb {
        if let Some(url) = falkor.url { cfg.falkordb.url = url; }
        if let Some(gn) = falkor.graph_name { cfg.falkordb.graph_name = gn; }
    }
    if let Some(cas) = partial.cas {
        if let Some(enabled) = cas.enabled { cfg.cas.enabled = enabled; }
        if let Some(storage_path) = cas.storage_path { cfg.cas.storage_path = storage_path; }
        if let Some(hash) = cas.hash_algorithm { cfg.cas.hash_algorithm = hash; }
        if let Some(comp) = cas.compression { cfg.cas.compression = comp; }
        if let Some(dedup) = cas.dedup_threshold { cfg.cas.dedup_threshold = dedup; }
    }
    if let Some(ver) = partial.versioning {
        if let Some(enabled) = ver.enabled { cfg.versioning.enabled = enabled; }
        if let Some(max_versions) = ver.max_versions { cfg.versioning.max_versions = max_versions; }
        if let Some(auto) = ver.auto_snapshot_interval { cfg.versioning.auto_snapshot_interval = auto; }
        if let Some(fanout) = ver.merkle_tree_fanout { cfg.versioning.merkle_tree_fanout = fanout; }
    }
    if let Some(log) = partial.logging {
        if let Some(level) = log.level { cfg.logging.level = level; }
        if let Some(format) = log.format { cfg.logging.format = format; }
    }
    if let Some(api) = partial.api {
        if let Some(port) = api.port { cfg.api.port = port; }
        if let Some(mcs) = api.max_context_size { cfg.api.max_context_size = mcs; }
        if let Some(v) = api.enable_version_api { cfg.api.enable_version_api = v; }
        if let Some(v) = api.enable_quality_api { cfg.api.enable_quality_api = v; }
        if let Some(v) = api.enable_aase_api { cfg.api.enable_aase_api = v; }
    }
    if let Some(qm) = partial.quality_metrics {
        if let Some(v) = qm.calculate_cohesion { cfg.quality_metrics.calculate_cohesion = v; }
        if let Some(v) = qm.calculate_coupling { cfg.quality_metrics.calculate_coupling = v; }
        if let Some(v) = qm.maintainability_threshold { cfg.quality_metrics.maintainability_threshold = v; }
        if let Some(v) = qm.complexity_warning { cfg.quality_metrics.complexity_warning = v; }
    }
    if let Some(fw) = partial.file_watcher {
        if let Some(v) = fw.enabled { cfg.file_watcher.enabled = v; }
        if let Some(v) = fw.debounce_ms { cfg.file_watcher.debounce_ms = v; }
        if let Some(v) = fw.batch_threshold { cfg.file_watcher.batch_threshold = v; }
        if let Some(v) = fw.ignore_patterns { cfg.file_watcher.ignore_patterns = v; }
        if let Some(v) = fw.recursive { cfg.file_watcher.recursive = v; }
    }
    if let Some(aase) = partial.aase {
        if let Some(v) = aase.enabled { cfg.aase.enabled = v; }
        if let Some(v) = aase.context_path { cfg.aase.context_path = v; }
        if let Some(v) = aase.naming_convention { cfg.aase.naming_convention = v; }
        if let Some(v) = aase.auto_propagate { cfg.aase.auto_propagate = v; }
        if let Some(v) = aase.human_review_threshold { cfg.aase.human_review_threshold = v; }
        if let Some(v) = aase.artifact_versioning { cfg.aase.artifact_versioning = v; }
        if let Some(v) = aase.context_chain_depth { cfg.aase.context_chain_depth = v; }
    }
    if let Some(con) = partial.connascence {
        if let Some(v) = con.enabled { cfg.connascence.enabled = v; }
        if let Some(v) = con.detect_static { cfg.connascence.detect_static = v; }
        if let Some(v) = con.detect_dynamic { cfg.connascence.detect_dynamic = v; }
        if let Some(v) = con.strength_threshold { cfg.connascence.strength_threshold = v; }
        if let Some(v) = con.auto_suggest_refactoring { cfg.connascence.auto_suggest_refactoring = v; }
    }
    Ok(cfg)
}

#[derive(Debug, Deserialize)]
struct PartialConfig {
    engine: Option<PartialEngine>,
    parser: Option<PartialParser>,
    falkordb: Option<PartialFalkor>,
    cas: Option<PartialCas>,
    file_watcher: Option<PartialFileWatcher>,
    versioning: Option<PartialVersioning>,
    connascence: Option<PartialConnascence>,
    aase: Option<PartialAase>,
    quality_metrics: Option<PartialQualityMetrics>,
    api: Option<PartialApi>,
    logging: Option<PartialLogging>,
}

#[derive(Debug, Deserialize)]
struct PartialEngine { name: Option<String>, languages: Option<Vec<String>> }
#[derive(Debug, Deserialize)]
struct PartialParser { max_file_size_kb: Option<usize>, ignore_patterns: Option<Vec<String>> }
#[derive(Debug, Deserialize)]
struct PartialFalkor { url: Option<String>, graph_name: Option<String> }
#[derive(Debug, Deserialize)]
struct PartialCas { enabled: Option<bool>, storage_path: Option<PathBuf>, hash_algorithm: Option<String>, compression: Option<String>, dedup_threshold: Option<f32> }
#[derive(Debug, Deserialize)]
struct PartialFileWatcher { enabled: Option<bool>, debounce_ms: Option<u64>, batch_threshold: Option<usize>, ignore_patterns: Option<Vec<String>>, recursive: Option<bool> }
#[derive(Debug, Deserialize)]
struct PartialVersioning { enabled: Option<bool>, max_versions: Option<usize>, auto_snapshot_interval: Option<u64>, merkle_tree_fanout: Option<usize> }
#[derive(Debug, Deserialize)]
struct PartialConnascence { enabled: Option<bool>, detect_static: Option<bool>, detect_dynamic: Option<bool>, strength_threshold: Option<f32>, auto_suggest_refactoring: Option<bool> }
#[derive(Debug, Deserialize)]
struct PartialAase { enabled: Option<bool>, context_path: Option<PathBuf>, naming_convention: Option<String>, auto_propagate: Option<bool>, human_review_threshold: Option<f32>, artifact_versioning: Option<bool>, context_chain_depth: Option<usize> }
#[derive(Debug, Deserialize)]
struct PartialQualityMetrics { calculate_cohesion: Option<bool>, calculate_coupling: Option<bool>, maintainability_threshold: Option<u32>, complexity_warning: Option<u32> }
#[derive(Debug, Deserialize)]
struct PartialApi { port: Option<u16>, max_context_size: Option<usize>, enable_version_api: Option<bool>, enable_quality_api: Option<bool>, enable_aase_api: Option<bool> }
#[derive(Debug, Deserialize)]
struct PartialLogging { level: Option<String>, format: Option<String> }

#[derive(Subcommand)]
pub enum VersionCommands {
    #[command(about = "List snapshots")]
    List {
        #[arg(short, long, value_name = "PATH")]
        path: PathBuf,
        #[arg(short, long)]
        limit: Option<usize>,
    },
    #[command(about = "Show snapshot metadata")]
    Show {
        #[arg(short, long, value_name = "PATH")]
        path: PathBuf,
        #[arg(long, value_name = "ID")]
        id: String,
    },
    #[command(about = "Diff two snapshots")]
    Diff {
        #[arg(short, long, value_name = "PATH")]
        path: PathBuf,
        #[arg(long, value_name = "ID")]
        from: String,
        #[arg(long, value_name = "ID")]
        to: String,
    },
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Analyze a codebase and generate semantic graph")]
    Analyze {
        #[arg(short, long, value_name = "PATH")]
        path: PathBuf,
        
        #[arg(short, long)]
        languages: Option<Vec<String>>,
        
        #[arg(long, value_name = "MESSAGE")]
        message: Option<String>,
    },
    
    #[command(about = "Versioning: list and show snapshots")]
    Version {
        #[command(subcommand)]
        sub: VersionCommands,
    },
    
    #[command(about = "Start file watcher for real-time updates")]
    Watch {
        #[arg(short, long, value_name = "PATH")]
        path: PathBuf,
    },
    
    #[command(about = "Query the semantic graph")]
    Query {
        #[arg(short, long)]
        question: String,
        
        #[arg(long)]
        max_hops: Option<usize>,
    },
    
    #[command(about = "Show version history")]
    History {
        #[arg(short, long, value_name = "PATH")]
        path: Option<PathBuf>,
        
        #[arg(short, long)]
        limit: Option<usize>,
    },
    
    #[command(about = "Compare two versions")]
    Diff {
        #[arg(long)]
        from: String,
        
        #[arg(long)]
        to: String,
    },
    
    #[command(about = "Analyze connascence patterns")]
    Connascence {
        #[arg(short, long, value_name = "PATH")]
        path: PathBuf,
        
        #[arg(long)]
        min_strength: Option<f32>,
        
        #[arg(long)]
        conn_types: Option<Vec<String>>,
    },
    
    #[command(about = "Generate AASE context artifacts")]
    Aase {
        #[command(subcommand)]
        aase_command: AaseCommands,
    },
    
    #[command(about = "Show quality metrics")]
    Quality {
        #[arg(short, long, value_name = "PATH")]
        path: PathBuf,
        
        #[arg(short, long)]
        module: Option<String>,
        
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    
    #[command(about = "Start API server")]
    Serve {
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },

    #[command(about = "Visualize graphs (Mermaid)")]
    Viz {
        #[command(subcommand)]
        viz: VizCommands,
    },
}

#[derive(Subcommand)]
pub enum AaseCommands {
    #[command(about = "Generate context artifact for domain")]
    Generate {
        #[arg(short, long)]
        domain: String,
        
        #[arg(short, long, default_value = "Context")]
        artifact_type: String,
    },
    
    #[command(about = "Show context chain for domain")]
    Chain {
        #[arg(short, long)]
        domain: String,
    },
    
    #[command(about = "List all context artifacts")]
    List,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Load configuration
    let (config, config_is_default) = if let Some(config_path) = cli.config {
        (load_config_with_defaults(&config_path)?, false)
    } else {
        // Try to load from default locations
        let default_paths = vec![
            PathBuf::from("config.toml"),
            PathBuf::from("ccg.toml"),
            dirs::config_dir().unwrap_or_default().join("ccg").join("config.toml"),
        ];
        
        let mut config = None;
        for path in default_paths {
            if path.exists() {
                info!("Loading config from: {}", path.display());
                config = Some(load_config_with_defaults(&path)?);
                break;
            }
        }
        
        match config {
            Some(c) => (c, false),
            None => {
                info!("Using default configuration");
                (Config::default(), true)
            }
        }
    };
    
    // Initialize tracing according to config
    init_tracing(&config);
    
    match cli.command {
        Commands::Analyze { path, languages, message } => {
            info!("Starting analysis of: {}", path.display());
            analyze_command(config, path, languages, message, config_is_default).await
        }
        Commands::Watch { path } => {
            info!("Starting file watcher for: {}", path.display());
            watch_command(config, path).await
        }
        Commands::Query { question, max_hops } => {
            info!("Querying graph: {}", question);
            query_command(config, question, max_hops).await
        }
        Commands::History { path, limit } => {
            info!("Showing version history");
            history_command(config, path, limit).await
        }
        Commands::Diff { from, to } => {
            info!("Comparing versions: {} -> {}", from, to);
            diff_command(config, from, to).await
        }
        Commands::Connascence { path, min_strength, conn_types } => {
            info!("Analyzing connascence patterns in: {}", path.display());
            connascence_command(config, path, min_strength, conn_types).await
        }
        Commands::Aase { aase_command } => {
            info!("AASE command");
            aase_command_handler(config, aase_command).await
        }
        Commands::Version { sub } => {
            version_command(config, sub).await
        }
        Commands::Quality { path, module, format } => {
            info!("Calculating quality metrics for: {}", path.display());
            quality_command(config, path, module, format).await
        }
        Commands::Serve { port } => {
            info!("Starting API server on port: {}", port);
            serve_command(config, port).await
        }
        Commands::Viz { viz } => {
            viz_command(config, viz).await
        }
    }
}

async fn analyze_command(config: Config, path: PathBuf, languages: Option<Vec<String>>, message: Option<String>, config_is_default: bool) -> Result<()> {
    println!("🔍 Analyzing codebase at: {}", path.display());

    // Resolve CAS root from config (relative to repo path if relative)
    let cas_root = if config_is_default {
        // Preserve historical default for workspace under .ccg/cas
        path.join(".ccg").join("cas")
    } else if config.cas.storage_path.is_relative() {
        path.join(&config.cas.storage_path)
    } else {
        config.cas.storage_path.clone()
    };
    // Workspace directory: when using default config, always use the conventional `.ccg` dir
    // so that version commands can find snapshots. Otherwise, derive from configured CAS path.
    let ws_dir = if config_is_default {
        path.join(".ccg")
    } else {
        resolve_workspace_dir(&config, &path)
    };
    let cas = CasStore::new(code_context_graph_storage::cas::CasConfig { root: cas_root.clone() })
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
    let mut builder = MerkleBuilder::new();
    // Initialize parser registry and graph components
    let parser_registry = ParserRegistry::new();
    let graph_builder = GraphBuilder::new(&config.falkordb.graph_name);
    // Test hook: if CCG_GRAPH_TEST_RECORD is set, write queries to that file instead of connecting to Redis
    struct FileExec { path: PathBuf }
    impl GraphExecutor for FileExec {
        fn query(&self, _graph: &str, cypher: &str) -> anyhow::Result<redis::Value> {
            use std::io::Write;
            let mut f = fs::OpenOptions::new().create(true).append(true).open(&self.path)?;
            writeln!(f, "{}", cypher)?;
            Ok(redis::Value::Okay)
        }
    }
    let graph_client = if let Ok(p) = env::var("CCG_GRAPH_TEST_RECORD") {
        GraphClient::with_executor(&config.falkordb.graph_name, Box::new(FileExec { path: PathBuf::from(p) }))
    } else {
        GraphClient::new_with_redis(&config.falkordb.url, &config.falkordb.graph_name)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?
    };

    // Very simple fallback when language-specific parser is unavailable: extract
    // function names and import modules with string scanning to build minimal queries.
    fn basic_queries_from_source(src: &str, file_path: &str) -> Vec<String> {
        let mut q = Vec::new();
        q.push(format!("MERGE (f:File {{ path: '{}' }})", file_path.replace('\\', "/")));
        // functions: look for "def name("
        let mut i = 0usize;
        let bytes = src.as_bytes();
        while let Some(pos) = src[i..].find("def ") {
            let start = i + pos + 4;
            // read identifier
            let rest = &src[start..];
            let mut name = String::new();
            for ch in rest.chars() {
                if ch == '(' || ch == ' ' || ch == ':' { break; }
                name.push(ch);
            }
            if !name.is_empty() {
                q.push(format!("MERGE (fn:Function {{ name: '{}' }})", name.replace("'", "\\'")));
                q.push("MERGE (f)-[:CONTAINS]->(fn)".to_string());
            }
            i = start;
        }
        // imports: handle "import mod" and "from mod import"
        for line in src.split(|c| c == '\n' || c == '\r') {
            let l = line.trim();
            if let Some(rest) = l.strip_prefix("import ") {
                let mod_name = rest.split(|c: char| c.is_whitespace() || c == ',' ).next().unwrap_or("");
                if !mod_name.is_empty() {
                    q.push(format!("MERGE (m:Module {{ name: '{}' }})", mod_name.replace("'", "\\'")));
                    q.push("MERGE (f)-[:IMPORTS]->(m)".to_string());
                }
            } else if let Some(rest) = l.strip_prefix("from ") {
                let mod_name = rest.split_whitespace().next().unwrap_or("");
                if !mod_name.is_empty() {
                    q.push(format!("MERGE (m:Module {{ name: '{}' }})", mod_name.replace("'", "\\'")));
                    q.push("MERGE (f)-[:IMPORTS]->(m)".to_string());
                }
            }
        }
        q
    }

    if path.is_file() {
        if let Ok(bytes) = fs::read(&path) {
            // Respect max file size
            if bytes.len() <= config.parser.max_file_size_kb * 1024 {
                let mut files_indexed: usize = 0;
                let mut total_bytes: u64 = 0;
                let mut files_meta: Vec<FileEntry> = Vec::new();
                total_bytes += bytes.len() as u64;
                files_indexed += 1;
                builder.add(path_to_unix(&path), &bytes);
                // Parse and persist to graph (with fallback)
                let lang = LanguageDetector::detect_from_path(&path);
                if let Ok(src) = std::str::from_utf8(&bytes) {
                    let mut persisted = false;
                    if parser_registry.supports_language(&lang) {
                        match parser_registry.parse(src, lang) {
                            Ok(ast) => {
                                let mut queries = graph_builder.build_queries(&ast, &path_to_unix(&path));
                                let has_fn = queries.iter().any(|q| q.contains("(fn:Function"));
                                let has_mod = queries.iter().any(|q| q.contains("(m:Module"));
                                if !has_fn || !has_mod {
                                    let fb = basic_queries_from_source(src, &path_to_unix(&path));
                                    // Simpler: extend all fallback queries; MERGE is idempotent and the file alias is consistent
                                    queries.extend(fb);
                                }
                                let _ = graph_client.persist_queries(&queries);
                                persisted = true;
                            }
                            Err(_) => { /* fall through to fallback */ }
                        }
                    }
                    if !persisted {
                        let queries = basic_queries_from_source(src, &path_to_unix(&path));
                        let _ = graph_client.persist_queries(&queries);
                    }
                }
                // Store into CAS
                match cas.put_bytes(&bytes) {
                    Ok(h) => files_meta.push(FileEntry { path: path_to_unix(&path), hash: h }),
                    Err(_) => {}
                }
                let merkle = builder.build();
                println!("Indexed files: {}", files_indexed);
                println!("Total bytes: {}", total_bytes);
                println!("root: {}", merkle.root());
                // Persist snapshot metadata
                let snapshots_dir = ws_dir.join("snapshots");
                let _ = fs::create_dir_all(&snapshots_dir);
                let meta = SnapshotMeta::with_files(merkle.root().to_string(), files_indexed, total_bytes, files_meta, message);
                let meta_path = snapshots_dir.join(format!("{}.json", merkle.root()));
                if let Ok(json) = serde_json::to_string_pretty(&meta) {
                    let _ = fs::write(meta_path, json);
                }
                println!("✅ Initialized storage workspace");
                return Ok(())
            }
        }
    }

    let mut files_indexed: usize = 0;
    let mut total_bytes: u64 = 0;
    let mut files_meta: Vec<FileEntry> = Vec::new();
    let ignore_patterns = config.parser.ignore_patterns.clone();
    let max_size_bytes: u64 = (config.parser.max_file_size_kb as u64) * 1024;
    let lang_allow: Option<Vec<String>> = languages.or_else(|| Some(config.engine.languages.clone()));

    // Simple stack-based DFS to avoid extra deps
    let mut stack: Vec<PathBuf> = vec![path.clone()];
    while let Some(dir) = stack.pop() {
        let entries = match fs::read_dir(&dir) {
            Ok(e) => e,
            Err(err) => {
                eprintln!("Skipping unreadable dir {}: {}", dir.display(), err);
                continue;
            }
        };
        for entry in entries.flatten() {
            let p = entry.path();
            let file_name = entry.file_name();
            if file_name == ".ccg" { continue; }
            // Ignore directories by pattern (e.g., .git)
            if let Some(name) = file_name.to_str() {
                if should_ignore_name(name, &ignore_patterns) { continue; }
            }
            let md = match entry.metadata() { Ok(m) => m, Err(_) => continue };
            if md.is_dir() {
                stack.push(p);
                continue;
            }
            if md.is_file() {
                // Skip files exceeding max size limit
                if md.len() > max_size_bytes { continue; }
                // Read file bytes
                match fs::read(&p) {
                    Ok(bytes) => {
                        // Ignore files by pattern (e.g., *.min.js)
                        if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                            if should_ignore_name(name, &ignore_patterns) { continue; }
                        }
                        // Filter by allowed languages/extensions if provided
                        if let Some(ref allow) = lang_allow {
                            if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                                if !is_allowed_extension(ext, allow) { continue; }
                            } else {
                                continue;
                            }
                        }
                        total_bytes += bytes.len() as u64;
                        files_indexed += 1;
                        // Relative path (unix separators)
                        let rel_str = if let Ok(rel) = p.strip_prefix(&path) {
                            path_to_unix(rel)
                        } else {
                            // Fallback for cases like path "." where DirEntry gives absolute paths
                            path_to_unix(&p)
                        };
                        builder.add(rel_str.clone(), &bytes);
                            // Parse and persist to graph
                            let lang = LanguageDetector::detect_from_path(&p);
                            if let Ok(src) = std::str::from_utf8(&bytes) {
                                let mut persisted = false;
                                if parser_registry.supports_language(&lang) {
                                    match parser_registry.parse(src, lang) {
                                        Ok(ast) => {
                                            let mut queries = graph_builder.build_queries(&ast, &rel_str);
                                            let has_fn = queries.iter().any(|q| q.contains("(fn:Function"));
                                            let has_mod = queries.iter().any(|q| q.contains("(m:Module"));
                                            if !has_fn || !has_mod {
                                                let fb = basic_queries_from_source(src, &rel_str);
                                                queries.extend(fb);
                                            }
                                            let _ = graph_client.persist_queries(&queries);
                                            persisted = true;
                                        }
                                        Err(_) => { /* fallthrough */ }
                                    }
                                }
                                if !persisted {
                                    let queries = basic_queries_from_source(src, &rel_str);
                                    let _ = graph_client.persist_queries(&queries);
                                }
                            }
                            // Store into CAS and record file entry
                            match cas.put_bytes(&bytes) {
                                Ok(h) => files_meta.push(FileEntry { path: rel_str, hash: h }),
                                Err(_) => {}
                            }
                    },
                    Err(_) => {
                        // skip unreadable file
                    }
                }
            }
        }
    }

    let merkle = builder.build();
    println!("Indexed files: {}", files_indexed);
    println!("Total bytes: {}", total_bytes);
    println!("root: {}", merkle.root());
    // Persist snapshot metadata
    let snapshots_dir = ws_dir.join("snapshots");
    let _ = fs::create_dir_all(&snapshots_dir);
    let meta = SnapshotMeta::with_files(merkle.root().to_string(), files_indexed, total_bytes, files_meta, message);
    let meta_path = snapshots_dir.join(format!("{}.json", merkle.root()));
    if let Ok(json) = serde_json::to_string_pretty(&meta) {
        let _ = fs::write(meta_path, json);
    }
    println!("✅ Initialized storage workspace");
    Ok(())
}

fn resolve_workspace_dir(config: &Config, path: &Path) -> PathBuf {
    // Default workspace under repo
    let default_ws = path.join(".ccg");
    let default_cas = default_ws.join("cas");
    if default_cas.exists() {
        return default_ws;
    }
    // Otherwise derive from cas.storage_path
    let configured = if config.cas.storage_path.is_relative() {
        path.join(&config.cas.storage_path)
    } else {
        config.cas.storage_path.clone()
    };
    // If configured path points to a directory named "cas", treat its parent as workspace.
    // Otherwise, treat the configured path itself as the workspace root.
    if configured.file_name().and_then(|s| s.to_str()) == Some("cas") {
        configured.parent().map(|p| p.to_path_buf()).unwrap_or(default_ws)
    } else {
        configured
    }
}

async fn version_command(config: Config, sub: VersionCommands) -> Result<()> {
    match sub {
        VersionCommands::List { path, limit } => {
            let ws = resolve_workspace_dir(&config, &path);
            let dir = ws.join("snapshots");
            let mut entries: Vec<SnapshotMeta> = match fs::read_dir(&dir) {
                Ok(rd) => rd.filter_map(|e| e.ok()).filter_map(|e| {
                    let p = e.path();
                    if p.extension().and_then(|s| s.to_str()) == Some("json") {
                        fs::read_to_string(&p).ok()
                            .and_then(|s| serde_json::from_str::<SnapshotMeta>(&s).ok())
                    } else { None }
                }).collect(),
                Err(_) => Vec::new(),
            };
            entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            let count = limit.unwrap_or(entries.len());
            for meta in entries.into_iter().take(count) {
                println!("{} {} files {} bytes", meta.root, meta.total_files, meta.total_bytes);
            }
        }
        VersionCommands::Show { path, id } => {
            let ws = resolve_workspace_dir(&config, &path);
            let p = ws.join("snapshots").join(format!("{}.json", id));
            match fs::read_to_string(&p) {
                Ok(s) => match serde_json::from_str::<SnapshotMeta>(&s) {
                    Ok(meta) => {
                        println!("root: {}", meta.root);
                        println!("total_files: {}", meta.total_files);
                        println!("total_bytes: {}", meta.total_bytes);
                        println!("timestamp: {}", meta.timestamp);
                        if let Some(u) = meta.user { println!("user: {}", u); }
                        if let Some(m) = meta.message { println!("message: {}", m); }
                    }
                    Err(_) => println!("⚠️  Failed to parse snapshot metadata"),
                },
                Err(_) => println!("⚠️  Snapshot not found: {}", id),
            }
        }
        VersionCommands::Diff { path, from, to } => {
            let ws = resolve_workspace_dir(&config, &path);
            let dir = ws.join("snapshots");
            let p1 = dir.join(format!("{}.json", from));
            let p2 = dir.join(format!("{}.json", to));
            let s1 = fs::read_to_string(&p1).unwrap_or_default();
            let s2 = fs::read_to_string(&p2).unwrap_or_default();
            let m1: SnapshotMeta = serde_json::from_str(&s1).unwrap_or(SnapshotMeta::new(from, 0, 0, None));
            let m2: SnapshotMeta = serde_json::from_str(&s2).unwrap_or(SnapshotMeta::new(to, 0, 0, None));
            use std::collections::{HashMap, HashSet};
            let map1: HashMap<_, _> = m1.files.iter().map(|f| (f.path.clone(), f.hash.clone())).collect();
            let map2: HashMap<_, _> = m2.files.iter().map(|f| (f.path.clone(), f.hash.clone())).collect();
            let set1: HashSet<_> = map1.keys().cloned().collect();
            let set2: HashSet<_> = map2.keys().cloned().collect();
            let added: Vec<_> = set2.difference(&set1).cloned().collect();
            let removed: Vec<_> = set1.difference(&set2).cloned().collect();
            let changed: Vec<_> = set1.intersection(&set2)
                .filter(|p| map1.get(*p) != map2.get(*p))
                .cloned().collect();
            println!("Added:");
            for p in added { println!("{}", p); }
            println!("Removed:");
            for p in removed { println!("{}", p); }
            println!("Changed:");
            for p in changed { println!("{}", p); }
        }
    }
    Ok(())
}

fn path_to_unix(p: &Path) -> String {
    let s = p.to_string_lossy().to_string();
    if std::path::MAIN_SEPARATOR == '/' { s } else { s.replace('\\', "/") }
}

fn should_ignore_name(name: &str, patterns: &[String]) -> bool {
    for pat in patterns {
        let p = pat.trim();
        if p.is_empty() { continue; }
        if p.starts_with("*.") {
            let suffix = &p[1..]; // includes '.'
            if name.ends_with(suffix) { return true; }
        } else if p.starts_with('*') {
            let suffix = &p[1..];
            if name.ends_with(suffix) { return true; }
        } else {
            if name == p { return true; }
        }
    }
    false
}

fn init_tracing(config: &Config) {
    let level = match config.logging.level.to_lowercase().as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(level)
        .with_ansi(config.logging.format.to_lowercase() != "json")
        .with_target(true)
        .with_thread_names(false)
        .with_line_number(true)
        .with_file(true)
        .with_writer(std::io::stderr);
    subscriber.init();
}

fn is_allowed_extension(ext: &str, allowed_langs: &[String]) -> bool {
    for l in allowed_langs {
        match l.to_lowercase().as_str() {
            "python" => if ext.eq_ignore_ascii_case("py") { return true; },
            "javascript" | "js" => if ext.eq_ignore_ascii_case("js") { return true; },
            "java" => if ext.eq_ignore_ascii_case("java") { return true; },
            "kotlin" => if ext.eq_ignore_ascii_case("kt") || ext.eq_ignore_ascii_case("kts") { return true; },
            _ => {}
        }
    }
    false
}

async fn watch_command(_config: Config, path: PathBuf) -> Result<()> {
    println!("👁️  Starting file watcher for: {}", path.display());
    println!("⚠️  File watching functionality not yet implemented");
    Ok(())
}

async fn query_command(_config: Config, question: String, _max_hops: Option<usize>) -> Result<()> {
    println!("❓ Querying: {}", question);
    println!("⚠️  Query functionality not yet implemented");
    Ok(())
}

async fn history_command(_config: Config, _path: Option<PathBuf>, _limit: Option<usize>) -> Result<()> {
    println!("📜 Version history:");
    println!("⚠️  History functionality not yet implemented");
    Ok(())
}

async fn diff_command(_config: Config, from: String, to: String) -> Result<()> {
    println!("🔄 Comparing {} -> {}", from, to);
    println!("⚠️  Diff functionality not yet implemented");
    Ok(())
}

async fn connascence_command(_config: Config, path: PathBuf, _min_strength: Option<f32>, _conn_types: Option<Vec<String>>) -> Result<()> {
    println!("🔗 Analyzing connascence in: {}", path.display());
    println!("⚠️  Connascence analysis not yet implemented");
    Ok(())
}

async fn aase_command_handler(_config: Config, command: AaseCommands) -> Result<()> {
    match command {
        AaseCommands::Generate { domain, artifact_type } => {
            println!("🎯 Generating {} artifact for domain: {}", artifact_type, domain);
            println!("⚠️  AASE generation not yet implemented");
        }
        AaseCommands::Chain { domain } => {
            println!("🔗 Context chain for domain: {}", domain);
            println!("⚠️  AASE chain functionality not yet implemented");
        }
        AaseCommands::List => {
            println!("📋 Context artifacts:");
            println!("⚠️  AASE listing not yet implemented");
        }
    }
    Ok(())
}

async fn quality_command(_config: Config, path: PathBuf, _module: Option<String>, _format: String) -> Result<()> {
    println!("📊 Quality metrics for: {}", path.display());
    println!("⚠️  Quality metrics not yet implemented");
    Ok(())
}

async fn serve_command(_config: Config, port: u16) -> Result<()> {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!(%addr, "Starting API server");

    let app = api::router();

    let listener = TcpListener::bind(addr).await?;
    info!(%addr, "Listening");
    serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
            info!("Shutdown signal received");
        })
        .await?;
    Ok(())
}

async fn viz_command(_config: Config, viz: VizCommands) -> Result<()> {
    match viz {
        VizCommands::Class { path, out, format, filter_class } => {
            let registry = ParserRegistry::new();
            // Helper to write output given a mermaid diagram string
            let write_output = |mermaid: String| -> Result<()> {
                match format.to_lowercase().as_str() {
                    "html" => {
                        let html = format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>CCG Mermaid Diagram</title>
  <style>body{{font-family:sans-serif;margin:0;padding:16px;background:#0b0e14;color:#e6e1cf}}</style>
  <script src="https://cdn.jsdelivr.net/npm/mermaid@10/dist/mermaid.min.js"></script>
  <script>mermaid.initialize({{ startOnLoad: true, theme: 'dark' }});</script>
  </head>
<body>
  <div class="mermaid">{diagram}</div>
</body>
</html>"#, diagram = mermaid);
                        std::fs::write(&out, html)?;
                    }
                    _ => {
                        std::fs::write(&out, mermaid)?;
                    }
                }
                Ok(())
            };

            let filter_ref = filter_class.as_ref().map(|v| v.as_slice());

            if path.is_dir() {
                // Parse all supported files under directory
                let mut asts: Vec<code_context_graph_parser::ast::SimplifiedAST> = Vec::new();
                for entry in walkdir::WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
                    let p = entry.path();
                    if p.is_file() {
                        let lang = LanguageDetector::detect_from_path(p);
                        if registry.supports_language(&lang) {
                            if let Ok(src) = std::fs::read_to_string(p) {
                                if let Ok(ast) = registry.parse(&src, lang) {
                                    asts.push(ast);
                                }
                            }
                        }
                    }
                }
                // Merge: exporter over multiple ASTs
                let mut mermaid = String::from("classDiagram\n");
                for ast in &asts {
                    let part = ClassDiagramExporter::from_ast_with_filter(ast, filter_ref);
                    // Skip duplicate header
                    for line in part.lines().skip_while(|l| *l == "classDiagram") {
                        mermaid.push_str(line);
                        mermaid.push('\n');
                    }
                }
                write_output(mermaid)?;
            } else {
                // Single file path behavior
                let source = std::fs::read_to_string(&path)?;
                let lang = LanguageDetector::detect_from_path(&path);
                if !registry.supports_language(&lang) {
                    return Err(code_context_graph_core::CodeGraphError::Parser { message: format!("Unsupported language for {}", path.display()) }.into());
                }
                let ast = registry.parse(&source, lang)?;
                let mermaid = ClassDiagramExporter::from_ast_with_filter(&ast, filter_ref);
                write_output(mermaid)?;
            }
            
            println!("✅ Mermaid class diagram written to {}", out.display());
        }
    }
    Ok(())
}