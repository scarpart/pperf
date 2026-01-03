# Data Model: Clap CLI Refactor

**Feature**: 005-clap-cli-refactor
**Date**: 2026-01-03

## CLI Structure

This feature defines CLI argument structures using Clap's derive macros. These replace the manual argument parsing in `main.rs`.

### Entities

#### Cli (Root Parser)

The top-level CLI structure that handles global options and subcommands.

| Field | Type | Description |
|-------|------|-------------|
| command | Commands | The subcommand to execute |

**Attributes**:
- name: "pperf"
- version: From Cargo.toml
- about: "Perf report analyzer"

#### Commands (Subcommand Enum)

Enumeration of available subcommands.

| Variant | Associated Type | Description |
|---------|-----------------|-------------|
| Top | TopArgs | Display top functions by CPU time |

**Future extensibility**: Additional subcommands can be added as new variants.

#### TopArgs (Top Subcommand Arguments)

Arguments for the `top` subcommand.

| Field | Type | Default | Short | Long | Description |
|-------|------|---------|-------|------|-------------|
| sort_self | bool | false | s | self | Sort by Self% instead of Children% |
| number | usize | 10 | n | number | Number of functions to display |
| targets | Option<Vec<String>> | None | t | targets | Filter by function name substrings |
| hierarchy | bool | false | H | hierarchy | Display call relationships between targets |
| debug | bool | false | D | debug | Show calculation path for hierarchy percentages |
| no_color | bool | false | - | no-color | Disable colored output |
| file | PathBuf | required | - | - | Perf report file to analyze (positional) |

**Validation Rules**:
- `number` must be >= 1 (enforced via value_parser range)
- `file` must be provided (positional, required)
- `hierarchy` requires `targets` to be non-empty (post-parse validation)

### State Transitions

N/A - This is a CLI parser with no state machine.

### Relationships

```
Cli
  └── Commands (enum, 1:1)
        └── Top(TopArgs) (variant)
              ├── targets: Option<Vec<String>> (0..n filter terms)
              └── file: PathBuf (required input file)
```

### Mapping to Existing Types

| Clap Struct Field | Maps To | Notes |
|-------------------|---------|-------|
| TopArgs.sort_self | SortOrder::Self_ | When true |
| TopArgs.sort_self (false) | SortOrder::Children | When false |
| TopArgs.number | count: usize | Direct mapping |
| TopArgs.targets | targets: Vec<String> | Unwrap or empty vec |
| TopArgs.hierarchy | hierarchy_flag | Direct mapping |
| TopArgs.debug | debug_flag | Direct mapping |
| TopArgs.no_color | no_color_flag | Direct mapping |
| TopArgs.file | file_path: &str | Convert PathBuf to str |

### Error Mapping

| Clap Error | Current Error | Exit Code |
|------------|---------------|-----------|
| Missing subcommand | "Usage: pperf <subcommand>..." | 3 |
| Unknown subcommand | "Unknown subcommand: X" | 3 |
| Missing file argument | "Usage: pperf top..." | 3 |
| Invalid --number value | PperfError::InvalidCount | 3 |
| Unknown option | "Unknown option: X" | 3 |

| Post-Parse Error | Current Error | Exit Code |
|------------------|---------------|-----------|
| --hierarchy without --targets | PperfError::HierarchyRequiresTargets | 3 |
| File not found | PperfError::FileNotFound | 1 |
| Invalid file format | PperfError::InvalidFormat | 2 |
| No matching functions | PperfError::NoMatches | 4 |
