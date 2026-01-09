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

# Filter using exact function signatures from a file
pperf top --target-file targets.txt perf-report.txt

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
├── filter.rs    # Target filtering (substring and exact signature matching)
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
- **Indirect calls**: `(via do_4d_transform 5.29% × 1.73% = 0.09%)` - shows intermediary chain PLUS callee's direct percentage for calls that traverse non-target functions
- **Standalone entries**: `(standalone: 38.00% - 12.37% (rd_optimize_transform) = 25.63%)` - shows subtraction breakdown for adjusted percentages
- Only active when combined with `--hierarchy`; has no effect in normal mode
- Annotations rendered in gray (DIM) color when color output is enabled

Example with `--debug`:
```
Children%   Self%  Function
   73.86    0.00  TransformPartition::rd_optimize_transform
    5.31    0.00      DCT4DBlock::DCT4DBlock
                      (direct: 5.31%)
    0.09    0.00          std::inner_product
                          (via do_4d_transform 5.29% × 1.73% = 0.09%)
   21.20    0.00  DCT4DBlock::DCT4DBlock
                  (standalone: 25.12% - 3.92% (rd_optimize_transform) = 21.20%)
    2.96    0.00      std::inner_product
                      (via do_4d_transform 24.80% × 10.07% = 2.96%)
```

## Percentage Semantics and Computation

### Design Specification

From `examples/software-design.txt`, the intended output format for A→B→C where A also calls C directly:

```
XX% YY% functionA    // Children% relative to total execution time
XX% YY%   functionB  // relative% of caller's time (A's time going to B)
XX% YY%     functionC // relative% of B's time (B's time going to C)
XX% YY%   functionC  // relative% of A's time (A directly calling C, not via B)
XX% YY% functionB    // standalone: B's remaining time not reached via A
XX% YY%   functionC  // relative% of standalone B's time going to C
XX% YY% functionC    // standalone: C's remaining time after subtracting contributions
```

### Two Types of Percentages

| Type | Reference Frame | Example |
|------|-----------------|---------|
| **Standalone/Entry %** | Total execution time (100%) | `51.52% rd_optimize_transform` |
| **Callee/Relative %** | Parent caller's time | `47.31% evaluate_split` (under rd_optimize) |

**Key rule**: Callee percentages are ALWAYS relative to their immediate caller, not to total execution time.

### Standalone Percentage Computation

```
standalone% = original_children% - sum(contributions from target callers)

contribution = caller_original% × callee_relative% / 100
```

Example for `evaluate_split_for_partitions`:
- Original Children%: 47.31%
- Called by `rd_optimize_transform` at 47.31% of rd_optimize's time
- Contribution: 73.86% × 47.31% / 100 = 34.94%
- Standalone: 47.31% - 34.94% = **12.37%**

### Why Standalone Percentages Can Sum to >100%

Given these standalone values:
```
51.52%  rd_optimize_transform
42.73%  rd_optimize_hexadecatree
12.37%  evaluate_split_for_partitions
30.40%  get_mSubbandLF_significance
21.20%  DCT4DBlock
 9.44%  inner_product
------
167.66%  TOTAL (exceeds 100%)
```

**Root cause**: Caller standalones INCLUDE their callee's time by design.

```
rd_optimize_transform standalone (51.52%) INCLUDES:
├─ rd_optimize exclusive time
├─ time spent in evaluate_split (13.08% absolute)
├─ time spent in hexadecatree (13.08% absolute)
├─ time spent in DCT4DBlock (3.92% absolute)
└─ ... all callee time

hexadecatree standalone (42.73%) = hexadecatree time NOT via rd_optimize
```

The overlap is intentional:
- rd_optimize's 51.52% includes 13.08% in hexadecatree (when rd_optimize calls it)
- hexadecatree's 42.73% is time NOT via rd_optimize (different call paths)
- Both are shown because they answer different questions

**Verification**: hexadecatree total = 13.08% (via rd_optimize) + 42.73% (standalone) = 55.81% ✓

**What standalone subtracts**: Only contributions FROM target callers to the callee:
- If target A calls target B, we subtract A's contribution from B's standalone
- We do NOT subtract B's contribution from A's standalone (A keeps full Children%)

**Why this design?**
1. **Full function profiles**: Callers show their total time including all callees
2. **Attribution breakdown**: Callees shown indented with their contribution
3. **Remaining time**: Callee standalones show time from OTHER call paths

If we wanted sum ≤ 100%, we'd need to show only EXCLUSIVE time for callers, losing the ability to show the call relationship hierarchy.

**This is expected behavior**, not a bug. The hierarchy answers: *"What does this function's execution profile look like, and what remains after attribution?"* — not *"What disjoint partition of execution does each function represent?"*

### Callee Display Under Standalone Entries

Callees under standalone entries show their **original relative percentage** (relative to caller's TOTAL time, not standalone time):

```
   12.37    0.00  evaluate_split_for_partitions  <- 12.37% standalone
   47.23    0.00      rd_optimize_transform      <- 47.23% of evaluate_split's TOTAL time
```

**Why 47.23% > 12.37%**: Different reference frames:
- 12.37% = standalone (relative to 100% total execution)
- 47.23% = callee's share of evaluate_split's original 47.31% time

We cannot compute "callee % relative to standalone" because perf data doesn't tell us how the standalone portion distributes among callees.

### Multi-Level Nesting

Both first pass (root callers) and second pass (standalone entries) support recursive nesting:

```
   12.37    0.00  evaluate_split_for_partitions     <- standalone entry
   47.23    0.00      rd_optimize_transform         <- level 1 callee
   17.71    0.00          rd_optimize_hexadecatree  <- level 2 (nested)
    0.65    0.00              get_mSubbandLF_significance  <- level 3
```

Implementation: `display_standalone_callees_recursive` in `output.rs` uses `callee_to_callee_map` to look up nested callees.

### Contribution Tracking

To avoid showing the same callee multiple times:
1. First pass tracks `consumed_absolute` for each callee shown under root callers
2. Second pass checks `remainder = absolute_pct - consumed` before displaying
3. Only callees with `remainder > 0.01` are shown

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
| `--target-file <path>` | | Filter using exact function signatures from a file (one per line) |
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
- **007**: Exact target file matching via `--target-file` flag - reduces ambiguity in function name matching by using exact function signatures from a file
- **008**: Multi-level nesting for standalone entries - callees under standalone entries now recursively show their own callees, matching the design spec in `examples/software-design.txt`
