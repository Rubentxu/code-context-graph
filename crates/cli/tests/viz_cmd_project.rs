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
fn viz_class_project_python_md() {
    let tmp = tempdir().unwrap();
    let out_path = tmp.path().join("python_project.md");

    let example_dir = workspace_root().join("examples/python");

    let mut cmd = Command::cargo_bin("ccg").unwrap();
    cmd.arg("viz")
        .arg("class")
        .arg("--path").arg(example_dir)
        .arg("--out").arg(&out_path)
        .arg("--format").arg("md");

    cmd.assert().success();

    let content = fs::read_to_string(&out_path).unwrap();
    assert!(content.contains("classDiagram"));
    // Should include at least one known class from example.py
    assert!(content.contains("class User") || content.contains("class Example") || content.contains("class Greeter"));
}

#[test]
fn viz_class_project_java_md() {
    let tmp = tempdir().unwrap();
    let out_path = tmp.path().join("java_project.md");

    let example_dir = workspace_root().join("examples/java");

    let mut cmd = Command::cargo_bin("ccg").unwrap();
    cmd.arg("viz")
        .arg("class")
        .arg("--path").arg(example_dir)
        .arg("--out").arg(&out_path)
        .arg("--format").arg("md");

    cmd.assert().success();

    let content = fs::read_to_string(&out_path).unwrap();
    assert!(content.contains("classDiagram"));
    assert!(content.contains("class User") || content.contains("class UserService") || content.contains("class Example"));
}
