# Quickstart: Multi-Report Averaging

**Feature**: 006-multi-report-averaging
**Date**: 2026-01-05

## Overview

This feature enables pperf to analyze multiple perf report files from separate runs, averaging the metrics to provide statistically meaningful profiling data.

## Usage

### Basic Multi-File Averaging

```bash
# Analyze 3 report files, display averaged percentages
pperf top examples/Bikes_005_rep0.txt examples/Bikes_005_rep1.txt examples/Bikes_005_rep2.txt
```

Output shows averaged Children% and Self% for each function.

### With Target Filtering

```bash
# Filter to specific functions across all reports
pperf top -t rd_optimize -t DCT4DBlock examples/Bikes_005_rep*.txt
```

### Hierarchy Mode with Multiple Files

```bash
# Show call relationships with averaged percentages
pperf top --hierarchy -t rd_optimize -t DCT4DBlock examples/Bikes_005_rep*.txt
```

### Debug Mode (Per-Report Breakdown)

```bash
# Show individual values from each report alongside averages
pperf top --debug -t rd_optimize examples/Bikes_005_rep*.txt
```

Example output:
```
Children%   Self%  Function
   72.62    0.00  TransformPartition::rd_optimize_transform
                  (values: 73.86%, 73.60%, 70.40%)
```

### Combined Hierarchy + Debug

```bash
pperf top --hierarchy --debug -t rd_optimize -t DCT4DBlock examples/Bikes_005_rep*.txt
```

Shows per-report breakdowns for both hierarchy percentages and standalone entries.

## Single File (Backward Compatible)

```bash
# Works exactly as before
pperf top examples/Bikes_005_rep0.txt
```

No averaging annotations; identical to previous behavior.

## Quick Reference

| Scenario | Command |
|----------|---------|
| Average 3 files | `pperf top file1.txt file2.txt file3.txt` |
| Glob pattern | `pperf top reports/*.txt` |
| With filters | `pperf top -t func1 -t func2 file*.txt` |
| See per-report values | `pperf top --debug file*.txt` |
| Hierarchy + average | `pperf top -H -t target file*.txt` |

## Implementation Entry Points

| Task | File | Function/Section |
|------|------|------------------|
| CLI argument change | main.rs | TopArgs struct |
| Averaging logic | averaging.rs | New module |
| Debug format | output.rs | format_table, format_hierarchy_table |
| Integration tests | tests/top_command.rs | New multi-file tests |
