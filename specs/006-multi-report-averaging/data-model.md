# Data Model: Multi-Report Averaging

**Feature**: 006-multi-report-averaging
**Date**: 2026-01-05

## Entities

### Existing Entities (Unchanged)

#### PerfEntry

Represents a single function's profiling data from a perf report.

| Field | Type | Description |
|-------|------|-------------|
| children_pct | f64 | Percentage of time spent in function + callees |
| self_pct | f64 | Percentage of time spent in function only |
| symbol | String | Full function signature (used as unique identifier) |

**Location**: `src/parser.rs`

### New/Extended Entities

#### AveragedPerfEntry

Represents a function's aggregated profiling data across multiple reports.

| Field | Type | Description |
|-------|------|-------------|
| children_pct | f64 | Averaged Children% across reports |
| self_pct | f64 | Averaged Self% across reports |
| symbol | String | Full function signature (unique key for matching) |
| per_report_values | Vec<Option<(f64, f64)>> | Individual (children, self) per report; None if missing |
| report_count | usize | Number of reports containing this function |

**Derivation**: `children_pct = sum(present_children) / report_count`

**Design Notes**:
- `per_report_values` maintains file order (index corresponds to CLI argument order)
- `Option` allows distinguishing "not present" from "0%"
- For single-file input, this reduces to current `PerfEntry` behavior

#### ReportSet

Collection of parsed reports for aggregation.

| Field | Type | Description |
|-------|------|-------------|
| reports | Vec<Vec<PerfEntry>> | Parsed entries from each file, in file order |
| file_paths | Vec<PathBuf> | Original file paths (for error messages) |

**Operations**:
- `parse_all(paths: &[PathBuf]) -> Result<ReportSet, PperfError>`: Parse all files
- `average() -> Vec<AveragedPerfEntry>`: Compute averaged entries

## Data Flow

```text
Input: file1.txt, file2.txt, file3.txt
           │
           ▼
    ┌─────────────────┐
    │  parse_file()   │  ← For each file (existing function)
    │  per file       │
    └─────────────────┘
           │
           ▼
    ┌─────────────────┐
    │   ReportSet     │  ← New: Holds all parsed reports
    │                 │
    └─────────────────┘
           │
           ▼
    ┌─────────────────┐
    │  average()      │  ← New: Aggregate by symbol, compute means
    │                 │
    └─────────────────┘
           │
           ▼
    Vec<AveragedPerfEntry>
           │
           ├──────────────────────────┐
           ▼                          ▼
    ┌─────────────────┐      ┌─────────────────┐
    │  filter_entries │      │ build_hierarchy │
    │  (existing)     │      │ (extended)      │
    └─────────────────┘      └─────────────────┘
           │                          │
           ▼                          ▼
    ┌─────────────────┐      ┌─────────────────┐
    │  format_table   │      │ format_hierarchy│
    │  (extended)     │      │ (extended)      │
    └─────────────────┘      └─────────────────┘
           │                          │
           └──────────┬───────────────┘
                      ▼
                  Output (stdout)
```

## Aggregation Algorithm

```text
1. Initialize: symbol_data = HashMap<String, Vec<Option<(f64, f64)>>>

2. For each report (index i):
   a. Parse file → Vec<PerfEntry>
   b. For each PerfEntry:
      - Get or create symbol_data[entry.symbol]
      - Pad with None up to index i if needed
      - Set symbol_data[entry.symbol][i] = Some((children_pct, self_pct))

3. For each symbol in symbol_data:
   a. Count present values (non-None)
   b. Sum children_pct and self_pct for present values
   c. Compute averages: sum / count
   d. Create AveragedPerfEntry with averages and per_report_values
```

## State Transitions

Not applicable - this is a stateless CLI tool. All data flows through in a single pass.

## Validation Rules

| Rule | Applied At | Description |
|------|------------|-------------|
| File exists | parse_file() | Each file must exist and be readable |
| Valid format | parse_file() | Each file must be parseable perf report |
| At least one file | CLI parsing | clap `required = true` enforces this |
| Percentage range | Existing | 0.0 ≤ pct ≤ 100.0 (from perf output) |

## Relationship to Existing Code

### Modules Affected

| Module | Change Type | Description |
|--------|-------------|-------------|
| main.rs | Modify | Accept Vec<PathBuf> instead of single PathBuf |
| parser.rs | None | Reuse existing parse_file() |
| averaging.rs | New | ReportSet, AveragedPerfEntry, aggregation logic |
| output.rs | Modify | Format per_report_values in debug mode |
| hierarchy.rs | Modify | Accept averaged entries, merge call relations |
| filter.rs | Modify | Work with AveragedPerfEntry |
| symbol.rs | None | Unchanged |
| lib.rs | Modify | Export new module |

### Backward Compatibility

When `files.len() == 1`:
- `per_report_values` contains single `Some((children, self))`
- `report_count = 1`
- Averaged values equal original values
- No per-report debug output (matches current behavior)
