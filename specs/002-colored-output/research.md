# Research: Colored Output with Simplified Function Names

**Feature**: 002-colored-output
**Date**: 2026-01-02

## 1. ANSI Terminal Colors (No External Dependencies)

### Decision
Use raw ANSI escape codes directly via Rust string literals.

### Rationale
- Constitution mandates no external dependencies
- ANSI codes are simple escape sequences: `\x1b[XXm` format
- All target platforms (Linux, macOS, modern Windows) support ANSI
- Total code overhead: ~20 lines for color constants

### Implementation
```rust
pub const RESET: &str = "\x1b[0m";
pub const BLUE: &str = "\x1b[34m";      // User functions
pub const YELLOW: &str = "\x1b[33m";    // Library/system
pub const RED: &str = "\x1b[31m";       // Unresolved
```

### Alternatives Considered
| Alternative | Rejected Because |
|-------------|------------------|
| `colored` crate | External dependency violates constitution |
| `termcolor` crate | External dependency violates constitution |
| 256-color codes | Unnecessary complexity; 16-color ANSI sufficient |

---

## 2. TTY Detection for Auto Color Disable

### Decision
Use `std::io::IsTerminal` trait (stabilized in Rust 1.70).

### Rationale
- Part of std library (no external dependency)
- Cross-platform (Unix and Windows)
- Simple API: `stdout().is_terminal()`

### Implementation
```rust
use std::io::{stdout, IsTerminal};

fn should_use_color(no_color_flag: bool) -> bool {
    if no_color_flag {
        return false;
    }
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }
    stdout().is_terminal()
}
```

### Alternatives Considered
| Alternative | Rejected Because |
|-------------|------------------|
| `atty` crate | External dependency |
| Always output color | Breaks piped output readability |
| Manual fd check | Platform-specific, IsTerminal handles this |

---

## 3. Symbol Classification Algorithm

### Decision
Classify symbols using a priority-based rule set matching perf report markers.

### Rationale
- Perf report includes `[.]` (user space) and `[k]` (kernel) markers
- Shared object names provide library identification
- Hex-only patterns identify unresolved symbols

### Classification Rules (in order)

1. **Unresolved (Red)**:
   - Pattern: starts with `0x` or is all hex digits
   - Example: `0x7d4c47223efe`, `0000000000000000`

2. **Library/System (Yellow)**:
   - Symbol starts with `std::`
   - Symbol starts with `__` (libc internals)
   - Kernel marker `[k]` present in original line
   - Known library prefixes: `pthread_`, `malloc`, `free`, `memset`, `memcpy`

3. **User (Blue)**:
   - Default for all other symbols with `[.]` marker
   - Matches command name in perf report

### Alternatives Considered
| Alternative | Rejected Because |
|-------------|------------------|
| Parse shared object column | Requires changing PerfEntry structure significantly |
| Heuristic only (no markers) | Less accurate than using perf's own markers |

---

## 4. Symbol Simplification Algorithm

### Decision
Strip components using bracket/parenthesis matching with left-to-right parsing.

### Rationale
- C++ symbols have predictable structure: `return_type namespace::function<templates>(args)`
- Nested templates require bracket counting, not simple regex
- Lambda detection needs special handling

### Simplification Steps

1. **Strip return type**: Find first `(` or `<`, work backward to find type end
2. **Strip template parameters**: Remove `<...>` with nested bracket counting
3. **Strip arguments**: Remove `(...)` with nested parenthesis counting
4. **Strip clone suffixes**: Remove `.cold`, `.part.N`, `.isra.N`, `.constprop.N`
5. **Handle lambdas**: Pattern `{lambda(...)}` → `{lambda}`

### Examples

| Original | Simplified |
|----------|------------|
| `void Hierarchical4DEncoder::get_mSubbandLF_significance(unsigned int, ...)` | `Hierarchical4DEncoder::get_mSubbandLF_significance` |
| `double std::inner_product<double*, double const*, double>(double*, ...)` | `std::inner_product` |
| `main::{lambda(int)#1}::operator()(int) const` | `main::{lambda}` |
| `func.cold.123` | `func` |
| `0x7d4c47223efe` | `0x7d4c47223efe` (unchanged) |

### Alternatives Considered
| Alternative | Rejected Because |
|-------------|------------------|
| Regex-based parsing | Fails on nested templates like `vector<pair<int, int>>` |
| Demangle + re-parse | Symbols already demangled in perf report |
| Only strip arguments | Templates still clutter output significantly |

---

## 5. Parser Integration

### Decision
Extend `PerfEntry` with optional metadata, add new `symbol.rs` module.

### Rationale
- Keep parser focused on extraction
- New module handles classification and simplification
- Separation of concerns aligns with simplicity principle

### Data Flow
```
parser.rs (extract raw symbol)
    → symbol.rs (classify + simplify)
        → output.rs (apply color + format)
```

### Alternatives Considered
| Alternative | Rejected Because |
|-------------|------------------|
| Inline in output.rs | Mixes formatting with business logic |
| Modify PerfEntry struct | Over-complicates simple data structure |

---

## 6. NO_COLOR Standard Compliance

### Decision
Check `NO_COLOR` environment variable presence (any value).

### Rationale
- Industry standard: https://no-color.org/
- Simple check: presence of variable, not its value
- Used by many CLI tools (cargo, git, etc.)

### Implementation
```rust
if std::env::var("NO_COLOR").is_ok() {
    // Disable colors regardless of value
}
```

---

## Summary of Technical Decisions

| Area | Decision | Key Benefit |
|------|----------|-------------|
| Colors | Raw ANSI codes | No dependencies |
| TTY Detection | `std::io::IsTerminal` | Std library, cross-platform |
| Classification | Rule-based with perf markers | Accurate, matches perf semantics |
| Simplification | Bracket-counting parser | Handles nested templates correctly |
| Module Structure | New `symbol.rs` | Clean separation of concerns |
| NO_COLOR | Check variable presence | Industry standard compliance |
