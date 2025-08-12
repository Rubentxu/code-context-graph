use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use code_context_graph_storage::merkle::MerkleBuilder;

#[test]
fn analyze_minimal_creates_cas_snapshot() {
    // Prepare a temporary repo
    let tmp = tempfile::tempdir().unwrap();
    std::fs::write(tmp.path().join("main.py"), b"print('hello')\n").unwrap();

    // Run `ccg analyze <path>`
    let mut cmd = Command::cargo_bin("ccg").unwrap();
    cmd.arg("analyze").arg("--path").arg(tmp.path());
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Analyzing codebase"));

    // Verify CAS directory was created
    let cas_dir = tmp.path().join(".ccg").join("cas");
    assert!(cas_dir.exists(), ".ccg/cas directory should be created by analyze");

    // Later: verify CAS snapshot exists once CAS is implemented
}

#[test]
fn analyze_with_message_and_show_displays_it() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();
    std::fs::write(repo.join("a.py"), b"print('a')\n").unwrap();

    // Analyze with message
    let assert = Command::cargo_bin("ccg").unwrap()
        .args(["analyze", "--path"]).arg(repo)
        .args(["--message"]).arg("first snapshot")
        .assert().success();
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let root_line = out.lines().find(|l| l.starts_with("root: ")).unwrap();
    let root = root_line.trim_start_matches("root: ").trim().to_string();

    // Show contains message
    Command::cargo_bin("ccg").unwrap()
        .args(["version", "show", "--path"]).arg(repo)
        .args(["--id"]).arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains("message: first snapshot"));
}

#[test]
fn version_diff_reports_added_removed_changed() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();
    // Initial files: a.py, b.py
    std::fs::write(repo.join("a.py"), b"print('a1')\n").unwrap();
    std::fs::write(repo.join("b.py"), b"print('b')\n").unwrap();
    // First analyze
    let a1 = Command::cargo_bin("ccg").unwrap()
        .args(["analyze", "--path"]).arg(repo)
        .assert().success();
    let out1 = String::from_utf8(a1.get_output().stdout.clone()).unwrap();
    let r1 = out1.lines().find(|l| l.starts_with("root: ")).unwrap().trim_start_matches("root: ").trim().to_string();

    // Change a.py, remove b.py, add c.py
    std::fs::write(repo.join("a.py"), b"print('a2')\n").unwrap();
    std::fs::remove_file(repo.join("b.py")).unwrap();
    std::fs::write(repo.join("c.py"), b"print('c')\n").unwrap();

    // Second analyze
    let a2 = Command::cargo_bin("ccg").unwrap()
        .args(["analyze", "--path"]).arg(repo)
        .assert().success();
    let out2 = String::from_utf8(a2.get_output().stdout.clone()).unwrap();
    let r2 = out2.lines().find(|l| l.starts_with("root: ")).unwrap().trim_start_matches("root: ").trim().to_string();

    // version diff from r1 to r2
    Command::cargo_bin("ccg").unwrap()
        .args(["version", "diff", "--path"]).arg(repo)
        .args(["--from"]).arg(&r1)
        .args(["--to"]).arg(&r2)
        .assert()
        .success()
        .stdout(predicate::str::contains("Added:")
            .and(predicate::str::contains("c.py")))
        .stdout(predicate::str::contains("Removed:")
            .and(predicate::str::contains("b.py")))
        .stdout(predicate::str::contains("Changed:")
            .and(predicate::str::contains("a.py")));
}

#[test]
fn version_list_and_show_after_analyze() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();
    std::fs::write(repo.join("a.py"), b"print('a')\n").unwrap();

    // Run analyze and capture root
    let assert = Command::cargo_bin("ccg").unwrap()
        .args(["analyze", "--path"]).arg(repo)
        .assert()
        .success();
    let out = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let root_line = out.lines().find(|l| l.starts_with("root: ")).unwrap();
    let root = root_line.trim_start_matches("root: ").trim().to_string();

    // version list shows the root
    Command::cargo_bin("ccg").unwrap()
        .args(["version", "list", "--path"]).arg(repo)
        .assert()
        .success()
        .stdout(predicate::str::contains(&root));

    // version show prints metadata including root
    Command::cargo_bin("ccg").unwrap()
        .args(["version", "show", "--path"]).arg(repo)
        .args(["--id"]).arg(&root)
        .assert()
        .success()
        .stdout(predicate::str::contains(&format!("root: {}", root)));
}

#[test]
fn analyze_respects_max_file_size_kb() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();

    // Create small and big file
    let small = b"print('small')\n".to_vec();
    let big = vec![b'a'; 2048]; // 2KB
    std::fs::write(repo.join("ok.py"), &small).unwrap();
    std::fs::write(repo.join("big.py"), &big).unwrap();

    // Config limiting max file size to 1KB
    let config_path = repo.join("ccg.toml");
    let config = r#"engine = { name = "ccg", languages = ["python"] }
        [parser]
        max_file_size_kb = 1
        ignore_patterns = []
        [falkordb]
        url = "redis://localhost:6379"
        graph_name = "code_graph"
        [cas]
        enabled = true
        storage_path = "./.ccg/cas"
        hash_algorithm = "blake3"
        compression = "none"
        dedup_threshold = 0.8
        [file_watcher]
        enabled = true
        debounce_ms = 100
        batch_threshold = 50
        ignore_patterns = []
        recursive = true
        [versioning]
        enabled = true
        max_versions = 1000
        auto_snapshot_interval = 3600
        merkle_tree_fanout = 2
        [connascence]
        enabled = false
        detect_static = true
        detect_dynamic = false
        strength_threshold = 0.7
        auto_suggest_refactoring = false
        [aase]
        enabled = false
        context_path = "./context"
        naming_convention = "strict"
        auto_propagate = false
        human_review_threshold = 0.8
        artifact_versioning = false
        context_chain_depth = 3
        [quality_metrics]
        calculate_cohesion = true
        calculate_coupling = true
        maintainability_threshold = 65
        complexity_warning = 10
        [api]
        port = 8080
        max_context_size = 8192
        enable_version_api = true
        enable_quality_api = true
        enable_aase_api = false
        [logging]
        level = "info"
        format = "text"
    "#;
    std::fs::write(&config_path, config).unwrap();

    // Expected merkle with only small file
    let mut mb = MerkleBuilder::new();
    mb.add("ok.py", &small);
    let expected_root = mb.build().root();

    let mut cmd = Command::cargo_bin("ccg").unwrap();
    cmd.arg("--config").arg(&config_path)
        .arg("analyze").arg("--path").arg(repo);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Indexed files: 1"))
        .stdout(predicate::str::contains(&format!("root: {}", expected_root)));    
}

#[test]
fn analyze_respects_config_ignore_and_cas_path() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();
    // Create files, including ignored ones
    std::fs::create_dir_all(repo.join(".git")).unwrap();
    std::fs::create_dir_all(repo.join("pkg")).unwrap();
    std::fs::write(repo.join("keep.py"), b"print('keep')\n").unwrap();
    std::fs::write(repo.join("pkg").join("ignore.min.js"), b"console.log('x');\n").unwrap();
    std::fs::write(repo.join(".git").join("ignored.txt"), b"ignored\n").unwrap();

    // Config with ignore patterns and custom CAS path
    let cas_dir = repo.join("custom_cas");
    let config_path = repo.join("ccg.toml");
    let config = format!(
        r#"engine = {{ name = "ccg", languages = ["python"] }}
           [parser]
           max_file_size_kb = 1024
           ignore_patterns = ["*.min.js", ".git"]
           [falkordb]
           url = "redis://localhost:6379"
           graph_name = "code_graph"
           [cas]
           enabled = true
           storage_path = "{}"
           hash_algorithm = "blake3"
           compression = "none"
           dedup_threshold = 0.8
           [file_watcher]
           enabled = true
           debounce_ms = 100
           batch_threshold = 50
           ignore_patterns = [".git"]
           recursive = true
           [versioning]
           enabled = true
           max_versions = 1000
           auto_snapshot_interval = 3600
           merkle_tree_fanout = 2
           [connascence]
           enabled = false
           detect_static = true
           detect_dynamic = false
           strength_threshold = 0.7
           auto_suggest_refactoring = false
           [aase]
           enabled = false
           context_path = "./context"
           naming_convention = "strict"
           auto_propagate = false
           human_review_threshold = 0.8
           artifact_versioning = false
           context_chain_depth = 3
           [quality_metrics]
           calculate_cohesion = true
           calculate_coupling = true
           maintainability_threshold = 65
           complexity_warning = 10
           [api]
           port = 8080
           max_context_size = 8192
           enable_version_api = true
           enable_quality_api = true
           enable_aase_api = false
           [logging]
           level = "info"
           format = "text"
        "#,
        cas_dir.display()
    );
    std::fs::write(&config_path, config).unwrap();

    // Expected: only keep.py is indexed
    let mut mb = MerkleBuilder::new();
    let keep = std::fs::read(repo.join("keep.py")).unwrap();
    mb.add("keep.py", &keep);
    let expected_root = mb.build().root();

    // Run analyze with config
    let mut cmd = Command::cargo_bin("ccg").unwrap();
    cmd.arg("--config").arg(&config_path)
        .arg("analyze").arg("--path").arg(repo);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Indexed files: 1"))
        .stdout(predicate::str::contains(&format!("root: {}", expected_root)));

    // CAS should be at configured location
    assert!(cas_dir.exists(), "configured cas dir should exist");
}

#[test]
fn logging_respects_config_info_level() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();
    std::fs::write(repo.join("x.py"), b"print('x')\n").unwrap();

    let config_path = repo.join("ccg.toml");
    let config = r#"engine = { name = "ccg", languages = ["python"] }
        [parser]
        max_file_size_kb = 1024
        ignore_patterns = []
        [falkordb]
        url = "redis://localhost:6379"
        graph_name = "code_graph"
        [cas]
        enabled = true
        storage_path = "./.ccg/cas"
        hash_algorithm = "blake3"
        compression = "none"
        dedup_threshold = 0.8
        [file_watcher]
        enabled = true
        debounce_ms = 100
        batch_threshold = 50
        ignore_patterns = []
        recursive = true
        [versioning]
        enabled = true
        max_versions = 1000
        auto_snapshot_interval = 3600
        merkle_tree_fanout = 2
        [connascence]
        enabled = false
        detect_static = true
        detect_dynamic = false
        strength_threshold = 0.7
        auto_suggest_refactoring = false
        [aase]
        enabled = false
        context_path = "./context"
        naming_convention = "strict"
        auto_propagate = false
        human_review_threshold = 0.8
        artifact_versioning = false
        context_chain_depth = 3
        [quality_metrics]
        calculate_cohesion = true
        calculate_coupling = true
        maintainability_threshold = 65
        complexity_warning = 10
        [api]
        port = 8080
        max_context_size = 8192
        enable_version_api = true
        enable_quality_api = true
        enable_aase_api = false
        [logging]
        level = "info"
        format = "text"
    "#;
    std::fs::write(&config_path, config).unwrap();

    let mut cmd = Command::cargo_bin("ccg").unwrap();
    cmd.arg("--config").arg(&config_path)
        .arg("analyze").arg("--path").arg(repo);
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Starting analysis of:"));
}

#[test]
fn analyze_ingests_files_and_prints_summary() {
    let tmp = tempfile::tempdir().unwrap();
    // Create a small tree of files
    std::fs::create_dir_all(tmp.path().join("pkg/sub")).unwrap();
    std::fs::write(tmp.path().join("a.py"), b"print('a')\n").unwrap();
    std::fs::write(tmp.path().join("pkg").join("b.js"), b"console.log('b');\n").unwrap();
    std::fs::write(tmp.path().join("pkg/sub").join("c.java"), b"class C {}\n").unwrap();

    // Expected Merkle root using the same algorithm (fanout default=2)
    let mut mb = MerkleBuilder::new();
    let a = std::fs::read(tmp.path().join("a.py")).unwrap();
    let b = std::fs::read(tmp.path().join("pkg").join("b.js")).unwrap();
    let c = std::fs::read(tmp.path().join("pkg/sub").join("c.java")).unwrap();
    mb.add("a.py", &a);
    mb.add("pkg/b.js", &b);
    mb.add("pkg/sub/c.java", &c);
    let tree = mb.build();
    let expected_root = tree.root();

    // Run analyze
    let mut cmd = Command::cargo_bin("ccg").unwrap();
    cmd.arg("analyze").arg("--path").arg(tmp.path());
    // Validate summary
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Indexed files: 3"))
        .stdout(predicate::str::contains(&format!("root: {}", expected_root)));
}
