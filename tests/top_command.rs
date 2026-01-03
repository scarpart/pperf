use std::process::Command;

#[test]
fn test_top_command_basic() {
    let output = Command::new("cargo")
        .args(["run", "--", "top", "perf-report.txt"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "Command failed: {}", stderr);
    assert!(stdout.contains("Children%"), "Output should have header");
    assert!(stdout.contains("Self%"), "Output should have Self% column");
    assert!(
        stdout.contains("Function"),
        "Output should have Function column"
    );

    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() >= 2, "Should have header + at least 1 data row");
    assert!(
        lines.len() <= 11,
        "Should have at most 10 data rows + header"
    );
}

#[test]
fn test_top_command_sorted_by_children() {
    let output = Command::new("cargo")
        .args(["run", "--", "top", "perf-report.txt"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = stdout.lines().skip(1).collect();

    let mut prev_pct: f64 = 100.0;
    for line in lines.iter().take(10) {
        let pct_str = line.split_whitespace().next().unwrap_or("0");
        let pct: f64 = pct_str.parse().unwrap_or(0.0);
        assert!(
            pct <= prev_pct,
            "Results should be sorted by Children% descending"
        );
        prev_pct = pct;
    }
}

#[test]
fn test_top_command_file_not_found() {
    let output = Command::new("cargo")
        .args(["run", "--", "top", "nonexistent.txt"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(), "Should fail for missing file");
    assert_eq!(output.status.code(), Some(1), "Exit code should be 1");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("File not found") || stderr.contains("nonexistent.txt"),
        "Error message should mention file not found"
    );
}

#[test]
fn test_top_command_self_flag() {
    let output = Command::new("cargo")
        .args(["run", "--", "top", "--self", "perf-report.txt"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "Command failed: {}", stderr);

    // Verify output is sorted by Self% (second column) descending
    let lines: Vec<&str> = stdout.lines().skip(1).collect();
    let mut prev_self_pct: f64 = 100.0;
    for line in lines.iter().take(10) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            let self_pct: f64 = parts[1].parse().unwrap_or(0.0);
            assert!(
                self_pct <= prev_self_pct,
                "Results should be sorted by Self% descending: {} should be <= {}",
                self_pct,
                prev_self_pct
            );
            prev_self_pct = self_pct;
        }
    }
}

#[test]
fn test_top_command_self_flag_short() {
    let output = Command::new("cargo")
        .args(["run", "--", "top", "-s", "perf-report.txt"])
        .output()
        .expect("Failed to execute command");

    assert!(
        output.status.success(),
        "Command with -s flag should succeed"
    );
}

#[test]
fn test_top_command_n_flag() {
    let output = Command::new("cargo")
        .args(["run", "--", "top", "-n", "5", "perf-report.txt"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "Command failed: {}", stderr);

    let data_lines: Vec<&str> = stdout.lines().skip(1).collect();
    assert_eq!(
        data_lines.len(),
        5,
        "Should have exactly 5 data rows, got {}",
        data_lines.len()
    );
}

#[test]
fn test_top_command_n_flag_long() {
    let output = Command::new("cargo")
        .args(["run", "--", "top", "--number", "3", "perf-report.txt"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());

    let data_lines: Vec<&str> = stdout.lines().skip(1).collect();
    assert_eq!(data_lines.len(), 3, "Should have exactly 3 data rows");
}

#[test]
fn test_top_command_n_zero_error() {
    let output = Command::new("cargo")
        .args(["run", "--", "top", "-n", "0", "perf-report.txt"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(), "Should fail for -n 0");
    assert_eq!(
        output.status.code(),
        Some(3),
        "Exit code should be 3 for invalid count"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Invalid") || stderr.contains("count"),
        "Error message should mention invalid count"
    );
}

#[test]
fn test_top_command_n_invalid_value() {
    let output = Command::new("cargo")
        .args(["run", "--", "top", "-n", "abc", "perf-report.txt"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success(), "Should fail for non-numeric -n");
    assert_eq!(
        output.status.code(),
        Some(3),
        "Exit code should be 3 for invalid count"
    );
}

#[test]
fn test_top_command_targets_filter() {
    let output = Command::new("cargo")
        .args(["run", "--", "top", "--targets", "DCT4D", "perf-report.txt"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "Command failed: {}", stderr);

    // All data rows should contain DCT4D prefix
    for line in stdout.lines().skip(1) {
        assert!(
            line.contains("DCT4D"),
            "Filtered output should only contain DCT4D functions, got: {}",
            line
        );
    }
}

#[test]
fn test_top_command_targets_short_flag() {
    let output = Command::new("cargo")
        .args(["run", "--", "top", "-t", "std::", "perf-report.txt"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());

    for line in stdout.lines().skip(1) {
        assert!(
            line.contains("std::"),
            "Filtered output should only contain std:: functions"
        );
    }
}

#[test]
fn test_top_command_targets_multiple() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "top",
            "--targets",
            "DCT4D",
            "Block4D",
            "perf-report.txt",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());

    for line in stdout.lines().skip(1) {
        assert!(
            line.contains("DCT4D") || line.contains("Block4D"),
            "Filtered output should contain DCT4D or Block4D functions, got: {}",
            line
        );
    }
}

#[test]
fn test_top_command_targets_no_matches() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "top",
            "--targets",
            "NonExistentFunction",
            "perf-report.txt",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(
        !output.status.success(),
        "Should fail when no functions match"
    );
    assert_eq!(
        output.status.code(),
        Some(4),
        "Exit code should be 4 for no matches"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No matching") || stderr.contains("no match"),
        "Error should mention no matching functions"
    );
}

#[test]
fn test_top_command_combined_flags() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "top",
            "--self",
            "-n",
            "3",
            "--targets",
            "Block4D",
            "perf-report.txt",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "Combined flags failed: {}", stderr);

    let data_lines: Vec<&str> = stdout.lines().skip(1).collect();
    assert!(data_lines.len() <= 3, "Should have at most 3 rows");

    for line in &data_lines {
        assert!(
            line.contains("Block4D"),
            "Should only contain Block4D functions"
        );
    }
}
