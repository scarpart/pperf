# Data Model: Colored Output with Simplified Function Names

**Feature**: 002-colored-output
**Date**: 2026-01-02

## Entities

### SymbolType (New)

Classifies a symbol's origin for color coding.

| Variant | Color | Description |
|---------|-------|-------------|
| `User` | Blue | Functions from the user's binary |
| `Library` | Yellow | Standard library, libc, kernel functions |
| `Unresolved` | Red | Hex addresses, unparseable symbols |

**Derivation Rules** (in priority order):
1. If symbol matches hex pattern → `Unresolved`
2. If symbol starts with `std::` or `__` or is known libc → `Library`
3. If kernel marker `[k]` present → `Library`
4. Otherwise → `User`

---

### SimplifiedSymbol (New)

A function name stripped of noise for display.

| Field | Type | Description |
|-------|------|-------------|
| `original` | `String` | Raw symbol from perf report |
| `simplified` | `String` | Cleaned name: `Namespace::Function` |
| `symbol_type` | `SymbolType` | Classification for coloring |

**Transformation Rules**:
1. Strip leading return type (before first `::` or function name)
2. Strip template parameters (`<...>` with nesting)
3. Strip argument list (`(...)` with nesting)
4. Strip clone suffixes (`.cold`, `.part.N`, `.isra.N`, `.constprop.N`)
5. Collapse lambda syntax to `{lambda}`

---

### ColorConfig (New)

Runtime configuration for color output.

| Field | Type | Description |
|-------|------|-------------|
| `enabled` | `bool` | Whether to output ANSI codes |

**Determination Logic**:
```
enabled = NOT --no-color flag
          AND NOT NO_COLOR env var set
          AND stdout is a TTY
```

---

### PerfEntry (Existing - No Changes)

The existing struct remains unchanged. Color and simplification are applied at output time.

| Field | Type | Description |
|-------|------|-------------|
| `children_pct` | `f64` | Inclusive CPU percentage |
| `self_pct` | `f64` | Exclusive CPU percentage |
| `symbol` | `String` | Raw function symbol |

---

## ANSI Color Codes

Constants for terminal coloring:

| Name | Code | Usage |
|------|------|-------|
| `RESET` | `\x1b[0m` | Clear formatting |
| `BLUE` | `\x1b[34m` | User functions |
| `YELLOW` | `\x1b[33m` | Library functions |
| `RED` | `\x1b[31m` | Unresolved symbols |

---

## Data Flow

```
Input: PerfEntry { symbol: "void Class::method<T>(int, ...)" }
                         │
                         ▼
               ┌─────────────────┐
               │  symbol.rs      │
               │  classify()     │──► SymbolType::User
               │  simplify()     │──► "Class::method"
               └─────────────────┘
                         │
                         ▼
               ┌─────────────────┐
               │  output.rs      │
               │  format_table() │
               │  apply colors   │
               └─────────────────┘
                         │
                         ▼
Output: "\x1b[34mClass::method\x1b[0m" (blue, if TTY)
    or: "Class::method" (plain, if piped)
```

---

## State Transitions

None. This feature is stateless - each symbol is processed independently.

---

## Validation Rules

| Rule | Applies To | Validation |
|------|------------|------------|
| Non-empty | `SimplifiedSymbol.simplified` | Must not be empty string |
| Preserved hex | Unresolved symbols | Must keep original hex format |
| No ANSI in pipe | Piped output | Must contain zero `\x1b` sequences |
| Length limit | Simplified symbol | Truncate at 100 chars with `...` |
