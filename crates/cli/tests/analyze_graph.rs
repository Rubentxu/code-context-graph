use assert_cmd::prelude::*;
use std::process::Command;
use tempfile::tempdir;
use std::fs;

// This test assumes a global test hook in graph crate to capture executed queries
#[test]
fn analyze_persists_basic_function_and_imports_to_graph() {
    let dir = tempdir().unwrap();
    // create a minimal config.toml
    let config = r#"
        [engine]
        name = "ccg"
        languages = ["python"]
        
        [parser]
        max_file_size_kb = 1024
        ignore_patterns = []

        [falkordb]
        url = "redis://localhost:6379"
        graph_name = "code_graph_test"

        [cas]
        enabled = true
        storage_path = "./.ccg"
        hash_algorithm = "blake3"
        compression = "zstd"
        dedup_threshold = 0.8

        [versioning]
        enabled = true
        max_versions = 10
        auto_snapshot_interval = 3600
        merkle_tree_fanout = 16

        [logging]
        level = "error"
        format = "text"
    "#;
    fs::write(dir.path().join("config.toml"), config).unwrap();
    // create source file
    let src = r#"import os\n\n\ndef foo(x):\n    return os.getcwd()\n"#;
    let src_path = dir.path().join("main.py");
    fs::write(&src_path, src).unwrap();

    // Tell graph crate to record queries into a temp file via env var hook
    let record_path = dir.path().join("queries.log");
    let mut cmd = Command::cargo_bin("ccg").unwrap();
    cmd.current_dir(dir.path())
        .env("CCG_GRAPH_TEST_RECORD", record_path.to_str().unwrap())
        .arg("--config").arg("config.toml")
        .arg("analyze")
        .arg("--path").arg(".");
    cmd.assert().success();

    let recorded = fs::read_to_string(&record_path).expect("record file should exist");
    assert!(recorded.contains("MERGE (fn:Function { name: 'foo' })"));
    assert!(recorded.contains("MERGE (m:Module { name: 'os' })"));
}
