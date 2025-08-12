use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;
use tempfile::tempdir;
use std::fs;
use std::path::PathBuf;

#[test]
fn viz_class_generates_mermaid_md() {
    let tmp = tempdir().unwrap();
    let out_path = tmp.path().join("graph.md");

    // Compute absolute path to example file from workspace root
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().unwrap().parent().unwrap();
    let example_path = workspace_root.join("examples/python/example.py");

    let mut cmd = Command::cargo_bin("ccg").unwrap();
    cmd.arg("viz")
        .arg("class")
        .arg("--path").arg(example_path)
        .arg("--out").arg(&out_path);

    cmd.assert().success();

    let content = fs::read_to_string(&out_path).unwrap();
    assert!(content.contains("classDiagram"), "Output did not contain Mermaid header.\nOutput was:\n{}", content);
}
