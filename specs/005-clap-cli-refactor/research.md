# Research: Clap CLI Refactor

**Feature**: 005-clap-cli-refactor
**Date**: 2026-01-03

## Research Tasks

### 1. Clap Derive Best Practices for Subcommand CLIs

**Decision**: Use Clap v4 with derive macros for declarative CLI definition.

**Rationale**:
- Derive macros reduce boilerplate and improve maintainability
- Type-safe argument parsing with compile-time validation
- Automatic help and version generation
- Subcommand pattern is well-supported via `#[derive(Subcommand)]`

**Alternatives Considered**:
- Clap builder API: More verbose, same functionality, rejected for simplicity
- Other crates (structopt, argh): Clap is the de-facto standard; structopt merged into Clap v3+

**Implementation Pattern**:
```rust
#[derive(Parser)]
#[command(name = "pperf", version, about = "Perf report analyzer")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Top(TopArgs),
}

#[derive(Args)]
struct TopArgs {
    // options here
}
```

### 2. Handling --targets with Variable Arguments + Trailing File

**Decision**: Use Clap's `num_args` with a required positional `file` argument.

**Rationale**:
The current behavior is:
- `--targets func1 func2 file.txt` where `file.txt` is consumed as the file
- The code detects a file path by checking `Path::new(&args[i]).exists()`

This is problematic because:
- User must ensure file exists for correct parsing
- Ambiguous if a target happens to match an existing file name

**Solution**: Make `file` a required positional argument that must come last:
```rust
#[derive(Args)]
struct TopArgs {
    /// Filter by function name substrings
    #[arg(short = 't', long = "targets", num_args = 1..)]
    targets: Vec<String>,

    /// Perf report file to analyze
    file: PathBuf,
}
```

This means users must use explicit syntax:
- `pperf top --targets func1 func2 -- file.txt` (explicit separator)
- `pperf top -t func1 -t func2 file.txt` (repeated flag)
- `pperf top file.txt --targets func1 func2` (file first)

**Compatibility Note**: The current ad-hoc parsing allows `--targets func1 func2 file.txt` without explicit separator because it checks file existence. Clap cannot replicate this behavior without custom validation. We'll use `trailing_var_arg = true` on targets to consume remaining args, then pop the last one as the file path via custom logic.

**Final Approach**: Use `last = true` for the file argument and `num_args = 0..` for targets with custom post-processing:
```rust
#[arg(short = 't', long = "targets", num_args = 0..)]
targets: Vec<String>,

#[arg(last = true)]
file: PathBuf,
```

Actually, after further analysis: Clap's `trailing_var_arg` on the parent can help, but the cleanest approach is to keep targets as optional multi-value and make file positional. This changes the syntax slightly but is cleaner:

**Adopted Approach**:
```rust
#[arg(short = 't', long, num_args = 1..)]
targets: Option<Vec<String>>,

/// Perf report file to analyze
file: PathBuf,
```

With this, users can do:
- `pperf top file.txt` (no targets)
- `pperf top --targets func1 func2 file.txt` (targets before file - works!)
- `pperf top file.txt --targets func1 func2` (file before targets - also works!)

Clap handles this correctly because the positional `file` is required and unambiguous.

### 3. Preserving Exit Codes

**Decision**: Handle Clap errors with custom exit codes matching current behavior.

**Rationale**: Clap uses exit code 2 for argument errors by default. We need exit code 3.

**Implementation**:
```rust
fn main() {
    let cli = Cli::try_parse();

    match cli {
        Ok(cli) => run(cli),
        Err(e) => {
            e.print().unwrap();
            std::process::exit(3);  // Argument errors -> exit 3
        }
    }
}
```

Or use Clap's error customization to set exit code.

### 4. Clap Dependency Specification

**Decision**: Use `clap = { version = "4", features = ["derive"] }`

**Rationale**:
- Version 4 is stable and widely used
- `derive` feature enables derive macros
- No need for `cargo`, `env`, or other features
- Minimal dependency footprint

**Cargo.toml addition**:
```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
```

### 5. Count Validation (n >= 1)

**Decision**: Use Clap's `value_parser` with `RangeFrom` for count validation.

**Implementation**:
```rust
#[arg(short = 'n', long = "number", default_value = "10", value_parser = clap::value_parser!(usize).range(1..))]
number: usize,
```

This rejects 0 with an automatic error message.

## Summary

All NEEDS CLARIFICATION items have been resolved:
- Clap v4 with derive macros
- Standard positional file argument with multi-value targets option
- Custom exit code handling for argument errors
- Value parser for count validation
