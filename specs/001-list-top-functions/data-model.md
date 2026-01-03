# Data Model: List Top Functions

**Feature**: 001-list-top-functions
**Date**: 2026-01-02

## Core Entities

### PerfEntry

Represents a single function's profiling data extracted from a perf report line.

| Field | Type | Description | Constraints |
|-------|------|-------------|-------------|
| children_pct | f64 | Inclusive CPU time percentage | 0.0 to 100.0 |
| self_pct | f64 | Exclusive CPU time percentage | 0.0 to 100.0 |
| symbol | String | Full function name/symbol | Non-empty |

**Derived Properties**:
- `base_name`: Symbol up to first `(` for grouping overloads
- `is_unresolved`: True if symbol is a hex address (starts with `0x`)

**Notes**:
- Command and Shared Object columns are parsed but not stored (not needed for output)
- Clone suffixes like `[clone .isra.0]` are part of the symbol

### SortOrder

Enum specifying how to sort entries.

| Variant | Description |
|---------|-------------|
| Children | Sort by children_pct descending (default) |
| Self | Sort by self_pct descending |

**Tie-breaking**: When primary sort values are equal, secondary sort by the other percentage.

### FilterMatch

Represents a filter target and its matching behavior.

| Field | Type | Description |
|-------|------|-------------|
| pattern | String | User-provided prefix or full name |

**Matching Logic**:
1. If pattern equals symbol exactly → match
2. If symbol starts with pattern → match
3. Otherwise → no match

### TopCommandOptions

Configuration for the `top` subcommand.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| file_path | PathBuf | (required) | Path to perf report file |
| count | usize | 10 | Number of entries to display |
| sort_by | SortOrder | Children | Sort order |
| targets | Vec<String> | [] | Filter patterns (empty = no filter) |

## Relationships

```text
TopCommandOptions
    │
    ├── file_path ──→ [perf-report.txt file]
    │
    └── targets ──→ FilterMatch (0..*)
                        │
                        └── matches ──→ PerfEntry (0..*)

PerfEntry (1..*) ──sorted by──→ SortOrder
```

## State Transitions

This feature is stateless. Each invocation:
1. Reads file
2. Parses entries
3. Filters (optional)
4. Sorts
5. Outputs

No persistent state between invocations.

## Validation Rules

### PerfEntry
- `children_pct` and `self_pct` must be valid f64 values
- `symbol` must be non-empty after trimming

### TopCommandOptions
- `file_path` must exist and be readable
- `count` must be > 0
- `targets` entries must be non-empty strings

## Error Types

| Error | Condition | User Message |
|-------|-----------|--------------|
| FileNotFound | File path doesn't exist | "File not found: {path}" |
| InvalidFormat | No valid perf entries found | "Invalid perf report format" |
| InvalidCount | count <= 0 | "Invalid value for -n: expected positive integer" |
| NoMatches | Filters match zero entries | "No matching functions found" |

## Output Format

Table with aligned columns, no borders (ls -l style):

```text
Children%   Self%  Function
   90.74    0.00  parallel_for_with_progress<...>
   71.80    0.00  TransformPartition::rd_optimize_transform #1
   71.78    0.00  TransformPartition::rd_optimize_transform #2
   21.72   11.94  Hierarchical4DEncoder::get_mSubbandLF_significance
```

**Column Widths**:
- Children%: 9 chars (right-aligned, 2 decimal places)
- Self%: 7 chars (right-aligned, 2 decimal places)
- Function: Variable (left-aligned, truncated if > 100 chars)

**Truncation**: Long function names truncated with `...` to fit 120-column terminal.
