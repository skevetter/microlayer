mod common;

use common::run_picolayer;
use serial_test::serial;

#[test]
#[serial]
fn test_x_python_version() {
    let output = run_picolayer(&["x", "python@3.11", "--version"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Python 3.11"));
}

#[test]
#[serial]
fn test_x_node_version() {
    let output = run_picolayer(&["x", "node@18", "--version"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("v18"));
}

#[test]
#[serial]
fn test_x_with_working_directory() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let working_dir = temp_dir.path().to_str().unwrap();
    let script_path = temp_dir.path().join("test_script.py");
    std::fs::write(&script_path, "print('Hello from script')").expect("Failed to write script");

    let output = run_picolayer(&[
        "x",
        "--working-dir",
        working_dir,
        "python",
        "test_script.py",
    ]);

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Hello from script"));
}

#[test]
#[serial]
fn test_x_dependency_detection() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let package_json = temp_dir.path().join("package.json");
    std::fs::write(&package_json, r#"{"name": "test", "version": "1.0.0"}"#)
        .expect("Failed to write package.json");
    let output = run_picolayer(&[
        "x",
        "--working-dir",
        temp_dir.path().to_str().unwrap(),
        "node",
        "--version",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("v"));
}

#[test]
#[serial]
fn test_x_python_with_requirements() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let requirements_txt = temp_dir.path().join("requirements.txt");
    std::fs::write(&requirements_txt, "requests==2.28.0")
        .expect("Failed to write requirements.txt");

    let output = run_picolayer(&[
        "x",
        "--working-dir",
        temp_dir.path().to_str().unwrap(),
        "python",
        "--version",
    ]);

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Python"));
}

#[test]
#[serial]
fn test_x_go_with_mod() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let go_mod = temp_dir.path().join("go.mod");
    std::fs::write(&go_mod, "module test\n\ngo 1.19").expect("Failed to write go.mod");

    let output = run_picolayer(&[
        "x",
        "--working-dir",
        temp_dir.path().to_str().unwrap(),
        "go",
        "version",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("go version"));
}

#[test]
#[serial]
fn test_x_python_with_version_simple() {
    let output = run_picolayer(&["x", "python@3.10", "--version"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Python 3.10"));
}

#[test]
#[serial]
fn test_x_python_latest() {
    let output = run_picolayer(&["x", "python", "--version"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Python"));
}

#[test]
#[serial]
fn test_x_python_script() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let script_path = temp_dir.path().join("test.py");
    std::fs::write(&script_path, "print('Hello from Python!')").expect("Failed to write script");

    let output = run_picolayer(&[
        "x",
        "--working-dir",
        temp_dir.path().to_str().unwrap(),
        "python",
        script_path.to_str().unwrap(),
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Hello from Python!"));
}

#[test]
#[serial]
fn test_x_node_with_version_simple() {
    let output = run_picolayer(&["x", "node@18", "--version"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("v18."));
}

#[test]
#[serial]
fn test_x_python_inline_code() {
    let output = run_picolayer(&["x", "python", "-c", "print('Hello from Python!')"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Hello from Python!"));
}

#[test]
#[serial]
fn test_x_node_inline_code() {
    let output = run_picolayer(&[
        "x",
        "node",
        "--",
        "-e",
        "console.log('Hello from Node.js!')",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Hello from Node.js!"));
}

#[test]
#[serial]
fn test_x_go_with_version() {
    let output = run_picolayer(&["x", "go@1.21", "version"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("go1.21"));
}

#[test]
#[serial]
fn test_x_ruby_inline() {
    let output = run_picolayer(&["x", "ruby", "-e", "puts 'Hello from Ruby!'"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("Hello from Ruby!"));
}

#[test]
#[serial]
fn test_x_with_env_vars() {
    let output = run_picolayer(&[
        "x",
        "--env",
        "TEST_VAR=hello_world",
        "python",
        "--",
        "-c",
        "import os; print(f'TEST_VAR={os.environ.get(\"TEST_VAR\", \"not found\")}')",
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("TEST_VAR=hello_world"));
}

#[test]
#[serial]
fn test_x_rust_with_version() {
    let output = run_picolayer(&["x", "rustc@1.70", "--version"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("rustc 1.70"));
}

#[test]
#[serial]
fn test_x_multiple_args() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");

    std::fs::write(&file1, "content1").expect("Failed to write file1");
    std::fs::write(&file2, "content2").expect("Failed to write file2");

    let output = run_picolayer(&[
        "x",
        "--working-dir",
        temp_dir.path().to_str().unwrap(),
        "python",
        "-c",
        &format!(
            "
import os
with open('{}', 'r') as f1, open('{}', 'r') as f2:
    print(f1.read().strip())
    print(f2.read().strip())
        ",
            file1.file_name().unwrap().to_str().unwrap(),
            file2.file_name().unwrap().to_str().unwrap()
        ),
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(stdout.contains("content1"));
    assert!(stdout.contains("content2"));
}
