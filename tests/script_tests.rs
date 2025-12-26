use std::path::{Path, PathBuf};
use std::process::Command;

// Get the project root directory
fn project_root() -> PathBuf {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    PathBuf::from(manifest_dir)
}

// Run a shell script and return true if it succeeds, false otherwise
fn run_script(script_path: &Path) -> bool {
    println!("Running script test: {}", script_path.display());
    
    let status = Command::new("bash")
        .arg(script_path)
        .status()
        .expect(&format!("Failed to execute script: {}", script_path.display()));
    
    let success = status.success();
    if success {
        println!("✅ Script test passed: {}", script_path.display());
    } else {
        println!("❌ Script test failed: {} (exit code: {:?})", 
                 script_path.display(), status.code());
    }
    
    success
}

// Test that runs all the script tests
#[test]
fn test_all_scripts() {
    let scripts_dir = project_root().join("tests").join("scripts");
    let mut all_passed = true;
    
    // Get all test_*.sh files
    let entries = std::fs::read_dir(&scripts_dir)
        .expect("Failed to read scripts directory");
    
    let test_scripts: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .filter(|entry| {
            if let Some(file_name) = entry.path().file_name() {
                let file_name_str = file_name.to_string_lossy();
                file_name_str.starts_with("test_") && file_name_str.ends_with(".sh")
            } else {
                false
            }
        })
        .map(|entry| entry.path())
        .collect();
    
    if test_scripts.is_empty() {
        panic!("No test scripts found in {}", scripts_dir.display());
    }
    
    println!("Found {} shell script tests", test_scripts.len());
    
    // Run each test script
    for script in test_scripts {
        if !run_script(&script) {
            all_passed = false;
        }
    }
    
    assert!(all_passed, "One or more shell script tests failed");
}

// Individual test cases for each script
// These allow running a specific script test with cargo test test_ai_commit_script for example

#[test]
fn test_ai_cursor_model_script() {
    let script = project_root().join("tests/scripts/test_ai_cursor_model.sh");
    assert!(run_script(&script), "ai-cursor-model.sh test failed");
}

#[test]
fn test_ai_commit_script() {
    let script = project_root().join("tests/scripts/test_ai_commit.sh");
    assert!(run_script(&script), "ai-commit.sh test failed");
}

#[test]
fn test_ai_clippy_script() {
    let script = project_root().join("tests/scripts/test_ai_clippy.sh");
    assert!(run_script(&script), "ai-clippy.sh test failed");
}

#[test]
fn test_ai_branch_script() {
    let script = project_root().join("tests/scripts/test_ai_branch.sh");
    assert!(run_script(&script), "ai-branch.sh test failed");
}

#[test]
fn test_ai_pr_script() {
    let script = project_root().join("tests/scripts/test_ai_pr.sh");
    assert!(run_script(&script), "ai-pr.sh test failed");
}

#[test]
fn test_ai_rules_symlinks_script() {
    let script = project_root().join("tests/scripts/test_ai_rules_symlinks.sh");
    assert!(run_script(&script), "ai-rules-symlinks.sh test failed");
}

#[test]
fn test_ai_readme_script() {
    let script = project_root().join("tests/scripts/test_ai_readme.sh");
    assert!(run_script(&script), "ai-readme.sh test failed");
}

#[test]
fn test_ai_update_main_script() {
    // This test runs the shell script test_ai_update_main.sh which uses mock git commands
    // and should NOT modify the actual repository or filesystem state
    let script = project_root().join("tests/scripts/test_ai_update_main.sh");
    assert!(run_script(&script), "ai-update-main.sh test failed");
} 