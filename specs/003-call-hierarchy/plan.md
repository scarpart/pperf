# Implementation Plan: Targeted Call Hierarchy Display

**Branch**: `003-call-hierarchy` | **Date**: 2026-01-02 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/003-call-hierarchy/spec.md`

## Summary

Add `--hierarchy` flag that, when used with `--targets`, displays targeted caller-callee relationships. Callees appear indented under callers with relative percentages (% of caller's time), while standalone entries show absolute percentages (% of total). The system parses perf report call trees, collapses intermediate non-target functions, handles recursion by stopping at first occurrence, and adjusts standalone percentages by subtracting accounted callee contributions.

## Technical Context

**Language/Version**: Rust (latest stable, 2024 edition)
**Primary Dependencies**: None (standard library only per constitution)
**Storage**: N/A (reads perf-report.txt files)
**Testing**: `cargo test` with TDD (NON-NEGOTIABLE per constitution)
**Target Platform**: Linux (where perf is available)
**Project Type**: Single CLI application
**Performance Goals**: Process 10k+ line perf reports in <1 second
**Constraints**: No external crates; must validate against real perf-report.txt samples
**Scale/Scope**: ~500 additional lines of code for hierarchy module

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Test-First Development | ✅ PASS | TDD cycle mandatory; tests written before implementation |
| II. Simplicity-First Design | ✅ PASS | Single new module; reuses existing parser infrastructure |
| III. Real Data Validation | ✅ PASS | Tests will use real perf-report.txt with known call relationships |
| IV. Incremental Feature Development | ✅ PASS | Builds on feature 002; clear testable scope |
| No External Dependencies | ✅ PASS | Standard library only |

## Project Structure

### Documentation (this feature)

```text
specs/003-call-hierarchy/
├── plan.md              # This file
├── research.md          # Call tree parsing research
├── data-model.md        # CallNode, CallRelation, HierarchyEntry
├── quickstart.md        # Usage examples
├── contracts/           # CLI contract updates
│   └── cli.md
└── tasks.md             # Implementation tasks (Phase 2)
```

### Source Code (repository root)

```text
src/
├── lib.rs               # Add pub mod hierarchy
├── main.rs              # Add --hierarchy flag handling
├── parser.rs            # Existing - may need call tree parsing extension
├── hierarchy.rs         # NEW: Call tree parsing and hierarchy computation
├── filter.rs            # Existing
├── output.rs            # Existing - modify for hierarchy output
└── symbol.rs            # Existing (feature 002)

tests/
├── top_command.rs       # Add hierarchy integration tests
└── fixtures/
    └── perf-report.txt  # Existing real test data
```

**Structure Decision**: Single project layout. New `hierarchy.rs` module handles call tree parsing, target relationship discovery, and percentage adjustment. Minimal changes to existing modules.

## Complexity Tracking

> No violations to justify - design stays within constitution constraints.

| Aspect | Complexity Level | Justification |
|--------|------------------|---------------|
| New Module | Low | Single `hierarchy.rs` adds focused functionality |
| Algorithm | Medium | Call tree traversal with recursion detection |
| Integration | Low | Hooks into existing parser and output infrastructure |

---

## Implementation Phases

### Phase 1: Call Tree Parsing (TDD)

**Goal**: Parse call tree lines from perf report into hierarchical data structures.

**Files**: `src/hierarchy.rs` (new), `src/lib.rs` (add module)

**Implementation Steps**:
1. Create `CallTreeLine` struct to represent a single call tree line
2. Implement `parse_call_tree_line()` to extract depth, percentage, symbol
3. Create `CallTreeNode` struct for hierarchical representation
4. Implement `build_call_tree()` to convert flat lines into tree structure
5. Handle depth tracking via `|` count for parent-child relationships

**TDD Cycle**:
- Write tests using real perf-report.txt call tree sections
- Test depth calculation from indentation
- Test percentage extraction from `--XX.XX%--` pattern
- Test tree building with proper parent-child relationships

---

### Phase 2: Target Relationship Discovery (TDD)

**Goal**: Find caller-callee relationships between target functions.

**Files**: `src/hierarchy.rs`

**Implementation Steps**:
1. Create `CallRelation` struct with relative and absolute percentages
2. Implement `find_target_callees()` - traverse from target looking for other targets
3. Implement recursion detection using `HashSet<String>` of seen targets
4. Implement percentage multiplication for intermediate function collapsing
5. Create `HierarchyEntry` struct for display data

**TDD Cycle**:
- Test simple caller-callee: A→B where both are targets
- Test intermediate collapsing: A→X→B where only A,B are targets
- Test recursive function handling
- Test multiple callers of same callee

---

### Phase 3: Percentage Adjustment (TDD)

**Goal**: Compute adjusted standalone percentages.

**Files**: `src/hierarchy.rs`

**Implementation Steps**:
1. Implement `compute_adjusted_percentages()`:
   - Sum absolute contributions for each target that appears as callee
   - Subtract from original Children% to get adjusted
   - Floor at 0.0
2. Integrate with HierarchyEntry

**TDD Cycle**:
- Test single caller contributing to callee
- Test multiple callers contributing to same callee
- Test adjusted percentage flooring at zero

---

### Phase 4: CLI Integration

**Goal**: Add `--hierarchy` flag and integrate with main flow.

**Files**: `src/main.rs`, `src/output.rs`

**Implementation Steps**:
1. Parse `--hierarchy` / `-H` flag in argument handling
2. Validate `--hierarchy` requires `--targets`
3. Extract call tree data from perf report file
4. Build hierarchy and compute relationships
5. Modify output formatting for indented entries

**TDD Cycle**:
- Integration test: verify `--hierarchy` requires `--targets`
- Integration test: verify indented output format
- Integration test: verify with real perf-report.txt data

---

## Testing Strategy

| Test Type | Coverage | Location |
|-----------|----------|----------|
| Unit | Call tree parsing | `src/hierarchy.rs` (inline tests) |
| Unit | Relationship discovery | `src/hierarchy.rs` (inline tests) |
| Unit | Percentage adjustment | `src/hierarchy.rs` (inline tests) |
| Integration | Full CLI flow | `tests/top_command.rs` |
| Real Data | Validation against perf-report.txt | All test files |

**Key Test Scenarios from spec.md**:
1. `rd_optimize_transform` → `DCT4DBlock` (17.23% relative)
2. `rd_optimize_transform` → `rd_optimize_hexadecatree` (recursive)
3. Multiple callers of same function
4. Adjusted standalone calculation

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Complex call tree format variations | Parse defensively; test with real data |
| Recursive function infinite loops | Track seen targets; stop at first occurrence |
| Percentage precision loss | Use f64; document rounding behavior |
| Integration with existing code | Minimal changes to main.rs; new module encapsulates complexity |
