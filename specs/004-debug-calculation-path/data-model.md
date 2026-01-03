# Data Model: Debug Calculation Path

**Feature**: 004-debug-calculation-path
**Date**: 2026-01-03

## Entity Changes

### CallRelation (Extended)

**Location**: `src/hierarchy.rs`

**Current fields** (unchanged):
- `caller: String` - Caller target function (simplified name)
- `callee: String` - Callee target function (simplified name)
- `relative_pct: f64` - Callee's contribution as % of caller's time
- `absolute_pct: f64` - Absolute contribution
- `context_root: Option<String>` - Root caller context for path-specific relations

**New field**:
- `intermediary_path: Vec<IntermediaryStep>` - Ordered list of non-target functions traversed between caller and callee

### IntermediaryStep (New)

**Location**: `src/hierarchy.rs`

A lightweight struct representing one step in the calculation path.

**Fields**:
- `symbol: String` - Simplified function name
- `percentage: f64` - Relative percentage at this step

**Derived property** (computed, not stored):
- `is_direct() -> bool` - Returns true if `intermediary_path` is empty

### CallerContribution (New)

**Location**: `src/hierarchy.rs`

Represents one caller's contribution to a standalone entry's adjusted percentage.

**Fields**:
- `caller: String` - Simplified name of the calling target function
- `absolute_pct: f64` - The contribution amount (absolute %) subtracted from original

### HierarchyEntry (Extended)

**Location**: `src/hierarchy.rs`

**Current fields** (unchanged):
- `symbol: String` - Simplified function name
- `original_children_pct: f64` - Original Children% from perf report
- `original_self_pct: f64` - Original Self% from perf report
- `adjusted_children_pct: f64` - After subtracting callee contributions
- `callees: Vec<CallRelation>` - Targeted callees under this function
- `is_caller: bool` - True if this function has targeted callees

**New field**:
- `contributions: Vec<CallerContribution>` - Breakdown of contributions FROM callers that were subtracted

## Relationships

```
CallRelation
    │
    ├── caller (String) ─────────────────┐
    │                                     │
    ├── callee (String) ─────────────────┤
    │                                     ├── Both are target functions
    ├── intermediary_path ────────────────┤   (match user's --targets)
    │       │                             │
    │       └── Vec<IntermediaryStep>     │
    │               │                     │
    │               └── symbol (String) ──┘── Non-target functions
    │                   percentage (f64)      in the call path
    │
    └── Used by: format_hierarchy_table()
                 format_debug_annotation()
```

## State Transitions

N/A - This feature adds display-only data. No state machines involved.

## Validation Rules

1. **Intermediary path ordering**: Steps must be in call order (caller → first intermediary → ... → callee)
2. **Percentage consistency**: Product of all percentages in path (divided by 100 at each step) should approximate the stored `relative_pct`
3. **Name simplification**: All symbols in `intermediary_path` must be simplified (same as `callee`)

## Data Flow

```
                     perf-report.txt
                           │
                           ▼
              ┌─────────────────────────┐
              │   parse_file_call_trees │
              └───────────┬─────────────┘
                          │
                          ▼
              ┌─────────────────────────┐
              │  compute_call_relations │
              │                         │
              │  find_target_callees()  │──── Now tracks intermediary_path
              │                         │     during DFS traversal
              └───────────┬─────────────┘
                          │
                          ▼
              ┌─────────────────────────┐
              │   Vec<CallRelation>     │──── Each relation now includes
              │   (with paths)          │     its calculation path
              └───────────┬─────────────┘
                          │
                          ▼
              ┌─────────────────────────┐
              │  format_hierarchy_table │
              │                         │
              │  if debug_flag:         │
              │    format_debug_annot() │──── Renders path as gray text
              └───────────┬─────────────┘
                          │
                          ▼
                    Terminal output
                    with annotations
```

## Example Data

**Scenario**: `rd_optimize_transform` calls `do_4d_transform` (42%) which calls `inner_product` (17.23%)

**CallRelation instance**:
```
CallRelation {
    caller: "rd_optimize_transform",
    callee: "std::inner_product",
    relative_pct: 7.23,  // 42.0 * 17.23 / 100
    absolute_pct: 5.19,  // 71.80 * 7.23 / 100
    context_root: None,
    intermediary_path: [
        IntermediaryStep { symbol: "do_4d_transform", percentage: 42.0 }
    ]
}
```

**Rendered debug annotation**:
```
(via do_4d_transform 42.00% × 17.23% = 7.23%)
```

**Direct call example** (no intermediaries):
```
CallRelation {
    caller: "rd_optimize_transform",
    callee: "DCT4DBlock::DCT4DBlock",
    relative_pct: 17.23,
    absolute_pct: 12.37,
    context_root: None,
    intermediary_path: []  // Empty = direct call
}
```

**Rendered debug annotation**:
```
(direct: 17.23%)
```

**Standalone entry example** (called by one target):
```
HierarchyEntry {
    symbol: "DCT4DBlock::DCT4DBlock",
    original_children_pct: 38.00,
    adjusted_children_pct: 25.63,
    contributions: [
        CallerContribution { caller: "rd_optimize_transform", absolute_pct: 12.37 }
    ]
}
```

**Rendered debug annotation**:
```
(standalone: 38.00% - 12.37% (rd_optimize_transform) = 25.63%)
```

**Standalone entry with multiple callers**:
```
HierarchyEntry {
    symbol: "std::inner_product",
    original_children_pct: 7.47,
    adjusted_children_pct: 0.58,
    contributions: [
        CallerContribution { caller: "rd_optimize_transform", absolute_pct: 5.19 },
        CallerContribution { caller: "DCT4DBlock", absolute_pct: 1.70 }
    ]
}
```

**Rendered debug annotation**:
```
(standalone: 7.47% - 5.19% (rd_optimize_transform) - 1.70% (DCT4DBlock) = 0.58%)
```
