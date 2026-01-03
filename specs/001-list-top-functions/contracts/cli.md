# CLI Contract: pperf

**Feature**: 001-list-top-functions
**Date**: 2026-01-02

## Command Structure

```text
pperf <SUBCOMMAND> [OPTIONS] <FILE>
```

## Subcommand: top

Display the top functions by CPU time from a perf report.

### Synopsis

```text
pperf top [OPTIONS] <FILE>
```

### Arguments

| Argument | Required | Description |
|----------|----------|-------------|
| FILE | Yes | Path to perf report text file |

### Options

| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| --self | -s | flag | false | Sort by Self% instead of Children% |
| --number | -n | integer | 10 | Number of functions to display |
| --targets | -t | string[] | [] | Filter by function name prefixes |
| --help | -h | flag | - | Show help message |

### Examples

```bash
# Show top 10 functions by Children% (default)
pperf top perf-report.txt

# Show top 5 functions by Self%
pperf top --self -n 5 perf-report.txt

# Filter to specific functions
pperf top --targets DCT4DBlock Hierarchical perf-report.txt

# Combine options
pperf top -s -n 20 -t TransformPartition perf-report.txt
```

### Output Format

Standard output, tabular format:

```text
Children%   Self%  Function
   90.74    0.00  parallel_for_with_progress<false, std::vector<...>>
   71.80    0.00  TransformPartition::rd_optimize_transform #1
   21.72   11.94  Hierarchical4DEncoder::get_mSubbandLF_significance
```

- Header row always present
- Percentages right-aligned with 2 decimal places
- Function names left-aligned, truncated with `...` if > 100 chars
- Duplicate base names distinguished with `#N` suffix

### Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | File not found |
| 2 | Invalid perf report format |
| 3 | Invalid arguments |
| 4 | No matching functions (when using --targets) |

### Error Messages

All errors written to stderr:

```text
Error: File not found: nonexistent.txt
Error: Invalid perf report format
Error: Invalid value for -n: expected positive integer
Error: No matching functions found
```

## Future Subcommands (Out of Scope)

Reserved for future features:
- `pperf tree` - Show call tree hierarchy
- `pperf compare` - Compare two perf reports
- `pperf filter` - Extract specific functions to new file

## Argument Parsing Notes

### --targets Behavior

The `--targets` option accepts multiple values:

```bash
# All of these are equivalent:
pperf top --targets DCT4DBlock Hierarchical file.txt
pperf top -t DCT4DBlock -t Hierarchical file.txt
pperf top --targets=DCT4DBlock --targets=Hierarchical file.txt
```

The FILE argument must come after all options.

### Prefix Matching

Target patterns match function name prefixes:
- `DCT4D` matches `DCT4DBlock::DCT4DBlock`, `DCT4DBlock::transform`, etc.
- `TransformPartition::rd` matches `TransformPartition::rd_optimize_transform`
- Exact match: `DCT4DBlock::DCT4DBlock(Block4D const&, double)` matches only that signature

### Version Information

```bash
pperf --version
# Output: pperf 0.1.0
```
