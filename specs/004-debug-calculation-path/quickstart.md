# Quickstart: Debug Calculation Path

**Feature**: 004-debug-calculation-path
**Date**: 2026-01-03

## Prerequisites

- Feature 003 (call-hierarchy) must be merged to the working branch
- Rust toolchain installed (`cargo`, `rustc`)
- Access to `perf-report.txt` test data

## Usage

### Basic Debug Output

```bash
# Show hierarchy with calculation path annotations
pperf top --hierarchy --debug -t rd_optimize DCT4DBlock inner_product perf-report.txt

# Short form
pperf top -H -D -t rd_optimize DCT4DBlock inner_product perf-report.txt
```

### Expected Output

```
Children%   Self%  Function
   71.80    0.00  TransformPartition::rd_optimize_transform
   17.23    0.00      DCT4DBlock::DCT4DBlock
                      (direct: 17.23%)
    0.07    0.00          std::inner_product
                          (via do_4d_transform 0.42% × 17.23% = 0.07%)
   25.92    0.00  DCT4DBlock::DCT4DBlock
    2.21    0.00      std::inner_product
                      (direct: 2.21%)
    6.89    7.45  std::inner_product
```

### Without Color

```bash
# Disable color (annotations appear as plain text)
pperf top --hierarchy --debug --no-color -t rd_optimize DCT4DBlock perf-report.txt
```

### Debug Flag Without Hierarchy

```bash
# No effect - output is normal (no annotations)
pperf top --debug -t rd_optimize perf-report.txt
```

## Development

### Build

```bash
cargo build --release
```

### Test

```bash
# Run all tests
cargo test

# Run specific hierarchy debug tests
cargo test debug_annotation
```

### Lint

```bash
cargo fmt
cargo clippy
```

## Key Files

| File | Purpose |
|------|---------|
| `src/main.rs` | Parses `--debug`/`-D` flag |
| `src/hierarchy.rs` | `CallRelation` with `intermediary_path` |
| `src/output.rs` | `format_debug_annotation()` function |
| `src/symbol.rs` | `DIM` color constant |

## Troubleshooting

### Annotations not appearing

1. Verify `--hierarchy` flag is present (debug requires hierarchy mode)
2. Check that targets have actual intermediaries (direct calls show `(direct: X%)`)

### Colors not working

1. Check terminal supports ANSI codes
2. Verify `--no-color` is not set
3. Check `NO_COLOR` environment variable is not set
