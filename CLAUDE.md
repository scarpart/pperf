# pperf - Perf Report Analyzer

A Rust CLI tool for analyzing `perf report` output files with simplified symbols and call hierarchy visualization.

## Quick Reference

```bash
# Basic usage - top 10 functions by Children%
pperf top perf-report.txt

# Sort by Self% instead
pperf top --self perf-report.txt

# Filter to specific functions (use -t for each target)
pperf top -t rd_optimize -t DCT4D perf-report.txt

# Show call hierarchy between targets
pperf top --hierarchy -t rd_optimize_transform -t DCT4DBlock perf-report.txt

# Debug mode - show calculation path breakdown
pperf top --hierarchy --debug -t rd_optimize -t DCT4DBlock -t inner_product perf-report.txt
```

## Architecture

```
src/
├── main.rs      # CLI entry point, argument parsing, orchestration
├── lib.rs       # Library root, error types (PperfError enum)
├── parser.rs    # Perf report parsing (parse_file, parse_line, PerfEntry)
├── filter.rs    # Target substring matching
├── symbol.rs    # Symbol simplification and color classification
├── output.rs    # Table formatting (format_table, format_hierarchy_table)
└── hierarchy.rs # Call tree parsing and relationship discovery
```

## Key Features

### Symbol Simplification (`symbol.rs`)
Strips template parameters, argument lists, return types, and clone suffixes from C++ symbols for readability.

### Colored Output (`symbol.rs`, `output.rs`)
Color-codes symbols by type: user functions (white), std:: (cyan), libc (yellow), hex addresses (red).

### Call Hierarchy (`hierarchy.rs`)
The `--hierarchy` flag shows caller-callee relationships:

```
Children%   Self%  Function
   71.80    0.00  TransformPartition::rd_optimize_transform
   17.23    0.00      DCT4DBlock::DCT4DBlock          <- 4-space indent, relative %
   25.92    0.00  DCT4DBlock::DCT4DBlock              <- standalone, adjusted %
```

**Key concepts:**
- **Relative %**: Callee's percentage of caller's time (shown indented)
- **Adjusted %**: Original % minus contributions already shown under callers (standalone entries)
- **Contribution calculation**: Groups all caller→callee relations by caller, takes MAX absolute_pct per caller (handles duplicate relations from different traversal contexts)
- **Context-specific nesting**: When A→B→C are all targets, C is shown under B with path-specific percentages
- **Remainder display**: Standalone entries show remainder callees (overall% - consumed%)
- **Recursive handling**: For recursive functions (e.g., rd_optimize→rd_optimize), uses direct percentage from perf
- **Deduplication**: Multiple entries with same simplified symbol → only first shown
- **Depth calculation**: Based on column position of `--XX.XX%--` pattern (÷11)

### Debug Mode (`--debug` flag)
Shows calculation path annotations for hierarchy percentages:
- **Direct calls**: `(direct: 17.23%)` - shown on gray line below direct caller→callee entries
- **Indirect calls**: `(via do_4d_transform 4.98% = 0.07%)` - shows intermediary chain for calls that traverse non-target functions
- **Standalone entries**: `(standalone: 38.00% - 12.37% (rd_optimize_transform) = 25.63%)` - shows subtraction breakdown for adjusted percentages
- Only active when combined with `--hierarchy`; has no effect in normal mode
- Annotations rendered in gray (DIM) color when color output is enabled

Example with `--debug`:
```
Children%   Self%  Function
   71.80    0.00  TransformPartition::rd_optimize_transform
   17.23    0.00      DCT4DBlock::DCT4DBlock
                      (direct: 17.23%)
    0.07    0.00          std::inner_product
                          (via Transformed4DBlock::do_4d_transform 4.98% = 0.07%)
   25.92    0.00  DCT4DBlock::DCT4DBlock
                  (standalone: 38.29% - 12.37% (TransformPartition::rd_optimize_transform) = 25.92%)
```

## Perf Report Format

Perf reports have top-level entries with call trees:
```
71.80%  0.00%  binary  [.] TransformPartition::rd_optimize_transform
        |
        |--17.23%--DCT4DBlock::DCT4DBlock    <- relative to parent
```

The `hierarchy.rs` module parses these call trees and discovers relationships between target functions, handling recursive calls and intermediate (non-target) functions.

## CLI Options

| Option | Short | Description |
|--------|-------|-------------|
| `--self` | `-s` | Sort by Self% instead of Children% |
| `--number <N>` | `-n` | Limit output to N entries (default: 10) |
| `--targets <name>` | `-t` | Filter to functions matching substring (repeatable) |
| `--hierarchy` | `-H` | Show call relationships between targets |
| `--debug` | `-D` | Show calculation path annotations (requires `--hierarchy`) |
| `--no-color` | | Disable ANSI color output |
| `--help` | `-h` | Show help message |
| `--version` | | Show version |

## Development

```bash
cargo build --release
cargo test
cargo clippy
```

**Dependencies**: clap v4 (with derive feature) for CLI argument parsing.

## Active Technologies
- Rust (stable, edition 2024)
- clap v4 (derive feature) for CLI parsing
- N/A (CLI tool, no persistent storage)

## Features Implemented
- **001**: Basic perf report parsing and table output
- **002**: Symbol simplification and colored output
- **003**: Call hierarchy display with `--hierarchy` flag
- **004**: Debug calculation path with `--debug` flag (shows percentage derivation annotations)
- **005**: Clap CLI refactor - replaced ad-hoc argument parsing with Clap derive macros
