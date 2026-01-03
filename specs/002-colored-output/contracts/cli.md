# CLI Contract: Colored Output

**Feature**: 002-colored-output
**Date**: 2026-01-02

## Command Updates

### `pperf top` (Updated)

```
pperf top [OPTIONS] <FILE>
```

#### New Option

| Option | Short | Type | Default | Description |
|--------|-------|------|---------|-------------|
| `--no-color` | | flag | false | Disable colored output |

#### Environment Variables

| Variable | Effect |
|----------|--------|
| `NO_COLOR` | If set (any value), disables colors |

#### Color Behavior

| Condition | Colors |
|-----------|--------|
| TTY + no flags + no NO_COLOR | Enabled |
| Piped to file | Disabled |
| `--no-color` flag | Disabled |
| `NO_COLOR` env set | Disabled |

---

## Output Format

### With Colors (TTY)

```
Children%   Self%  Function
   90.74    0.00  [BLUE]Hierarchical4DEncoder::get_mSubband[RESET]
    7.45    7.45  [YELLOW]std::inner_product[RESET]
   16.30   16.30  [RED]0x7d4c47223efe[RESET]
```

Where:
- `[BLUE]` = `\x1b[34m`
- `[YELLOW]` = `\x1b[33m`
- `[RED]` = `\x1b[31m`
- `[RESET]` = `\x1b[0m`

### Without Colors (Piped/--no-color)

```
Children%   Self%  Function
   90.74    0.00  Hierarchical4DEncoder::get_mSubband
    7.45    7.45  std::inner_product
   16.30   16.30  0x7d4c47223efe
```

---

## Symbol Simplification Examples

| Input (Raw) | Output (Simplified) |
|-------------|---------------------|
| `void Hierarchical4DEncoder::get_mSubbandLF_significance(unsigned int, LightfieldCoordinate<unsigned int> const&, LightfieldDimension<unsigned int, true> const&) const` | `Hierarchical4DEncoder::get_mSubbandLF_significance` |
| `double std::inner_product<double*, double const*, double>(double*, double*, double const*, double)` | `std::inner_product` |
| `void TransformPartition::evaluate_split_for_partitions<(PartitionFlag)1, (PartitionFlag)2>(Block4D const&, ...)` | `TransformPartition::evaluate_split_for_partitions` |
| `DCT4DBlock::DCT4DBlock(Block4D const&, double)` | `DCT4DBlock::DCT4DBlock` |
| `main::{lambda(int)#1}::operator()(int) const` | `main::{lambda}` |
| `func.cold.123` | `func` |
| `0x7d4c47223efe` | `0x7d4c47223efe` |
| `0000000000000000` | `0000000000000000` |

---

## Color Classification Rules

### Priority Order

1. **Unresolved (Red)**
   - Starts with `0x`
   - All characters are hex digits (0-9, a-f, A-F)

2. **Library (Yellow)**
   - Starts with `std::`
   - Starts with `__` (double underscore - libc internals)
   - Matches: `pthread_*`, `malloc`, `free`, `memset`, `memcpy`, `memmove`
   - Contains `@GLIBC` or `@GCC`

3. **User (Blue)**
   - All other symbols

---

## Exit Codes

No changes to existing exit codes:

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | File not found |
| 2 | Invalid format |
| 3 | Invalid arguments |
| 4 | No matches (with --targets) |

---

## Help Text Update

```
OPTIONS:
    --self, -s           Sort by Self% instead of Children%
    -n, --number <N>     Number of functions to display (default: 10)
    --targets, -t <N>... Filter by function name substrings
    --no-color           Disable colored output
    --help, -h           Show this help message
    --version            Show version information
```
