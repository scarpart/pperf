# Quickstart: Clap CLI Refactor

**Feature**: 005-clap-cli-refactor
**Date**: 2026-01-03

## Overview

This feature replaces the manual CLI argument parsing in `main.rs` with Clap derive macros. The user-facing behavior remains identical.

## Prerequisites

- Rust stable toolchain (edition 2024)
- Existing pperf codebase with passing tests

## Implementation Steps

### 1. Add Clap Dependency

Update `Cargo.toml`:
```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
```

### 2. Define CLI Structures

In `main.rs`, add the Clap-derived structs:
```rust
use clap::{Parser, Subcommand, Args};

#[derive(Parser)]
#[command(name = "pperf", version, about = "Perf report analyzer")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Display top functions by CPU time
    Top(TopArgs),
}

#[derive(Args)]
struct TopArgs {
    /// Sort by Self% instead of Children%
    #[arg(short = 's', long = "self")]
    sort_self: bool,

    /// Number of functions to display
    #[arg(short = 'n', long, default_value = "10", value_parser = clap::value_parser!(usize).range(1..))]
    number: usize,

    /// Filter by function name substrings
    #[arg(short = 't', long, num_args = 1..)]
    targets: Option<Vec<String>>,

    /// Display call relationships between targets
    #[arg(short = 'H', long)]
    hierarchy: bool,

    /// Show calculation path for hierarchy percentages
    #[arg(short = 'D', long)]
    debug: bool,

    /// Disable colored output
    #[arg(long)]
    no_color: bool,

    /// Perf report file to analyze
    file: std::path::PathBuf,
}
```

### 3. Update main() Function

Replace manual parsing with Clap:
```rust
fn main() {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            e.print().expect("Failed to print error");
            std::process::exit(3);
        }
    };

    let result = match cli.command {
        Commands::Top(args) => run_top(args),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        // ... existing exit code logic
    }
}
```

### 4. Update run_top() Signature

Change `run_top` to accept the struct:
```rust
fn run_top(args: TopArgs) -> Result<(), PperfError> {
    // Validate hierarchy requires targets
    if args.hierarchy && args.targets.as_ref().map_or(true, |t| t.is_empty()) {
        return Err(PperfError::HierarchyRequiresTargets);
    }

    // Map to existing variables
    let sort_order = if args.sort_self { SortOrder::Self_ } else { SortOrder::Children };
    let count = args.number;
    let targets = args.targets.unwrap_or_default();
    let hierarchy_flag = args.hierarchy;
    let debug_flag = args.debug;
    let no_color_flag = args.no_color;
    let file_path = args.file;

    // ... rest of existing logic unchanged
}
```

### 5. Remove print_help() Function

Clap auto-generates help. Delete the manual `print_help()` function.

## Verification

1. Run existing tests: `cargo test`
2. Verify help output: `pperf --help` and `pperf top --help`
3. Verify all examples from CLAUDE.md Quick Reference
4. Check exit codes for error conditions

## Common Issues

### "Unknown option" for valid flags
Ensure short/long attributes match existing flags exactly.

### Exit code mismatch
Use `Cli::try_parse()` and handle errors manually to control exit codes.

### Targets parsing differs
If users report issues with `--targets func1 func2 file.txt`, they may need to use `--targets func1 func2 -- file.txt` with explicit separator, or put file first.
