# CLI Contract: Targeted Call Hierarchy Display

**Feature**: 003-call-hierarchy
**Date**: 2026-01-02

## Command Updates

### `pperf top` (Updated)

```
pperf top [OPTIONS] <FILE>
```

#### New Option

| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--hierarchy` | `-H` | flag | false | Display call relationships between target functions |

#### Option Dependency

| Condition | Behavior |
|-----------|----------|
| `--hierarchy` without `--targets` | Error: "--hierarchy requires --targets" |
| `--hierarchy` with `--targets` | Display targeted call hierarchy |
| `--targets` without `--hierarchy` | Existing flat output (unchanged) |

---

## Output Format

### Standard Output (Without --hierarchy)

Unchanged from feature 002:

```
Children%   Self%  Function
   71.80    0.00  rd_optimize_transform
   38.00    5.00  DCT4DBlock
```

### Hierarchy Output (With --hierarchy --targets)

```
Children%   Self%  Function
   71.80    0.00  rd_optimize_transform
   17.23    0.00    DCT4DBlock
   25.63    5.00  DCT4DBlock
```

#### Interpretation

| Entry Type | Indentation | Percentage Meaning |
|------------|-------------|-------------------|
| Caller (standalone) | None (0 spaces) | Absolute % of total program time |
| Callee (indented) | 2 spaces before name | Relative % of caller's time |
| Callee (standalone) | None (0 spaces) | Adjusted absolute % (original - accounted) |

---

## Output Structure

### Line Format

```
[Children%] [Self%]  [Indent][Function]
```

Where:
- `[Children%]`: 8 chars, right-aligned, 2 decimal places
- `[Self%]`: 8 chars, right-aligned, 2 decimal places
- `[Indent]`: 0 or 2 spaces depending on entry type
- `[Function]`: Simplified function name with color coding

### Example with Actual Data

Given targets `rd_optimize_transform` (71.80% absolute) and `DCT4DBlock`:

```
Children%   Self%  Function
   71.80    0.00  rd_optimize_transform          <- Caller, absolute %
   17.23    0.00    DCT4DBlock                   <- Callee, relative % of caller
   25.63    5.00  DCT4DBlock                     <- Standalone adjusted absolute %
```

**Calculation**:
- DCT4DBlock original standalone = 38.00%
- DCT4DBlock contribution under rd_optimize_transform = 17.23% relative
- Absolute contribution = 71.80% × 17.23% / 100 = 12.37%
- Adjusted standalone = 38.00% - 12.37% = 25.63%

---

## Sorting Behavior

| Flag | Primary Sort | Callee Placement |
|------|--------------|------------------|
| Default | Children% (descending) | Immediately after caller |
| `--self` | Self% (descending) | Immediately after caller |

Callees are NOT re-sorted; they appear in order under their caller, then the next standalone entry follows.

---

## Edge Cases

| Case | Behavior |
|------|----------|
| No call relationships between targets | Flat output (each target standalone) |
| Target has no targeted callees | Displayed without indented entries |
| Adjusted percentage < 0 | Display as 0.00% |
| Recursive target (A→A) | Show once as callee under itself |
| Multiple callers of same callee | Callee appears indented under each caller |
| Perf report lacks call tree data | Warning message, flat output |

---

## Error Messages

| Condition | Exit Code | Message |
|-----------|-----------|---------|
| `--hierarchy` without `--targets` | 3 | "error: --hierarchy requires --targets to be specified" |
| No targets found in perf report | 4 | "error: no functions matching targets found" |
| No call tree data available | 0 | "warning: no call tree data found, showing flat output" |

---

## Help Text Update

```
OPTIONS:
    --self, -s           Sort by Self% instead of Children%
    -n, --number <N>     Number of functions to display (default: 10)
    --targets, -t <N>... Filter by function name substrings
    --hierarchy, -H      Display call relationships between targets
    --no-color           Disable colored output
    --help, -h           Show this help message
    --version            Show version information

EXAMPLES:
    pperf top perf-report.txt
    pperf top --self -n 20 perf-report.txt
    pperf top --targets DCT4DBlock rd_optimize perf-report.txt
    pperf top --hierarchy --targets DCT4DBlock rd_optimize perf-report.txt
```

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | File not found |
| 2 | Invalid format |
| 3 | Invalid arguments (includes --hierarchy without --targets) |
| 4 | No matches (with --targets) |
