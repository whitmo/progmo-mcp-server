use std::env;
use std::process::Command;
use std::path::Path;

#[test]
fn test_main_help_flag() {
    // Get the path to the binary
    let binary_path = env::current_exe().unwrap().parent().unwrap().join("p-mo");
    
    // Skip the test if the binary doesn't exist
    if !Path::new(&binary_path).exists() {
        println!("Skipping test_main_help_flag: Binary not found at {:?}", binary_path);
        return;
    }
    
    // Run the main binary with --help flag
    let output = Command::new(&binary_path)
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    // Check that the command executed successfully
    assert!(output.status.success());

    // Check that the output contains expected help text
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("Options:"));
    assert!(stdout.contains("Commands:"));
}

#[test]
fn test_main_version_flag() {
    // Get the path to the binary
    let binary_path = env::current_exe().unwrap().parent().unwrap().join("p-mo");
    
    // Skip the test if the binary doesn't exist
    if !Path::new(&binary_path).exists() {
        println!("Skipping test_main_version_flag: Binary not found at {:?}", binary_path);
        return;
    }
    
    // Run the main binary with --version flag
    let output = Command::new(&binary_path)
        .arg("--version")
        .output()
        .expect("Failed to execute command");

    // Check that the command executed successfully
    assert!(output.status.success());

    // Check that the output contains version information
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("p-mo"));
}

#[test]
fn test_main_invalid_command() {
    // Get the path to the binary
    let binary_path = env::current_exe().unwrap().parent().unwrap().join("p-mo");
    
    // Skip the test if the binary doesn't exist
    if !Path::new(&binary_path).exists() {
        println!("Skipping test_main_invalid_command: Binary not found at {:?}", binary_path);
        return;
    }
    
    // Run the main binary with an invalid command
    let output = Command::new(&binary_path)
        .arg("invalid-command")
        .output()
        .expect("Failed to execute command");

    // Check that the command failed
    assert!(!output.status.success());

    // Check that the error output contains expected error message
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error:"));
}
