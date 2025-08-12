use std::fs;
use std::io::Write;
use std::path::PathBuf;

use code_context_graph_core::Config;

fn write_temp_file(contents: &str) -> PathBuf {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("config.toml");
    let mut f = fs::File::create(&path).unwrap();
    f.write_all(contents.as_bytes()).unwrap();
    // Keep directory alive by leaking (test cleans nothing persistent)
    std::mem::ManuallyDrop::new(dir);
    path
}

#[test]
fn default_config_has_expected_values() {
    let cfg = Config::default();
    assert_eq!(cfg.engine.name, "code-context-graph");
    assert!(cfg.engine.languages.iter().any(|l| l == "python"));
    assert_eq!(cfg.api.port, 8080);
    assert!(cfg.cas.enabled);
    assert_eq!(cfg.versioning.merkle_tree_fanout, 16);
    assert_eq!(cfg.logging.level, "info");
    assert_eq!(cfg.logging.format, "json");
}

#[test]
fn from_file_parses_valid_toml() {
    let toml = r#"
        [engine]
        name = "ccg-test"
        languages = ["python", "javascript"]

        [parser]
        max_file_size_kb = 512
        ignore_patterns = ["*.min.js"]

        [falkordb]
        url = "redis://localhost:6379"
        graph_name = "g"

        [cas]
        enabled = true
        storage_path = "./tmp"
        hash_algorithm = "blake3"
        compression = "zstd"
        dedup_threshold = 0.9

        [file_watcher]
        enabled = false
        debounce_ms = 100
        batch_threshold = 10
        ignore_patterns = []
        recursive = true

        [versioning]
        enabled = true
        max_versions = 10
        auto_snapshot_interval = 60
        merkle_tree_fanout = 8

        [connascence]
        enabled = false
        detect_static = true
        detect_dynamic = false
        strength_threshold = 0.5
        auto_suggest_refactoring = false

        [aase]
        enabled = false
        context_path = "./context"
        naming_convention = "strict"
        auto_propagate = false
        human_review_threshold = 0.7
        artifact_versioning = false
        context_chain_depth = 3

        [quality_metrics]
        calculate_cohesion = true
        calculate_coupling = true
        maintainability_threshold = 50
        complexity_warning = 12

        [api]
        port = 9090
        max_context_size = 4096
        enable_version_api = true
        enable_quality_api = false
        enable_aase_api = false

        [logging]
        level = "debug"
        format = "text"
    "#;

    let path = write_temp_file(toml);
    let cfg = Config::from_file(&path).expect("parsed");

    assert_eq!(cfg.engine.name, "ccg-test");
    assert_eq!(cfg.api.port, 9090);
    assert_eq!(cfg.versioning.merkle_tree_fanout, 8);
    assert_eq!(cfg.logging.level, "debug");
    assert_eq!(cfg.logging.format, "text");
}

#[test]
fn from_file_invalid_toml_returns_error() {
    let toml = "not = [valid";
    let path = write_temp_file(toml);
    let err = Config::from_file(&path).unwrap_err();
    let msg = format!("{}", err);
    assert!(msg.contains("Failed to parse config"));
}
