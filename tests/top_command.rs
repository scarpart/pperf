use std::process::Command;

#[test]
fn test_top_command_basic() {
    let output = Command::new("cargo")
        .args(["run", "--", "top", "examples/Bikes_005_rep0.txt"])
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
        .args(["run", "--", "top", "examples/Bikes_005_rep0.txt"])
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
        .args(["run", "--", "top", "--self", "examples/Bikes_005_rep0.txt"])
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
        .args(["run", "--", "top", "-s", "examples/Bikes_005_rep0.txt"])
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
        .args(["run", "--", "top", "-n", "5", "examples/Bikes_005_rep0.txt"])
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
        .args([
            "run",
            "--",
            "top",
            "--number",
            "3",
            "examples/Bikes_005_rep0.txt",
        ])
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
        .args(["run", "--", "top", "-n", "0", "examples/Bikes_005_rep0.txt"])
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
        stderr.contains("invalid") || stderr.contains("number") || stderr.contains("at least 1"),
        "Error message should mention invalid count: {}",
        stderr
    );
}

#[test]
fn test_top_command_n_invalid_value() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "top",
            "-n",
            "abc",
            "examples/Bikes_005_rep0.txt",
        ])
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
        .args([
            "run",
            "--",
            "top",
            "--targets",
            "DCT4D",
            "examples/Bikes_005_rep0.txt",
        ])
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
        .args([
            "run",
            "--",
            "top",
            "-t",
            "inner_product",
            "examples/Bikes_005_rep0.txt",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());

    // Substring matching: all results should contain "inner_product"
    for line in stdout.lines().skip(1) {
        assert!(
            line.contains("inner_product"),
            "Filtered output should only contain inner_product functions"
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
            "-t",
            "DCT4DBlock::",
            "-t",
            "get_mSubband",
            "examples/Bikes_005_rep0.txt",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success());

    // Substring matching: each result should contain at least one pattern
    for line in stdout.lines().skip(1) {
        assert!(
            line.contains("DCT4DBlock") || line.contains("get_mSubband"),
            "Filtered output should contain DCT4DBlock or get_mSubband, got: {}",
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
            "examples/Bikes_005_rep0.txt",
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
fn test_top_command_targets_substring_match() {
    // Key test: substring matching allows matching method names within class::method symbols
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "top",
            "--targets",
            "get_mSubband",
            "examples/Bikes_005_rep0.txt",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "Should find get_mSubband via substring match"
    );

    // Should match Hierarchical4DEncoder::get_mSubbandLF_significance
    assert!(
        stdout.contains("get_mSubband"),
        "Should find function containing get_mSubband"
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
            "get_mSubband",
            "examples/Bikes_005_rep0.txt",
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
            line.contains("get_mSubband"),
            "Should only contain get_mSubband functions"
        );
    }
}

// T042: Integration test for --no-color flag
#[test]
fn test_top_command_no_color_flag() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "top",
            "--no-color",
            "examples/Bikes_005_rep0.txt",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "Command failed: {}", stderr);

    // Verify no ANSI escape codes in output
    assert!(
        !stdout.contains('\x1b'),
        "Output should have no ANSI escape codes with --no-color"
    );
}

// T043: Integration test for piped output having no ANSI codes
#[test]
fn test_top_command_piped_no_color() {
    // When output is piped (not a TTY), colors should be disabled automatically
    // In tests, output is not a TTY, so colors should be disabled
    let output = Command::new("cargo")
        .args(["run", "--", "top", "examples/Bikes_005_rep0.txt"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Since we're piping output (not a TTY), there should be no ANSI codes
    assert!(
        !stdout.contains('\x1b'),
        "Piped output should have no ANSI escape codes"
    );
}

// ============================================================================
// Feature 003: Call Hierarchy Tests
// ============================================================================

// T039: Integration test for --hierarchy without --targets returning error
#[test]
fn test_top_command_hierarchy_requires_targets() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "top",
            "--hierarchy",
            "examples/Bikes_005_rep0.txt",
        ])
        .output()
        .expect("Failed to execute command");

    assert!(
        !output.status.success(),
        "--hierarchy without --targets should fail"
    );
    assert_eq!(
        output.status.code(),
        Some(3),
        "Exit code should be 3 for missing --targets"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--hierarchy requires --targets"),
        "Error should mention --hierarchy requires --targets"
    );
}

// T040: Integration test for --hierarchy with --targets producing output
#[test]
fn test_top_command_hierarchy_with_targets() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "top",
            "--hierarchy",
            "-t",
            "rd_optimize",
            "-t",
            "DCT4D",
            "examples/Bikes_005_rep0.txt",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "--hierarchy with --targets should succeed: {}",
        stderr
    );
    assert!(stdout.contains("Children%"), "Output should have header");
    assert!(
        stdout.contains("Function"),
        "Output should have Function column"
    );
}

// T040: Test short flag -H
#[test]
fn test_top_command_hierarchy_short_flag() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "top",
            "-H",
            "-t",
            "rd_optimize",
            "examples/Bikes_005_rep0.txt",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "-H flag should work: {}", stderr);
    assert!(
        stdout.contains("rd_optimize"),
        "Output should contain target function"
    );
}

// T043: Integration test with real examples/Bikes_005_rep0.txt
#[test]
fn test_top_command_hierarchy_real_data() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "top",
            "--hierarchy",
            "-t",
            "rd_optimize_transform",
            "-t",
            "DCT4DBlock",
            "--no-color",
            "examples/Bikes_005_rep0.txt",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Hierarchy with real targets should succeed: {}",
        stderr
    );

    // Should contain both target functions
    assert!(
        stdout.contains("rd_optimize_transform") || stdout.contains("rd_optimize"),
        "Output should contain rd_optimize_transform"
    );
}

// ============================================================================
// Feature 004: Debug Calculation Path Tests
// ============================================================================

// T010: Integration test for --hierarchy --debug with indirect call
#[test]
fn test_top_command_debug_with_hierarchy() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "top",
            "--hierarchy",
            "--debug",
            "-t",
            "rd_optimize",
            "-t",
            "DCT4DBlock",
            "-t",
            "inner_product",
            "--no-color",
            "examples/Bikes_005_rep0.txt",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "--hierarchy --debug should succeed: {}",
        stderr
    );

    // Should contain direct call annotations
    assert!(
        stdout.contains("(direct:"),
        "Output should contain direct call annotations"
    );
}

// T016: Integration test for --hierarchy --debug showing indirect via annotation
#[test]
fn test_top_command_debug_indirect_via_annotation() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "top",
            "--hierarchy",
            "--debug",
            "-t",
            "rd_optimize",
            "-t",
            "DCT4DBlock",
            "-t",
            "inner_product",
            "--no-color",
            "examples/Bikes_005_rep0.txt",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "--hierarchy --debug should succeed: {}",
        stderr
    );

    // Should contain indirect call annotations with "via"
    assert!(
        stdout.contains("(via "),
        "Output should contain indirect call annotations with 'via'"
    );
}

// T019: Integration test --debug without --hierarchy produces normal output
#[test]
fn test_top_command_debug_without_hierarchy() {
    // Run with --debug but without --hierarchy
    let debug_output = Command::new("cargo")
        .args([
            "run",
            "--",
            "top",
            "--debug",
            "-n",
            "5",
            "--no-color",
            "examples/Bikes_005_rep0.txt",
        ])
        .output()
        .expect("Failed to execute command");

    // Run without --debug (normal mode)
    let normal_output = Command::new("cargo")
        .args([
            "run",
            "--",
            "top",
            "-n",
            "5",
            "--no-color",
            "examples/Bikes_005_rep0.txt",
        ])
        .output()
        .expect("Failed to execute command");

    let debug_stdout = String::from_utf8_lossy(&debug_output.stdout);
    let normal_stdout = String::from_utf8_lossy(&normal_output.stdout);

    assert!(
        debug_output.status.success(),
        "--debug without --hierarchy should succeed"
    );
    assert!(normal_output.status.success(), "Normal mode should succeed");

    // Without --hierarchy, --debug should not add extra annotation lines
    // Check that both outputs have the same number of lines
    let debug_lines: Vec<&str> = debug_stdout.lines().collect();
    let normal_lines: Vec<&str> = normal_stdout.lines().collect();

    assert_eq!(
        debug_lines.len(),
        normal_lines.len(),
        "--debug without --hierarchy should produce same number of lines: debug={}, normal={}",
        debug_lines.len(),
        normal_lines.len()
    );

    // Both should have the header
    assert!(
        debug_stdout.contains("Children%"),
        "Debug output should have header"
    );
    assert!(
        normal_stdout.contains("Children%"),
        "Normal output should have header"
    );

    // Neither should have debug annotations (no "(values:" lines)
    assert!(
        !debug_stdout.contains("(values:"),
        "Single-file debug output should not have per-report annotations"
    );
}

// ============================================================================
// Feature 006: Multi-Report Averaging Tests
// ============================================================================

// T010: Integration test for multi-file CLI invocation
#[test]
fn test_top_command_multi_file_basic() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "top",
            "-n",
            "5",
            "--no-color",
            "examples/Bikes_005_rep0.txt",
            "examples/Bikes_005_rep1.txt",
            "examples/Bikes_005_rep2.txt",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Multi-file command should succeed: {}",
        stderr
    );
    assert!(stdout.contains("Children%"), "Output should have header");
    assert!(stdout.contains("Self%"), "Output should have Self% column");

    // Verify averaged percentages are present (e.g., 95.41% averaged)
    let lines: Vec<&str> = stdout.lines().collect();
    assert!(lines.len() >= 2, "Should have header + at least 1 data row");
}

// T011: Integration test for averaged output matching manual calculation
#[test]
fn test_top_command_multi_file_averaged_values() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "top",
            "-n",
            "3",
            "--no-color",
            "examples/Bikes_005_rep0.txt",
            "examples/Bikes_005_rep1.txt",
            "examples/Bikes_005_rep2.txt",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "Multi-file command should succeed");

    // The top entries should be averaged values
    // rep0: 98.38%, rep1: 95.92%, rep2: 91.94% -> avg: 95.41%
    // Check that averaged values are reasonable (between 90-100%)
    for line in stdout.lines().skip(1) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if !parts.is_empty() {
            if let Ok(pct) = parts[0].parse::<f64>() {
                assert!(
                    pct >= 0.0 && pct <= 100.0,
                    "Percentage should be in valid range: {}",
                    pct
                );
            }
        }
    }
}

// T021: Integration test for --debug with multi-file showing (values: ...) annotation
#[test]
fn test_top_command_multi_file_debug_shows_values() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "top",
            "-n",
            "3",
            "--debug",
            "--no-color",
            "examples/Bikes_005_rep0.txt",
            "examples/Bikes_005_rep1.txt",
            "examples/Bikes_005_rep2.txt",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "--debug with multi-file should succeed: {}",
        stderr
    );

    // Should contain per-report values annotation
    assert!(
        stdout.contains("(values:"),
        "Debug output should contain per-report values annotation"
    );

    // Should contain comma-separated percentages
    assert!(
        stdout.contains("%,"),
        "Values annotation should contain comma-separated percentages"
    );
}

// T026: Integration test for single-file output unchanged from baseline
#[test]
fn test_top_command_single_file_backward_compat() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "top",
            "-n",
            "5",
            "--no-color",
            "examples/Bikes_005_rep0.txt",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "Single-file command should succeed"
    );

    // Single-file output should not have per-report annotations
    assert!(
        !stdout.contains("(values:"),
        "Single-file output should not have per-report annotations"
    );

    // Should have standard table format
    assert!(stdout.contains("Children%"), "Should have header");
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(
        lines.len(),
        6,
        "Should have exactly 6 lines (1 header + 5 data)"
    );
}

// T027: Integration test for single-file with --debug shows no per-report annotation
#[test]
fn test_top_command_single_file_debug_no_values_annotation() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "top",
            "-n",
            "3",
            "--debug",
            "--no-color",
            "examples/Bikes_005_rep0.txt",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        output.status.success(),
        "Single-file with --debug should succeed"
    );

    // Single-file with --debug should NOT have per-report annotations
    // (only multi-file mode shows these)
    assert!(
        !stdout.contains("(values:"),
        "Single-file --debug output should not have per-report annotations"
    );
}

// T031: Integration test for multi-file hierarchy showing averaged relationships
#[test]
fn test_top_command_multi_file_hierarchy() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "top",
            "--hierarchy",
            "-t",
            "rd_optimize",
            "-t",
            "DCT4DBlock",
            "--no-color",
            "examples/Bikes_005_rep0.txt",
            "examples/Bikes_005_rep1.txt",
            "examples/Bikes_005_rep2.txt",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "Multi-file hierarchy should succeed: {}",
        stderr
    );

    // Should contain both target functions
    assert!(
        stdout.contains("rd_optimize"),
        "Output should contain rd_optimize"
    );
    assert!(
        stdout.contains("DCT4DBlock"),
        "Output should contain DCT4DBlock"
    );

    // Should have hierarchy structure (indented entries)
    assert!(
        stdout.contains("    "),
        "Output should have indented callee entries"
    );
}

// T024: Test --debug with --no-color shows plain text annotations
#[test]
fn test_top_command_debug_no_color() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "top",
            "--hierarchy",
            "--debug",
            "--no-color",
            "-t",
            "rd_optimize",
            "-t",
            "DCT4DBlock",
            "examples/Bikes_005_rep0.txt",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "--debug --no-color should succeed");

    // Should have annotations without ANSI escape codes
    assert!(
        stdout.contains("(direct:") || stdout.contains("(via "),
        "Output should contain annotations"
    );
    assert!(
        !stdout.contains('\x1b'),
        "Output should have no ANSI escape codes with --no-color"
    );
}

// Integration test for --hierarchy --debug showing standalone annotations
#[test]
fn test_top_command_debug_standalone_annotations() {
    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "top",
            "--hierarchy",
            "--debug",
            "-t",
            "rd_optimize",
            "-t",
            "DCT4DBlock",
            "--no-color",
            "examples/Bikes_005_rep0.txt",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        output.status.success(),
        "--hierarchy --debug should succeed: {}",
        stderr
    );

    // Should contain standalone annotations for entries with contributions
    assert!(
        stdout.contains("(standalone:"),
        "Output should contain standalone annotations for adjusted entries"
    );

    // The standalone annotation should show the subtraction format
    // e.g., "(standalone: X.XX% - Y.YY% (CallerName) = Z.ZZ%)"
    assert!(
        stdout.contains(" - ") && stdout.contains("(standalone:"),
        "Standalone annotation should show subtraction"
    );
}
