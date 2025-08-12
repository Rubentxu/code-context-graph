use assert_cmd::prelude::*;
use std::process::Command;
use tempfile::tempdir;
use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir.parent().unwrap().parent().unwrap().to_path_buf()
}

#[test]
fn viz_class_generates_mermaid_html() {
    let tmp = tempdir().unwrap();
    let out_path = tmp.path().join("graph.html");

    let example = workspace_root().join("examples/python/example.py");

    let mut cmd = Command::cargo_bin("ccg").unwrap();
    cmd.arg("viz")
        .arg("class")
        .arg("--path").arg(example)
        .arg("--out").arg(&out_path)
        .arg("--format").arg("html");

    cmd.assert().success();

    let content = fs::read_to_string(&out_path).unwrap();
    assert!(content.contains("<div class=\"mermaid\">") || content.contains("<pre class=\"mermaid\">"));
    assert!(content.contains("classDiagram"));
}

#[test]
fn viz_class_java_example_md() {
    let tmp = tempdir().unwrap();
    let out_path = tmp.path().join("java.md");

    let example = workspace_root().join("examples/java/Example.java");

    let mut cmd = Command::cargo_bin("ccg").unwrap();
    cmd.arg("viz")
        .arg("class")
        .arg("--path").arg(example)
        .arg("--out").arg(&out_path)
        .arg("--format").arg("md");

    cmd.assert().success();

    let content = fs::read_to_string(&out_path).unwrap();
    assert!(content.contains("classDiagram"));
}

#[test]
fn viz_class_filter_only_user_in_java() {
    let tmp = tempdir().unwrap();
    let out_path = tmp.path().join("java_user_only.md");

    let example = workspace_root().join("examples/java/Example.java");

    let mut cmd = Command::cargo_bin("ccg").unwrap();
    cmd.arg("viz")
        .arg("class")
        .arg("--path").arg(example)
        .arg("--out").arg(&out_path)
        .arg("--filter-class").arg("User");

    cmd.assert().success();

    let content = fs::read_to_string(&out_path).unwrap();
    assert!(content.contains("classDiagram"));
    assert!(content.contains("class User"));
    assert!(!content.contains("class UserService"));
}
