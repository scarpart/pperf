# Quickstart: pperf top

**Feature**: 001-list-top-functions

## Prerequisites

1. Rust toolchain (latest stable)
2. A perf report text file (from `perf report > report.txt`)

## Build

```bash
cargo build --release
```

Binary located at `target/release/pperf`.

## Basic Usage

### View Top 10 Functions

```bash
pperf top perf-report.txt
```

Output:
```text
Children%   Self%  Function
   90.74    0.00  parallel_for_with_progress<false, std::vector<...>>
   71.80    0.00  TransformPartition::rd_optimize_transform #1
   71.78    0.00  TransformPartition::rd_optimize_transform #2
   49.34    0.00  TransformPartition::evaluate_split_for_partitions<...>
   49.34    0.00  TransformPartition::evaluate_split<(PartitionFlag)1>
   38.29    0.00  DCT4DBlock::DCT4DBlock
   37.51    0.03  Hierarchical4DEncoder::rd_optimize_hexadecatree
   21.72   11.94  Hierarchical4DEncoder::get_mSubbandLF_significance
   20.94    0.02  Hierarchical4DEncoder::rd_optimize_hexadecatree::{lambda}
   20.12    0.00  Transformed4DBlock::do_4d_transform
```

### Sort by Self Time

To find functions that consume CPU directly (not via callees):

```bash
pperf top --self perf-report.txt
```

### Limit Results

```bash
pperf top -n 5 perf-report.txt
```

### Filter by Function Name

Focus on specific functions:

```bash
pperf top --targets DCT4DBlock perf-report.txt
```

Multiple targets:

```bash
pperf top --targets DCT4D Hierarchical perf-report.txt
```

## Common Workflows

### Identify Optimization Targets

1. Run `top` to see overall hotspots:
   ```bash
   pperf top perf-report.txt
   ```

2. Check Self% to find where time is actually spent:
   ```bash
   pperf top --self perf-report.txt
   ```

3. Drill into a specific component:
   ```bash
   pperf top --targets Hierarchical4DEncoder perf-report.txt
   ```

### Quick Performance Check

```bash
pperf top -n 3 perf-report.txt
```

Shows just the top 3 functions for a quick overview.

## Generating Perf Reports

If you need to create a perf report:

```bash
# Record performance data
perf record -e cycles -F 99 -g ./your-program [args]

# Generate text report
perf report -i perf.data -n > perf-report.txt
```

## Troubleshooting

### "File not found"

Verify the file path is correct:
```bash
ls -la perf-report.txt
```

### "Invalid perf report format"

Ensure the file is text output from `perf report`, not binary `perf.data`:
```bash
file perf-report.txt
# Should show: ASCII text
```

### "No matching functions found"

The `--targets` pattern didn't match any functions. Try a shorter prefix:
```bash
# Instead of:
pperf top --targets "Hierarchical4DEncoder::get_mSubbandLF_significance"

# Try:
pperf top --targets Hierarchical4D
```
