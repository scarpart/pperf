# Quickstart: Colored Output

**Feature**: 002-colored-output

## Basic Usage

After this feature, `pperf top` will automatically:
1. Display **simplified function names** (no return types, arguments, or templates)
2. **Color-code** functions by type when run in a terminal

```bash
# Standard usage - colors enabled automatically in terminal
pperf top perf-report.txt
```

## Output Example

```
Children%   Self%  Function
   90.74    0.00  Hierarchical4DEncoder::get_mSubband       # Blue (user code)
    7.45    7.45  std::inner_product                        # Yellow (library)
   16.30   16.30  0x7d4c47223efe                            # Red (unresolved)
```

## Color Meanings

| Color | Type | Examples |
|-------|------|----------|
| Blue | Your code | `MyClass::myMethod`, `process_data` |
| Yellow | Libraries | `std::vector`, `pthread_create`, kernel symbols |
| Red | Unresolved | `0x7d4c...`, `0000000000` |

## Disable Colors

```bash
# Method 1: Flag
pperf top --no-color perf-report.txt

# Method 2: Environment variable
NO_COLOR=1 pperf top perf-report.txt

# Method 3: Pipe (auto-detected)
pperf top perf-report.txt > output.txt   # No colors in file
```

## Combine with Other Flags

```bash
# Top 5 by self time, filtered, no colors
pperf top --self -n 5 --targets get_mSubband --no-color perf-report.txt
```

## Symbol Simplification

Long C++ names are automatically simplified:

| Before | After |
|--------|-------|
| `void std::inner_product<double*,...>(...)` | `std::inner_product` |
| `Class::method(int, string) const` | `Class::method` |
| `func.cold.123` | `func` |

Unresolved addresses are preserved as-is for debugging.
