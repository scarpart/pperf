# JPLM Encoder Performance Analysis

## Goal

Obtain a clear view of the performance characteristics of the JPLM encoder/decoder in Release mode to identify optimization targets.

## Profiling Methodology

### Data Collection
```bash
perf record -e cycles -F 99 -g ./jpl-encoder-bin [args]
perf report -i perf.data -n > perf-report.txt
```

### Flame Graph Generation
```bash
perf script | stackcollapse-perf.pl | flamegraph.pl > flame.svg
```

## Understanding `perf report` Output Format

### Column Definitions

| Column | Meaning |
|--------|---------|
| **Children** | Inclusive time: this function + all callees (time spent in call subtree) |
| **Self** | Exclusive time: cycles spent only within this function's own code |
| **Command** | The binary/process name |
| **Shared Object** | Library or executable containing the symbol |
| **Symbol** | Demangled function name |

### Why Functions Appear Multiple Times

The default `perf report` output sorts by **Children** (inclusive time) and shows each function as a separate entry with its call tree. Functions on the same call chain will show similar percentages:

```
parallel_for_with_progress (90.74% children, 0.00% self)
  └── run_for_block_4d (90.74% children, 0.00% self)
        └── rd_optimize_transform (71.80% children, 0.00% self)
```

This is **not duplication** - it's the same samples viewed from different call stack perspectives. Functions with high Children% but 0% Self are "pass-through" callers that don't consume CPU themselves.

### Multi-threaded Execution

Samples from all threads (e.g., 8 OpenMP threads) are **aggregated** into a single view. Per-thread breakdown available via `perf report --tid`.

## JPLM Encoder Hotspots (Release -O3)

### Top Functions by Self Time (Actual CPU Consumption)

| Self % | Function | Notes |
|--------|----------|-------|
| 11.94% | `Hierarchical4DEncoder::get_mSubbandLF_significance` | Primary optimization target |
| 7.45% | `std::inner_product<double*, double const*, double>` | DCT inner loop - vectorization candidate |
| 6.83% | `Block4D::get_linear_position` | 4D index linearization overhead |
| 4.65% | `Block4D::get_linear_position` | (additional call sites) |
| 3.70% | `__memset_avx2_unaligned_erms` | Memory initialization |
| 2.18% | `Lightfield::is_coordinate_valid` | Bounds checking |
| 1.79% | `std::__uniq_ptr_impl::_M_ptr()` | Smart pointer overhead |

### Call Hierarchy (by Inclusive Time)

```
90.74%  parallel_for_with_progress (OpenMP entry)
└── 90.74%  JPLM4DTransformModeLightFieldEncoder::run_for_block_4d
    ├── 71.80%  TransformPartition::rd_optimize_transform
    │   ├── 49.34%  evaluate_split_for_partitions<1,2> (recursive partitioning)
    │   │   └── 49.33%  rd_optimize_transform (recursive)
    │   │       ├── 30.47%  evaluate_split_for_partitions (deeper recursion)
    │   │       ├── 17.23%  DCT4DBlock::DCT4DBlock (DCT transform)
    │   │       └── 12.01%  rd_optimize_hexadecatree
    │   ├── 11.89%  DCT4DBlock::DCT4DBlock
    │   └── 9.17%   rd_optimize_hexadecatree
    └── 7.26%   LightFieldTransformMode::get_block_4D_from
```

### Unresolved Symbols

Addresses like `0x7d4c47223efe` (~12% combined) are unresolved due to:
- Aggressive inlining at -O3
- Missing debug symbols in system libraries
- Potential vDSO/kernel code

## Optimization Recommendations

### High-Priority Targets

1. **`get_mSubbandLF_significance` (11.94%)** - Most time-consuming single function
2. **`std::inner_product` (7.45%)** - DCT computation core; candidate for SIMD/AVX optimization
3. **`Block4D::get_linear_position` (6.83%)** - Consider caching or restructuring 4D indexing

### Compiler Flags for Better Profiling

For more accurate symbol resolution while maintaining optimization:
```bash
-O2 -g3 -fno-omit-frame-pointer -fno-inline -gdwarf-4
```

Or use DWARF-based stack unwinding:
```bash
perf record -e cycles -F 99 --call-graph dwarf ./jpl-encoder-bin
```

### Alternative Analysis Commands

```bash
# Flat view (no call tree nesting)
perf report -i perf.data --no-children -n

# Per-thread breakdown
perf report -i perf.data --tid

# Annotate specific function (see assembly hotspots)
perf annotate -i perf.data -s "Hierarchical4DEncoder::get_mSubbandLF_significance"
```

## Summary

The JPLM encoder spends ~90% of its time in the parallel block processing loop. Within that:
- **~38%** in DCT transforms (`DCT4DBlock` constructor, `do_4d_transform`, `inner_product`)
- **~37%** in hierarchical RD optimization (`rd_optimize_hexadecatree`, `get_mSubbandLF_significance`)
- **~7%** in data copying and block extraction

The recursive `rd_optimize_transform` / `evaluate_split_for_partitions` pattern dominates the profile, suggesting the RD optimization algorithm itself is the primary compute bottleneck.

## Active Technologies
- Rust (latest stable, 2024 edition) + None (standard library only per constitution) (001-list-top-functions)
- N/A (file-based input, stdout output) (001-list-top-functions)

## Recent Changes
- 001-list-top-functions: Added Rust (latest stable, 2024 edition) + None (standard library only per constitution)
