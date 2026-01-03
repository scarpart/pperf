# Feature Specification: Targeted Call Hierarchy Display

**Feature Branch**: `003-call-hierarchy`
**Created**: 2026-01-02
**Updated**: 2026-01-03
**Status**: Active
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

1. **Indented entries (callees under callers)**: Show percentage **relative to the immediate caller**
   - If A→B→C and all are targets: show B's % relative to A, then C's % relative to B
   - If A→B→C and only A,C are targets: show C's collapsed % relative to A (B.rel × C.rel)
   - Example: If B is 50% of A, and C is 40% of B, then C is 20% of A (when B is not a target)

2. **Standalone entries (top-level)**: Show **absolute percentage** of total program time
   - These are directly comparable and sum to ≤100%
   - Adjusted by subtracting absolute contributions from ROOT callers only
   - A "root caller" is a caller that is NOT itself shown as a callee under another target

### Recursion Handling

If a target function appears recursively (directly or indirectly) in the call tree:
- **Stop at the first (topmost) occurrence** of that target callee
- Do NOT traverse deeper into recursive calls of the same function
- This prevents double-counting and infinite traversal

---

## Multi-Level Nesting

### Core Concept

When multiple targets form a call chain (A → B → C where all are targets), the display shows the full nested hierarchy with increasing indentation levels:

```
Children%   Self%  Function
   71.80    0.00  A                      <- root caller (level 0)
   50.00    0.00      B                  <- callee of A (level 1, 4-space indent)
   40.00    0.00          C              <- callee of B (level 2, 8-space indent)
   10.00    0.00  C                      <- standalone (adjusted: original - contributions)
```

### Indentation Rules

- **Level 0** (root callers): No indentation
- **Level 1** (direct callees): 4 spaces
- **Level 2** (callees of callees): 8 spaces
- **Level N**: N × 4 spaces

### Consumption and Deduplication

When a caller→callee relationship is displayed nested under a parent, it is "consumed" and should NOT be repeated:

1. **A shows B, B shows C**: When B appears under A with C nested under B, then:
   - B's standalone entry does NOT show C as its callee (already shown under A→B)
   - C's standalone entry only counts contributions from its ROOT caller (A, not B)

2. **Example with consumption**:
   ```
   A (80%)
       B (50%)           <- B is 50% of A
           C (40%)       <- C is 40% of B (shown here, consumed)
   B (40%)               <- B standalone: 80% - (80% × 50%) = 40%
                         <- NO C shown here (A→B→C already consumed that path)
   C (5%)                <- C standalone: original - contribution from A only
   ```

3. **Independent branches**: If A→B→C AND separately D→C (different call path):
   ```
   A (80%)
       B (50%)
           C (40%)       <- C via A→B path
   D (30%)
       C (20%)           <- C via D path (different branch, NOT consumed)
   C (5%)                <- C standalone: original - A's contribution - D's contribution
   ```

### Contribution Calculation for Adjusted Percentages

For a target's standalone adjusted percentage:
- Only subtract contributions from **root callers** (callers at level 0)
- Do NOT subtract contributions from intermediate callers (they're already part of the root's contribution)

**Formula**: `adjusted% = original% - Σ(root_caller.children% × path_product_to_target)`

**Example**: A (80%) → B (50%) → C (40%)
- C's contribution from A = 80% × 50% × 40% = 16% (absolute)
- C's adjusted = C.original% - 16%
- Do NOT also subtract "B's contribution" (that's double-counting)

### Leaf Function Caller Chains (Bug Fix)

Perf report shows **caller chains** for leaf functions (high Self%, low Children%):
```
7.47%  7.45%  inner_product      <- leaf function
       |
       |--6.45%--caller_address
                 parallel_for     <- this is a CALLER, not a callee
                 run_for_block    <- this is a CALLER, not a callee
```

These continuation lines show the call path TO the function, not calls FROM it. The parser must:
- Detect leaf functions (Self% ≈ Children%)
- NOT treat caller chain entries as callees
- Only record actual callee relationships from non-leaf entries

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

#### Core Parsing
- **FR-001**: System MUST parse call tree structure from perf report indentation patterns using column position (÷11) for depth calculation.
- **FR-002**: System MUST identify caller-callee relationships between target functions, collapsing intermediate non-target functions via percentage multiplication.
- **FR-003**: System MUST calculate callee's relative contribution to caller by multiplying percentages through intermediate (non-target) functions in the chain.
- **FR-004**: System MUST detect leaf functions (Self% ≈ Children%) and NOT treat their caller chains as callee relationships.

#### Multi-Level Nesting Display
- **FR-005**: System MUST display targeted callees with multi-level indentation: N × 4 spaces for level N.
- **FR-006**: System MUST recursively display callees of callees when both are targets, forming nested hierarchies.
- **FR-007**: System MUST show relative percentages at each nesting level (% of immediate parent, not root).
- **FR-008**: System MUST track "consumed" caller→callee relationships to avoid displaying them multiple times.
- **FR-009**: When a relationship A→B→C is shown nested, B's standalone entry MUST NOT repeat C as a callee.

#### Standalone Entries and Adjustment
- **FR-010**: System MUST display standalone entries with **absolute percentages** (% of total program time).
- **FR-011**: System MUST subtract absolute contributions only from ROOT callers (level 0 callers, not intermediate).
- **FR-012**: System MUST floor adjusted percentages at 0.00% (never display negative).
- **FR-013**: System MUST skip standalone entries with 0.00% adjusted if they have no unique time.

#### Deduplication
- **FR-014**: System MUST deduplicate entries with the same simplified symbol, showing only the first (highest %).
- **FR-015**: System MUST deduplicate callees under a single caller, showing each callee once.
- **FR-016**: System MUST stop traversal at the first occurrence of a target callee to handle recursion.

#### CLI and Compatibility
- **FR-017**: System MUST accept `--hierarchy` / `-H` flag that enables targeted call hierarchy display.
- **FR-018**: System MUST require `--targets` when `--hierarchy` is specified, showing an error otherwise.
- **FR-019**: System MUST preserve existing flat output when `--hierarchy` is not specified (backward compatibility).
- **FR-020**: System MUST apply symbol simplification and color coding from feature 002.
- **FR-021**: System MUST handle multiple callers of the same callee, showing the callee under each caller.

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

### Example 1: Two-Level Hierarchy

Given targets `rd_optimize_transform` (71.80% absolute) and `DCT4DBlock` (38.00% absolute standalone):

```
Children%   Self%  Function
   71.80    0.00  rd_optimize_transform
   17.23    0.00      DCT4DBlock            <- RELATIVE: 17.23% of rd_optimize_transform's time
   25.63    5.00  DCT4DBlock                <- ABSOLUTE: 38.00% - (71.80% × 17.23%) = 25.63%
```

**Interpretation**:
- Indented 17.23%: "DCT4DBlock takes 17.23% of rd_optimize_transform's time"
- Standalone 25.63%: "DCT4DBlock takes 25.63% of total time NOT via rd_optimize_transform"

### Example 2: Three-Level Multi-Nesting

Given targets A (80%), B (called by A at 50%), C (called by B at 40%):

```
Children%   Self%  Function
   80.00    0.00  A                         <- root caller
   50.00    0.00      B                     <- B is 50% of A (level 1)
   40.00    0.00          C                 <- C is 40% of B (level 2)
   40.00    0.00  B                         <- B standalone: 80% - 40% = 40%
                                            <- NO C here (consumed under A→B)
    4.00    0.00  C                         <- C standalone: original - (80% × 50% × 40%) = original - 16%
```

**Key points**:
- B under A shows C nested (8-space indent)
- B standalone does NOT repeat C (relationship consumed)
- C's adjustment uses A's contribution (16%), not B's

### Example 3: Independent Branches

Given A→B→C and separately D→C:

```
Children%   Self%  Function
   80.00    0.00  A
   50.00    0.00      B
   40.00    0.00          C                 <- C via A→B (16% absolute contribution)
   30.00    0.00  D
   20.00    0.00      C                     <- C via D (6% absolute contribution)
   40.00    0.00  B                         <- B standalone (C NOT shown, consumed above)
    8.00    0.00  C                         <- C standalone: original - 16% - 6%
```

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
