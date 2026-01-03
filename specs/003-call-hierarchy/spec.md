# Feature Specification: Targeted Call Hierarchy Display

**Feature Branch**: `003-call-hierarchy`
**Created**: 2026-01-02
**Status**: Draft
**Input**: User description: "Show only targeted functions in hierarchy - callers show indented callees (only if both are targets), with adjusted percentages subtracting already-accounted callee time"

## Critical: Percentage Calculation Semantics

### Perf Report Percentages Are Relative

When we see in perf output:
```
|--71.80%--rd_optimize_transform
|          |--17.23%--DCT4DBlock
```

This means DCT4DBlock consumes 17.23% **of rd_optimize_transform's time**, not 17.23% of total time.

### Two Display Modes in pperf Output

1. **Indented entries (callees under callers)**: Show percentage **relative to the caller**
   - If A→B→C and targets are A,C: show C's contribution as % of A's time
   - Must multiply through intermediate B to collapse it out
   - Example: If B is 50% of A, and C is 40% of B, then C is 20% of A

2. **Standalone entries (top-level)**: Show **absolute percentage** of total program time
   - These are directly comparable and sum to ≤100%
   - Adjusted by subtracting absolute contributions shown under callers

### Recursion Handling

If a target function appears recursively (directly or indirectly) in the call tree:
- **Stop at the first (topmost) occurrence** of that target callee
- Do NOT traverse deeper into recursive calls of the same function
- This prevents double-counting and infinite traversal

---

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Display Targeted Callee Relationships (Priority: P1)

When a user specifies multiple target functions with `--targets`, the output shows only those functions. If a target function calls another target function (even indirectly through intermediate functions), the callee appears indented under the caller with its **relative contribution to that caller**. Only targets are shown - intermediate functions are collapsed out via multiplication.

**Why this priority**: This is the core feature - focused analysis of specific functions without noise from unrelated callees.

**Independent Test**: Run `pperf top --hierarchy --targets rd_optimize_transform DCT4DBlock perf-report.txt` and verify DCT4DBlock appears indented under rd_optimize_transform showing its contribution as a percentage of rd_optimize_transform's time.

**Acceptance Scenarios**:

1. **Given** targets A and C where A→B→C (A calls B which calls C, B is 50% of A, C is 40% of B), **When** user runs `pperf top --hierarchy --targets A C`, **Then** A appears with its absolute Children%, followed by indented C showing 20% (50% × 40% = C's share of A), and B is NOT shown.

2. **Given** targets `rd_optimize_transform` and `DCT4DBlock` from the perf report, **When** user runs with `--hierarchy`, **Then** DCT4DBlock appears indented under rd_optimize_transform with 17.23% (its relative contribution to rd_optimize_transform, after collapsing any intermediates).

3. **Given** multiple callers targeting the same callee, **When** user runs with `--hierarchy`, **Then** the callee appears indented under each caller separately with each caller's specific relative contribution.

---

### User Story 2 - Calculate Adjusted Standalone Percentages (Priority: P1)

When a target function appears both as a callee under another target and has its own top-level time, the top-level percentage subtracts all **absolute contributions** already accounted for under callers. This prevents double-counting and shows the "remaining" standalone time.

**Why this priority**: Accurate percentage calculation is essential - users need to know the actual remaining time not accounted for by caller relationships.

**Independent Test**: Run with targets where one function calls another, verify the callee's top-level shows (original absolute% - sum of absolute contributions via callers).

**Acceptance Scenarios**:

1. **Given** function A (30% Children% absolute) calls function C where C shows 15% relative to A, and C has 20% total standalone Children%, **When** hierarchy mode runs with targets A and C, **Then** C appears under A showing 15% (relative to A), and C's standalone shows 15.5% (20% - 4.5%, where 4.5% = 30% × 15% is the absolute contribution).

2. **Given** function C is called by target A (contributing 5% absolute) and target B (contributing 3% absolute), **When** hierarchy runs, **Then** C's standalone shows original% - 5% - 3%.

3. **Given** adjusted percentage would be negative (all C's time accounted under callers), **When** calculating, **Then** display 0.00% at standalone (floor at zero).

---

### User Story 3 - Handle Recursive Calls Correctly (Priority: P1)

When traversing the call tree to find target callees, if a target function appears recursively (calls itself directly or indirectly), the system stops at the first occurrence and does not continue traversing into recursive calls.

**Why this priority**: Prevents double-counting and infinite loops in recursive call patterns.

**Independent Test**: Run with a recursive function as target (like `rd_optimize_hexadecatree` which calls itself), verify it appears only once as a callee under its caller.

**Acceptance Scenarios**:

1. **Given** target A calls target B, and B calls itself recursively, **When** hierarchy is computed, **Then** B appears once under A with its contribution at the first call level only.

2. **Given** A→B→B→B (recursive), **When** computing B's contribution to A, **Then** use only the first B's percentage, ignoring deeper recursive calls.

3. **Given** indirect recursion A→B→C→B, **When** B is a target and traversing from A, **Then** stop at the first occurrence of B.

---

### User Story 4 - Maintain Proper Output Ordering (Priority: P2)

The output maintains sorting order (by Children% or Self% depending on flags), with indented callees appearing immediately after their callers. Functions that are only callees (no remaining standalone time) may be omitted from standalone entries.

**Why this priority**: Consistent ordering helps users scan results quickly.

**Independent Test**: Run with `--self` flag and verify indented entries appear after their caller, and remaining standalone entries maintain sort order.

**Acceptance Scenarios**:

1. **Given** sorted by Children%, **When** output displays, **Then** each target appears in sort order, with its targeted callees indented immediately below before the next standalone entry.

2. **Given** a callee's adjusted percentage is 0%, **When** displaying, **Then** that function either shows at 0.00% or is omitted from standalone (only shown as indented callee).

3. **Given** `--hierarchy` without `--targets`, **When** command runs, **Then** an error explains that `--hierarchy` requires `--targets`.

---

### Edge Cases

- What happens when a target function has no other targets as callees? Display the function normally without any indented entries.
- What happens when there's no call relationship between targets? Display each target at standalone without indentation.
- What happens with recursive calls (A→A)? Stop at first occurrence; show one indented A under A with its first-level recursive contribution only.
- What happens when perf report lacks call tree data? Display flat output with a warning that hierarchy data is unavailable.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST parse call tree structure from perf report indentation patterns.
- **FR-002**: System MUST identify caller-callee relationships between target functions, collapsing intermediate non-target functions via percentage multiplication.
- **FR-003**: System MUST calculate callee's relative contribution to caller by multiplying percentages through intermediate (non-target) functions in the chain.
- **FR-004**: System MUST display targeted callees indented (2 spaces) under their caller target with **relative percentages** (% of caller's time).
- **FR-005**: System MUST display standalone entries with **absolute percentages** (% of total program time).
- **FR-006**: System MUST subtract absolute callee contributions (caller.absolute% × callee.relative%) from callee's standalone percentage.
- **FR-007**: System MUST floor adjusted percentages at 0.00% (never display negative).
- **FR-008**: System MUST stop traversal at the first occurrence of a target callee to handle recursion correctly.
- **FR-009**: System MUST accept `--hierarchy` flag that enables targeted call hierarchy display.
- **FR-010**: System MUST require `--targets` when `--hierarchy` is specified, showing an error otherwise.
- **FR-011**: System MUST preserve existing flat output when `--hierarchy` is not specified (backward compatibility).
- **FR-012**: System MUST apply symbol simplification and color coding from feature 002.
- **FR-013**: System MUST handle multiple callers of the same callee, showing the callee under each caller with that caller's specific relative contribution.

### Key Entities

- **CallRelation**: Represents a caller→callee relationship between two target functions with:
  - `relative_pct`: callee's contribution as % of caller's time (for indented display)
  - `absolute_pct`: callee's absolute contribution (caller.absolute × relative, for subtraction)
- **AdjustedEntry**: A target function's display data including original absolute%, sum of absolute contributions as callee, adjusted%, and list of its targeted callees.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can identify call relationships between their target functions within 5 seconds of viewing output.
- **SC-002**: Displayed percentages are mathematically consistent (standalone% = original% - sum of absolute contributions as callee).
- **SC-003**: Output shows only target functions - no intermediate functions appear.
- **SC-004**: Standalone percentages are absolute and between 0% and 100%.
- **SC-005**: Recursive target functions appear only once per caller (no infinite nesting).

## Example Output (Conceptual)

Given targets `rd_optimize_transform` (71.80% absolute) and `DCT4DBlock` (38.00% absolute standalone):

```
Children%   Self%  Function
   71.80    0.00  rd_optimize_transform
   17.23    0.00    DCT4DBlock              <- RELATIVE: 17.23% of rd_optimize_transform's time
   25.63    5.00  DCT4DBlock                <- ABSOLUTE: 38.00% - (71.80% × 17.23%) = 38.00% - 12.37% = 25.63%
```

**Interpretation**:
- Indented 17.23%: "DCT4DBlock takes 17.23% of rd_optimize_transform's time"
- Standalone 25.63%: "DCT4DBlock takes 25.63% of total time NOT via rd_optimize_transform"

## Test Data from perf-report.txt

### Recommended Test Case 1: Simple Caller-Callee
- **Targets**: `rd_optimize_transform`, `DCT4DBlock`
- **Raw data**: DCT4DBlock shows 17.23% relative under rd_optimize_transform (line 31)
- **Expected indented**: 17.23% (relative to rd_optimize_transform)
- **Expected standalone**: DCT4DBlock.original% - (71.80% × 17.23%)

### Recommended Test Case 2: Recursive Function
- **Targets**: `rd_optimize_transform`, `rd_optimize_hexadecatree`
- **Raw data**: rd_optimize_hexadecatree calls itself recursively many times
- **Expected**: rd_optimize_hexadecatree appears once under rd_optimize_transform at first occurrence, recursion not traversed

### Recommended Test Case 3: Multiple Callers
- **Targets**: `rd_optimize_transform`, `get_mSubbandLF_significance`
- **Expected**: get_mSubbandLF_significance appears indented with relative %, standalone shows adjusted absolute %

## Assumptions

- Perf report uses standard call tree format with `|`, `--`, and percentage patterns.
- Percentages in perf call tree are RELATIVE to the parent function.
- The top-level Children% in perf report (first column) IS absolute.
- When collapsing A→B→C to A→C: relative% = B.relative_to_A × C.relative_to_B.
- If no direct/indirect call path exists between two targets, they are independent and only appear at standalone.
- Recursion is common in the codebase (e.g., rd_optimize_hexadecatree) and must be handled correctly.
