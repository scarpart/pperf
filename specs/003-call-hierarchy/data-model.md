# Data Model: Targeted Call Hierarchy Display

**Feature**: 003-call-hierarchy
**Date**: 2026-01-02

## Entities

### 1. CallTreeLine (Parsing)

Represents a single line from the perf report call tree section.

| Field | Type | Description |
|-------|------|-------------|
| `depth` | `usize` | Call stack depth (0 = top-level entry, 1+ = call tree) |
| `relative_pct` | `Option<f64>` | Percentage if present (from `--XX.XX%--` pattern) |
| `symbol` | `String` | Function name (simplified via symbol module) |
| `is_top_level` | `bool` | True if this is a top-level perf entry with absolute % |

**Parsing Rules**:
- Top-level: Starts with digits (percentage), `is_top_level = true`
- Call tree: Contains `|` or starts with `---`, `is_top_level = false`
- Depth: Count `|` characters for call tree lines
- Percentage: Extract from `--XX.XX%--` pattern

---

### 2. CallTreeNode (Tree Structure)

Hierarchical representation of a function and its callees.

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | `String` | Simplified function name |
| `relative_pct` | `f64` | Percentage relative to parent (0.0-100.0) |
| `children` | `Vec<CallTreeNode>` | Direct callees in the call tree |

**Construction**:
- Built from sequential CallTreeLine entries using depth to determine parent-child
- Each top-level entry starts a new tree
- Call tree lines are nested based on depth changes

**Invariants**:
- `relative_pct` is always relative to the immediate parent
- Root nodes (top-level entries) have `relative_pct = 100.0` (they represent all their samples)

---

### 3. CallRelation (Target Relationship)

Represents a caller→callee relationship between two target functions.

| Field | Type | Description |
|-------|------|-------------|
| `caller` | `String` | Caller target function (simplified name) |
| `callee` | `String` | Callee target function (simplified name) |
| `relative_pct` | `f64` | Callee's contribution as % of caller's time |
| `absolute_pct` | `f64` | Absolute contribution: `caller.children_pct × relative_pct / 100` |

**Computation**:
- When traversing from caller A to find target callee C through intermediates:
  - `relative_pct` = product of all intermediate percentages
  - `absolute_pct` = A's original Children% × `relative_pct` / 100

**Example**:
- A (71.80% absolute) → B (49.34% of A) → C (17.23% of B)
- C's relative to A = 49.34% × 17.23% / 100 = 8.50%
- C's absolute = 71.80% × 8.50% / 100 = 6.10%

---

### 4. HierarchyEntry (Display)

Target function with computed hierarchy data for output.

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | `String` | Simplified function name |
| `original_children_pct` | `f64` | Original Children% from perf report |
| `original_self_pct` | `f64` | Original Self% from perf report |
| `adjusted_children_pct` | `f64` | After subtracting callee contributions |
| `callees` | `Vec<CallRelation>` | Targeted callees under this function |
| `is_caller` | `bool` | True if this function has targeted callees |

**Adjustment Calculation**:
```
sum_absolute_contributions = Σ (caller.original_children_pct × callee.relative_pct / 100)
                            for all CallRelations where this function is the callee
adjusted_children_pct = max(0.0, original_children_pct - sum_absolute_contributions)
```

---

## Relationships

```
┌─────────────────────────────────────────────────────────────────┐
│                         perf-report.txt                          │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │ Top Entry 1  │    │ Top Entry 2  │    │ Top Entry N  │       │
│  │ (90.74%)     │    │ (78.97%)     │    │ (...)        │       │
│  └──────┬───────┘    └──────┬───────┘    └──────┬───────┘       │
│         │ has               │ has               │ has            │
│         ▼                   ▼                   ▼                │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │ CallTreeNode │    │ CallTreeNode │    │ CallTreeNode │       │
│  │ (tree root)  │    │ (tree root)  │    │ (tree root)  │       │
│  └──────┬───────┘    └──────────────┘    └──────────────┘       │
│         │ contains                                               │
│         ▼                                                        │
│  ┌──────────────┐                                                │
│  │   children   │ ──────► nested CallTreeNode...                 │
│  └──────────────┘                                                │
└─────────────────────────────────────────────────────────────────┘

                        ┌─────────────┐
                        │   targets   │ (user-specified)
                        │ [A, B, C]   │
                        └──────┬──────┘
                               │
              ┌────────────────┼────────────────┐
              │                │                │
              ▼                ▼                ▼
     ┌────────────────┐ ┌────────────────┐ ┌────────────────┐
     │ HierarchyEntry │ │ HierarchyEntry │ │ HierarchyEntry │
     │ symbol: A      │ │ symbol: B      │ │ symbol: C      │
     │ callees: [B,C] │ │ callees: [C]   │ │ callees: []    │
     └───────┬────────┘ └───────┬────────┘ └────────────────┘
             │                  │
             ▼                  ▼
      ┌─────────────┐    ┌─────────────┐
      │CallRelation │    │CallRelation │
      │ caller: A   │    │ caller: B   │
      │ callee: B   │    │ callee: C   │
      │ relative: X%│    │ relative: Y%│
      │ absolute: Z%│    │ absolute: W%│
      └─────────────┘    └─────────────┘
```

---

## State Transitions

### Parsing State Machine

```
START ──► READ_LINE
              │
              ├── is_comment ──────────► SKIP ──► READ_LINE
              │
              ├── is_top_level ────────► EMIT TopLevelEntry ──► READ_LINE
              │
              ├── is_call_tree_line ───► BUILD CallTreeNode
              │                                    │
              │                                    ├── deeper ──► PUSH to parent.children
              │                                    ├── same ────► ADD sibling
              │                                    └── shallower ► POP stack, ADD sibling
              │
              └── is_empty/separator ──► FINALIZE current tree ──► READ_LINE
```

### Hierarchy Computation State

```
START: List of targets, parsed call trees

FOR each target A in sorted order:
  1. FIND all CallTreeNodes matching A (by simplified name)
  2. FOR each occurrence:
     a. TRAVERSE subtree looking for other targets
     b. TRACK seen_targets to detect recursion
     c. MULTIPLY percentages through intermediates
     d. RECORD CallRelation for each found target

FOR each target B that appears as callee:
  1. SUM absolute contributions from all CallRelations
  2. COMPUTE adjusted_children_pct

OUTPUT in sorted order with indentation
```

---

## Validation Rules

| Rule | Entity | Description |
|------|--------|-------------|
| V1 | CallTreeLine | `relative_pct` must be 0.0-100.0 or None |
| V2 | CallTreeNode | `relative_pct` must be 0.0-100.0 |
| V3 | CallRelation | `relative_pct` must be 0.0-100.0 |
| V4 | CallRelation | `absolute_pct` = caller.children × relative / 100 |
| V5 | HierarchyEntry | `adjusted_children_pct` >= 0.0 (floored) |
| V6 | HierarchyEntry | `adjusted_children_pct` <= `original_children_pct` |
| V7 | Recursion | A target appears at most once as callee per caller |
