# Quickstart: Targeted Call Hierarchy Display

**Feature**: 003-call-hierarchy

## Basic Usage

The `--hierarchy` flag shows call relationships between your target functions:

```bash
# Show hierarchy between specific functions
pperf top --hierarchy --targets rd_optimize_transform DCT4DBlock perf-report.txt
```

## Output Example

```
Children%   Self%  Function
   71.80    0.00  rd_optimize_transform
   17.23    0.00    DCT4DBlock              <- 17.23% of rd_optimize_transform's time
   25.63    5.00  DCT4DBlock                <- Remaining 25.63% absolute
```

## Understanding the Output

| Entry | Indentation | Percentage Meaning |
|-------|-------------|-------------------|
| Caller | None | Absolute % of total program time |
| Callee (indented) | 2 spaces | % of caller's time (relative) |
| Callee (standalone) | None | Adjusted absolute % |

### How Percentages Work

**Indented entries** show relative contribution:
- "DCT4DBlock takes 17.23% of rd_optimize_transform's time"

**Standalone entries** show adjusted absolute:
- Original: 38.00%
- Accounted under caller: 71.80% × 17.23% = 12.37%
- Adjusted: 38.00% - 12.37% = 25.63%

## Common Patterns

### Find relationships between hotspots

```bash
# See how top functions relate to each other
pperf top --hierarchy --targets get_mSubband inner_product DCT4DBlock perf-report.txt
```

### Focus on a specific caller

```bash
# Who does rd_optimize_transform call?
pperf top --hierarchy --targets rd_optimize_transform DCT4DBlock inner_product perf-report.txt
```

### Sort by self time

```bash
# Same hierarchy, sorted by Self%
pperf top --hierarchy --self --targets rd_optimize DCT4D perf-report.txt
```

## Key Behaviors

### Intermediate Functions Are Collapsed

If A → B → C and you target A and C (but not B):
- C appears under A with combined percentage (B is collapsed)
- Percentages are multiplied: if B is 50% of A and C is 40% of B, C is 20% of A

### Recursive Functions

Recursive calls are handled correctly:
- Only the first occurrence is counted
- Prevents double-counting and infinite nesting

### Multiple Callers

If function C is called by both A and B:
- C appears indented under A with A's relative %
- C appears indented under B with B's relative %
- C's standalone shows total minus all contributions

## Requirements

`--hierarchy` requires `--targets`:

```bash
# ERROR: --hierarchy requires --targets
pperf top --hierarchy perf-report.txt

# CORRECT: specify targets
pperf top --hierarchy --targets funcA funcB perf-report.txt
```

## Combine with Feature 002

Colors are applied automatically (user code in blue, libraries in yellow):

```bash
# Full featured output
pperf top --hierarchy --targets rd_optimize DCT4D --no-color perf-report.txt
```
