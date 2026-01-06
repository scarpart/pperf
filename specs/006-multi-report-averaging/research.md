# Research: Multi-Report Averaging

**Feature**: 006-multi-report-averaging
**Date**: 2026-01-05

## Research Questions

### 1. Function Matching Strategy

**Question**: How should functions be uniquely identified across different report files?

**Decision**: Use the raw `symbol` field from `PerfEntry` as the unique key.

**Rationale**:
- The `symbol` field contains the full function signature (e.g., `TransformPartition::rd_optimize_transform(Block4D const&)`)
- This is already extracted by `parse_line()` in `parser.rs`
- Simplified names in `symbol.rs` are for display only; matching uses full signatures
- Full signatures handle overloaded functions correctly (different parameter types = different functions)

**Alternatives Considered**:
- Simplified names: Rejected because overloaded functions would collide
- Hash of signature: Rejected as unnecessary complexity; strings are sufficient keys

### 2. Data Structure for Aggregation

**Question**: How should per-report data be stored before averaging?

**Decision**: Use `HashMap<String, Vec<(f64, f64)>>` where key is symbol, value is list of (children_pct, self_pct) tuples from each report.

**Rationale**:
- Simple and direct - each symbol maps to its percentage values across all reports
- Vec maintains insertion order for debug output (file order)
- No new struct needed; use tuples for lightweight storage
- O(1) lookup for symbol matching

**Alternatives Considered**:
- Nested HashMap (symbol → report_index → percentages): Rejected as over-engineered
- New AveragedEntry struct with per-report storage: Acceptable but deferred until needed

### 3. Handling Functions Missing from Some Reports

**Question**: How should averaging work when a function appears in only some reports?

**Decision**: Average only over the reports where the function appears. Track report count per function.

**Rationale**:
- A function missing from a report likely means it wasn't sampled (not that it took 0%)
- Treating missing as 0% would artificially deflate percentages
- Debug mode shows which reports contained the function (values vs `-`)

**Alternatives Considered**:
- Treat missing as 0%: Rejected as statistically misleading
- Require functions in all reports: Rejected as too restrictive for real-world use

### 4. Hierarchy Mode with Averaged Data

**Question**: How should hierarchy calculations work with averaged data?

**Decision**: First average top-level percentages, then compute hierarchy relationships.

**Rationale**:
- Hierarchy parsing (`parse_file_call_trees`) requires file content, not just entries
- For multi-file, need to merge/average call tree relationships too
- Call relations (caller→callee percentages) should also be averaged across reports

**Implementation Approach**:
1. Parse call trees from each file separately
2. Merge call relations by averaging percentages for matching caller→callee pairs
3. Build hierarchy entries from averaged relations

### 5. CLI Argument Design

**Question**: How should multiple files be specified?

**Decision**: Change `file: PathBuf` to `files: Vec<PathBuf>` with `#[arg(required = true)]`.

**Rationale**:
- Natural extension of current interface
- Clap handles multiple positional arguments natively
- Maintains backward compatibility (1 file still works)
- No new flags needed

**Command Examples**:
```bash
# Single file (backward compatible)
pperf top file.txt

# Multiple files (new)
pperf top rep0.txt rep1.txt rep2.txt

# With flags
pperf top --hierarchy -t rd_optimize rep0.txt rep1.txt rep2.txt
```

### 6. Debug Output Format for Per-Report Values

**Question**: How should per-report breakdowns be displayed?

**Decision**: Show on annotation line below function: `(values: 73.86%, 73.60%, 70.40%)`

**Rationale**:
- Consistent with existing debug annotation style (gray, indented)
- Shows values in file order (matches command-line order)
- Missing values shown as `-` for clarity
- Applies to both standalone entries and hierarchy entries

**Format Examples**:
```
Children%   Self%  Function
   72.62    0.00  TransformPartition::rd_optimize_transform
                  (values: 73.86%, 73.60%, 70.40%)
```

For hierarchy with multi-file:
```
   72.62    0.00  TransformPartition::rd_optimize_transform
   17.50    0.00      DCT4DBlock::DCT4DBlock
                      (direct: 17.50%)
                      (values: 17.23%, 17.82%, 17.45%)
```

### 7. Error Handling for Invalid Files

**Question**: What happens if one of multiple files is invalid?

**Decision**: Fail immediately with error identifying the problematic file.

**Rationale**:
- Fail-fast prevents silent incorrect results
- User can fix the issue and retry
- Partial results would be confusing

**Error Message**: `Error: File not found: path/to/missing.txt` (existing PperfError::FileNotFound)

## Summary of Key Decisions

| Topic | Decision | Key Reason |
|-------|----------|------------|
| Function matching | Full symbol signature | Handles overloads correctly |
| Aggregation structure | HashMap<String, Vec<(f64, f64)>> | Simple, O(1) lookup |
| Missing functions | Average only present reports | Statistically sound |
| Hierarchy mode | Average after parsing all trees | Preserves relationship accuracy |
| CLI design | Vec<PathBuf> positional args | Backward compatible |
| Debug format | `(values: ...)` annotation line | Consistent with existing style |
| Error handling | Fail-fast on first error | Prevents silent corruption |
