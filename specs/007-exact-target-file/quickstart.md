# Quickstart: Exact Function Signature Target File

**Feature**: 007-exact-target-file
**Date**: 2026-01-08

## Overview

This feature adds a `--target-file` flag that accepts a file containing exact function signatures for precise targeting in perf report analysis. Unlike the existing `-t` substring matching, this mode requires each signature to match exactly one function, detecting and reporting ambiguity.

## Usage

### Basic Usage

1. Create a target file with exact function signatures:

```bash
cat > targets.txt << 'EOF'
# High-cost functions to analyze
Hierarchical4DEncoder::get_rd_for_below_inferior_bit_plane(LightfieldCoordinate<unsigned int> const&, LightfieldDimension<unsigned int, true> const&)
DCT4DBlock::DCT4DBlock(Block4D const&, double)
EOF
```

2. Run pperf with the target file:

```bash
pperf top --target-file targets.txt perf-report.txt
```

### With Hierarchy Analysis

```bash
pperf top --target-file targets.txt --hierarchy perf-report.txt
```

### Example Output

```
Children%   Self%  Function
   71.80    0.00  TransformPartition::rd_optimize_transform
   17.23    0.00      DCT4DBlock::DCT4DBlock
   38.29    5.21  DCT4DBlock::DCT4DBlock (standalone)
```

## Target File Format

```
# Comment lines (start with #)

# Full function signatures, one per line
Hierarchical4DEncoder::get_rd_for_below_inferior_bit_plane(LightfieldCoordinate<unsigned int> const&, LightfieldDimension<unsigned int, true> const&)

# Blank lines are ignored

DCT4DBlock::DCT4DBlock(Block4D const&, double)
```

**Rules**:
- One signature per line
- Lines starting with `#` are comments
- Empty lines are ignored
- Leading/trailing whitespace is trimmed

## Error Handling

### Ambiguous Signature

If a signature matches multiple functions:

```
Error: Ambiguous target signature 'DCT4DBlock'
Matches:
  - DCT4DBlock::DCT4DBlock(Block4D const&, double)
  - DCT4DBlock::inverse(Block4D&) const
Use the complete function signature.
```

Exit code: 5

### Unmatched Signature

If a signature matches no functions:

```
Error: No matches found for target signatures:
  - NonExistent::function()
```

Exit code: 6

### Conflicting Flags

```bash
pperf top --target-file targets.txt -t DCT4D perf-report.txt
# Error: The argument '--target-file' cannot be used with '--targets'
```

Exit code: 3

## Getting Exact Signatures

To obtain exact function signatures:

1. Run pperf without targets to see all top functions:
   ```bash
   pperf top perf-report.txt
   ```

2. Copy the full function signature from the output

3. Or use grep on the perf report:
   ```bash
   grep "function_name" perf-report.txt | head -5
   ```

## Migration from -t Flag

| Old Command | New Command |
|-------------|-------------|
| `pperf top -t DCT4D report.txt` | Create `targets.txt` with exact `DCT4DBlock::DCT4DBlock(Block4D const&, double)` |
| `pperf top -t func1 -t func2 report.txt` | Create `targets.txt` with both exact signatures |

**Note**: The `-t` flag still works for quick substring matching. Use `--target-file` when you need precise, unambiguous targeting.

## Files Changed

| File | Changes |
|------|---------|
| `src/main.rs` | Add `--target-file` argument, validation logic |
| `src/lib.rs` | Add new error variants |
| `src/filter.rs` | Add `parse_target_file()`, `filter_entries_exact()` |
| `tests/top_command.rs` | Integration tests for new flag |
