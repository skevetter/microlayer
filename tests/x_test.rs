mod common;

use anyhow::Result;
use common::run_picolayer;
use serial_test::serial;
use std::path::PathBuf;

fn get_platform_pkgx_paths() -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();

    if let Some(home_dir) = dirs_next::home_dir() {
        #[cfg(target_os = "macos")]
        {
            for path in ["Library/Caches/pkgx", "Library/Application Support/pkgx"] {
                paths.push(home_dir.join(path));
            }
        }

        #[cfg(target_os = "linux")]
        {
            let cache_dir = if let Ok(xdg_cache) = env::var("XDG_CACHE_HOME") {
                PathBuf::from(xdg_cache)
            } else {
                home_dir.join(".cache")
            };
            paths.push(cache_dir.join("pkgx"));

            let data_dir = if let Ok(xdg_data) = env::var("XDG_DATA_HOME") {
                PathBuf::from(xdg_data)
            } else {
                home_dir.join(".local/share")
            };
            paths.push(data_dir.join("pkgx"));
        }

        paths.push(home_dir.join(".pkgx"))
    }

    Ok(paths)
}

#[test]
#[serial]
fn test_x_without_existing_pkgx_cache() {
    let paths = get_platform_pkgx_paths().unwrap_or_default();
    assert!(!paths.is_empty(), "No pkgx paths found for this platform");

    for path in &paths {
        if path.exists() {
            std::fs::remove_dir_all(path).unwrap();
        }
    }

    assert!(
        paths.iter().all(|p| !p.exists()),
        "Some pkgx paths still exist: {:?}",
        paths
            .iter()
            .filter(|p| p.exists())
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<String>>()
    );

    let output = run_picolayer(&["x", "python", "--version"]);

    println!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    println!("STDERR: {}", String::from_utf8_lossy(&output.stderr));

    assert!(
        output.status.success(),
        "picolayer run failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    assert!(
        paths.iter().all(|p| !p.exists()),
        "Some pkgx paths still exist: {:?}",
        paths
            .iter()
            .filter(|p| p.exists())
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<String>>()
    );
}

#[test]
#[serial]
fn test_x_python_version() {
    let output = run_picolayer(&["x", "python@3.11", "--version"]);

    print!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    print!("STDERR: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

    assert!(stdout.contains("Python 3.11"));
}

#[test]
#[serial]
fn test_x_node_version() {
    let output = run_picolayer(&["x", "node@18", "--version"]);

    print!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    print!("STDERR: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

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

    print!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    print!("STDERR: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

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

    print!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    print!("STDERR: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

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

    print!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    print!("STDERR: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

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

    print!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    print!("STDERR: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

    assert!(stdout.contains("go version"));
}

#[test]
#[serial]
fn test_x_python_with_version_simple() {
    let output = run_picolayer(&["x", "python@3.10", "--version"]);

    print!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    print!("STDERR: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

    assert!(stdout.contains("Python 3.10"));
}

#[test]
#[serial]
fn test_x_python_latest() {
    let output = run_picolayer(&["x", "python", "--version"]);

    print!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    print!("STDERR: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

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

    print!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    print!("STDERR: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

    assert!(stdout.contains("Hello from Python!"));
}

#[test]
#[serial]
fn test_x_node_with_version_simple() {
    let output = run_picolayer(&["x", "node@18", "--version"]);

    print!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    print!("STDERR: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

    assert!(stdout.contains("v18."));
}

#[test]
#[serial]
fn test_x_python_inline_code() {
    let output = run_picolayer(&["x", "python", "-c", "print('Hello from Python!')"]);

    print!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    print!("STDERR: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

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

    print!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    print!("STDERR: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

    assert!(stdout.contains("Hello from Node.js!"));
}

#[test]
#[serial]
fn test_x_go_with_version() {
    let output = run_picolayer(&["x", "go@1.21", "version"]);

    print!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    print!("STDERR: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

    assert!(stdout.contains("go1.21"));
}

#[test]
#[serial]
fn test_x_ruby_inline() {
    let output = run_picolayer(&["x", "ruby", "-e", "puts 'Hello from Ruby!'"]);

    print!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    print!("STDERR: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);

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

    print!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    print!("STDERR: {}", String::from_utf8_lossy(&output.stderr));

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("Output: {}", stdout);

    assert!(stdout.contains("TEST_VAR=hello_world"));
}

#[test]
#[serial]
fn test_x_rust_with_version() {
    let output = run_picolayer(&["x", "rustc@1.70", "--version"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        println!("Error: {}", stderr);
    };
    println!("Output: {}", stdout);
    println!("Output: {}", stderr);

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
    print!("STDOUT: {}", String::from_utf8_lossy(&output.stdout));
    print!("STDERR: {}", String::from_utf8_lossy(&output.stderr));

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Error: {}", stderr);
    };

    assert!(stdout.contains("content1"));
    assert!(stdout.contains("content2"));
}
