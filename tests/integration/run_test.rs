use crate::common::run_picolayer;

#[test]
fn test_picolayer_run_python_version() {
    let output = run_picolayer(&["run", "python@3.11", "--version"]);

    if !output.status.success() {
        eprintln!(
            "Python version test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Python 3.11"));
}

#[test]
fn test_picolayer_run_node_version() {
    let output = run_picolayer(&["run", "node@18", "--version"]);

    if !output.status.success() {
        eprintln!(
            "Node version test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("v18"));
}

#[test]
fn test_picolayer_run_with_working_directory() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let working_dir = temp_dir.path().to_str().unwrap();

    let script_path = temp_dir.path().join("test_script.py");
    std::fs::write(&script_path, "print('Hello from script')").expect("Failed to write script");

    let output = run_picolayer(&[
        "run",
        "python",
        "test_script.py",
        "--working-dir",
        working_dir,
    ]);

    if !output.status.success() {
        eprintln!(
            "Working directory test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello from script"));
}

#[test]
fn test_picolayer_run_dependency_detection() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    let package_json = temp_dir.path().join("package.json");
    std::fs::write(&package_json, r#"{"name": "test", "version": "1.0.0"}"#)
        .expect("Failed to write package.json");

    let output = run_picolayer(&[
        "run",
        "node",
        "--version",
        "--working-dir",
        temp_dir.path().to_str().unwrap(),
    ]);

    if !output.status.success() {
        eprintln!(
            "Dependency detection test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("v"));
}

#[test]
fn test_picolayer_run_python_with_requirements() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    let requirements_txt = temp_dir.path().join("requirements.txt");
    std::fs::write(&requirements_txt, "requests==2.28.0")
        .expect("Failed to write requirements.txt");

    let output = run_picolayer(&[
        "run",
        "python",
        "--version",
        "--working-dir",
        temp_dir.path().to_str().unwrap(),
    ]);

    if !output.status.success() {
        eprintln!(
            "Python with requirements test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Python"));
}

#[test]
fn test_picolayer_run_go_with_mod() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");

    let go_mod = temp_dir.path().join("go.mod");
    std::fs::write(&go_mod, "module test\n\ngo 1.19").expect("Failed to write go.mod");

    let output = run_picolayer(&[
        "run",
        "go",
        "version",
        "--working-dir",
        temp_dir.path().to_str().unwrap(),
    ]);

    if !output.status.success() {
        eprintln!(
            "Go with mod test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("go version"));
}

#[test]
fn test_picolayer_run_python_with_version_simple() {
    let output = run_picolayer(&["run", "python@3.10", "--version"]);

    if !output.status.success() {
        eprintln!(
            "Python 3.10 version test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Python 3.10"));
}

#[test]
fn test_picolayer_run_python_latest() {
    let output = run_picolayer(&["run", "python", "--version"]);

    if !output.status.success() {
        eprintln!(
            "Python latest test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Python"));
}

#[test]
fn test_picolayer_run_python_script() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let script_path = temp_dir.path().join("test.py");
    std::fs::write(&script_path, "print('Hello from Python!')").expect("Failed to write script");

    let output = run_picolayer(&[
        "run",
        "python",
        script_path.to_str().unwrap(),
        "--working-dir",
        temp_dir.path().to_str().unwrap(),
    ]);

    if !output.status.success() {
        eprintln!(
            "Python script test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello from Python!"));
}

#[test]
fn test_picolayer_run_node_with_version_simple() {
    let output = run_picolayer(&["run", "node@18", "--version"]);

    if !output.status.success() {
        eprintln!(
            "Node 18 version test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("v18."));
}

#[test]
fn test_picolayer_run_node_inline_code() {
    let output = run_picolayer(&["run", "node", "-e", "console.log('Hello from Node.js!')"]);

    if !output.status.success() {
        eprintln!(
            "Node inline code test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello from Node.js!"));
}

#[test]
fn test_picolayer_run_go_with_version() {
    let output = run_picolayer(&["run", "go@1.21", "version"]);

    if !output.status.success() {
        eprintln!(
            "Go 1.21 version test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("go1.21"));
}

#[test]
fn test_picolayer_run_ruby_inline() {
    let output = run_picolayer(&["run", "ruby", "-e", "puts 'Hello from Ruby!'"]);

    if !output.status.success() {
        eprintln!(
            "Ruby inline test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hello from Ruby!"));
}

#[test]
fn test_picolayer_run_with_env_vars() {
    let output = run_picolayer(&[
        "run",
        "python",
        "-c",
        "import os; print(f'TEST_VAR={os.environ.get(\"TEST_VAR\", \"not found\")}')",
        "--env",
        "TEST_VAR=hello_world",
    ]);

    if !output.status.success() {
        eprintln!(
            "Python env vars test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("TEST_VAR=hello_world"));
}

#[test]
fn test_picolayer_run_with_force_pkgx() {
    let output = run_picolayer(&["run", "echo", "hello", "world", "--force-pkgx"]);

    if !output.status.success() {
        eprintln!(
            "Force pkgx test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hello world"));
}

#[test]
fn test_picolayer_run_rust_with_version() {
    let output = run_picolayer(&["run", "rustc@1.70", "--version"]);

    if !output.status.success() {
        eprintln!(
            "Rust 1.70 version test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("rustc 1.70"));
}

#[test]
fn test_picolayer_run_multiple_args() {
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");

    std::fs::write(&file1, "content1").expect("Failed to write file1");
    std::fs::write(&file2, "content2").expect("Failed to write file2");

    let output = run_picolayer(&[
        "run",
        "cat",
        file1.to_str().unwrap(),
        file2.to_str().unwrap(),
        "--working-dir",
        temp_dir.path().to_str().unwrap(),
    ]);

    if !output.status.success() {
        eprintln!(
            "Multiple args test failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("content1"));
    assert!(stdout.contains("content2"));
}
