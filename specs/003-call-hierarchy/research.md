# Research: Targeted Call Hierarchy Display

**Feature**: 003-call-hierarchy
**Date**: 2026-01-02

## Research Topics

### 1. Perf Report Call Tree Format

**Question**: How is the call tree structured in perf report output?

**Findings**:

The perf report call tree uses a text-based indentation format:

```
    90.74%     0.00%  command  binary  [.] top_level_function
            |
            ---top_level_function
               caller_function
               |
               |--71.80%--callee_function_1
               |          |
               |           --71.78%--deeper_callee
               |                     |
               |                     |--49.34%--even_deeper
               |                     |--9.17%--another_branch
```

**Key observations**:
1. **Top-level entries**: Start with percentage columns, are absolute values
2. **Call tree starts with**: `|` and `---` markers
3. **Indentation**: Pipes (`|`) indicate depth, each level adds ~10-12 spaces
4. **Percentage format**: `|--XX.XX%--function_name` for branches with percentage
5. **Single branch**: `--XX.XX%--function_name` (no leading pipe for single child)
6. **Function lines without %**: Are intermediate call sites (continuation of caller)

**Decision**: Parse call tree lines by:
- Detecting indentation depth by counting leading spaces/pipes
- Extracting percentage from `--XX.XX%--` pattern
- Extracting function name after the percentage marker

**Rationale**: Simple regex-free parsing using string patterns matches the observed format reliably.

---

### 2. Indentation Depth Calculation

**Question**: How to determine the call depth from line indentation?

**Findings**:

Analyzing the perf-report.txt structure:
- Level 0: Top-level entry (starts with percentage)
- Level 1: Lines starting with `---` or containing `---` after minimal indent
- Deeper levels: Each additional `|` block adds depth

The pattern is approximately:
- Count the number of `|` characters before the function content
- Each `|` + spacing block represents one level

**Decision**:
1. Trim leading spaces
2. Count occurrences of `|` to determine depth
3. Lines with `--XX.XX%--` contain percentage data
4. Lines without percentage are function name continuations

**Rationale**: The `|` count correlates directly with call stack depth in perf's output format.

---

### 3. Percentage Extraction Algorithm

**Question**: How to extract relative percentages from call tree lines?

**Findings**:

Percentage appears in two formats:
1. `|--17.23%--function_name` (with leading pipe)
2. `--17.23%--function_name` (without leading pipe, single child)

**Decision**: Use pattern matching:
```
1. Find "--" followed by digits, ".", "%", "--"
2. Extract the numeric value between the "--" markers
3. Function name follows after the closing "--"
```

**Rationale**: Consistent pattern allows simple string parsing without regex crate.

---

### 4. Recursion Detection Strategy

**Question**: How to detect and handle recursive function calls?

**Findings**:

From perf-report.txt, `rd_optimize_hexadecatree` calls itself multiple times:
```
|--12.01%--rd_optimize_hexadecatree
|          |--11.30%--rd_optimize_hexadecatree
|          |          |--10.59%--rd_optimize_hexadecatree
```

**Decision**:
- Track a "seen targets" set during traversal from each caller
- When traversing from caller A looking for target callees:
  - If target B is found, record it and STOP traversing into B's children
  - This captures the first (highest-level) occurrence only
- The contribution is the percentage at first occurrence

**Rationale**:
- Avoids double-counting recursive calls
- Captures the most significant (outer) contribution
- Matches user's expectation: "show me how much of A's time goes to B"

---

### 5. Intermediate Function Collapsing

**Question**: When A→B→C and only A,C are targets, how to compute C's contribution to A?

**Findings**:

Need to multiply percentages through the chain:
- A has Children% = 30% (absolute)
- B takes 50% of A's time (relative to A)
- C takes 40% of B's time (relative to B)
- C's contribution to A = 50% × 40% = 20% of A's time (relative to A)
- C's absolute contribution = 30% × 20% = 6% of total

**Decision**:
1. When traversing from target A, track cumulative relative percentage
2. Start with 1.0 (100% of A's time)
3. At each intermediate step, multiply by that step's relative percentage
4. When target C is found, record: relative_to_A = cumulative, absolute = A.children × cumulative

**Rationale**: Multiplication through chain correctly collapses intermediate functions.

---

### 6. Data Structure Design

**Question**: What data structures efficiently represent call relationships?

**Decision**:

```rust
/// Raw parsed call tree node
struct CallTreeNode {
    symbol: String,
    relative_pct: f64,  // Percentage relative to parent (0.0-100.0)
    depth: usize,       // Indentation depth
    children: Vec<CallTreeNode>,
}

/// Relationship between two target functions
struct CallRelation {
    caller: String,
    callee: String,
    relative_pct: f64,  // C's contribution as % of caller's time
    absolute_pct: f64,  // Caller.absolute × relative_pct / 100
}

/// Target function with adjusted percentages
struct HierarchyEntry {
    symbol: String,
    original_children_pct: f64,
    original_self_pct: f64,
    adjusted_children_pct: f64,
    callees: Vec<CallRelation>,  // Targeted callees under this function
}
```

**Rationale**: Separates parsing (CallTreeNode) from relationship computation (CallRelation) and display (HierarchyEntry).

---

### 7. Algorithm Overview

**Question**: What is the overall algorithm flow?

**Decision**:

```
1. PARSE: Read perf report, build CallTreeNode forest (one tree per top-level entry)
2. IDENTIFY: For each top-level entry, traverse tree to find target functions at any depth
3. RELATE: For each target A, traverse its subtree to find other targets B,C...
   - Track visited targets to handle recursion
   - Compute relative contribution by multiplying through intermediates
4. ADJUST: For each target that appears as callee:
   - Sum all absolute contributions (caller.absolute × relative)
   - adjusted_pct = original_pct - sum_of_contributions
   - Floor at 0.0
5. OUTPUT: Format with indentation for callees, standalone for adjusted entries
```

**Rationale**: Linear passes through data; O(n × m) where n=entries, m=targets.

---

## Alternatives Considered

| Decision | Chosen | Alternative | Why Rejected |
|----------|--------|-------------|--------------|
| Parsing method | String patterns | Regex crate | No external deps per constitution |
| Recursion handling | First occurrence only | Sum all recursive calls | Would over-count; spec requires stopping at first |
| Percentage display | Relative for indented, absolute for standalone | All absolute | Less intuitive; relative shows "% of caller's time" |
| Data structures | Three structs | Single unified struct | Separation of concerns; cleaner |

---

## Test Data Identified

From `perf-report.txt`:

| Test Case | Targets | Expected Relationship |
|-----------|---------|----------------------|
| Simple caller-callee | `rd_optimize_transform`, `DCT4DBlock` | DCT4DBlock 17.23% under rd_optimize_transform |
| Through intermediate | `rd_optimize_transform`, `inner_product` | inner_product via do_4d_transform chain |
| Recursive | `rd_optimize_transform`, `rd_optimize_hexadecatree` | hexadecatree 12.01% (first occurrence only) |
| Multiple callers | `evaluate_split`, `DCT4DBlock` | DCT4DBlock under different call paths |
