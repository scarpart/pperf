# pperf - Perf Report Analyzer

A Rust CLI tool for analyzing `perf report` output files with simplified symbols and call hierarchy visualization.

## Quick Reference

```bash
# Basic usage - top 10 functions by Children%
pperf top perf-report.txt

# Sort by Self% instead
pperf top --self perf-report.txt

# Filter to specific functions
pperf top --targets rd_optimize DCT4D perf-report.txt

# Show call hierarchy between targets
pperf top --hierarchy --targets rd_optimize_transform DCT4DBlock perf-report.txt
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

## Perf Report Format

Perf reports have top-level entries with call trees:
```
71.80%  0.00%  binary  [.] TransformPartition::rd_optimize_transform
        |
        |--17.23%--DCT4DBlock::DCT4DBlock    <- relative to parent
```

The `hierarchy.rs` module parses these call trees and discovers relationships between target functions, handling recursive calls and intermediate (non-target) functions.

## Development

```bash
cargo build --release
cargo test
cargo clippy
```

**Constraints**: Standard library only (no external dependencies per constitution).
