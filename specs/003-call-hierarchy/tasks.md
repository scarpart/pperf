# Tasks: Targeted Call Hierarchy Display

**Input**: Design documents from `/specs/003-call-hierarchy/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/cli.md

**Tests**: REQUIRED per constitution (TDD is NON-NEGOTIABLE)

**Organization**: Tasks are grouped by implementation phase following plan.md structure.

## Format: `[ID] [P?] [Phase] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Phase]**: Which implementation phase (P1-P4 for call tree parsing, relationship discovery, adjustment, CLI)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root
- Rust crate structure per plan.md
- New module: `src/hierarchy.rs`

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: New module creation and data structures

- [x] T001 Create src/hierarchy.rs with module declaration and CallTreeLine struct
- [x] T002 Add `pub mod hierarchy;` declaration in src/lib.rs
- [x] T003 Define CallTreeNode struct in src/hierarchy.rs
- [x] T004 Define CallRelation struct in src/hierarchy.rs
- [x] T005 Define HierarchyEntry struct in src/hierarchy.rs

**Checkpoint**: New module compiles with `cargo build` ✓

---

## Phase 2: Call Tree Parsing (TDD)

**Purpose**: Parse call tree lines from perf report into data structures

### Tests for Call Tree Parsing (TDD - Write First)

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T006 [P] [P2] Unit test for parse_call_tree_line() with percentage line `|--17.23%--function` in src/hierarchy.rs
- [x] T007 [P] [P2] Unit test for parse_call_tree_line() with no percentage (function continuation) in src/hierarchy.rs
- [x] T008 [P] [P2] Unit test for parse_call_tree_line() with top-level entry line in src/hierarchy.rs
- [x] T009 [P] [P2] Unit test for count_depth() returning correct depth from pipe count in src/hierarchy.rs
- [x] T010 [P] [P2] Unit test for extract_percentage() from `--XX.XX%--` pattern in src/hierarchy.rs
- [x] T011 [P] [P2] Unit test for extract_symbol() stripping percentage markers in src/hierarchy.rs
- [x] T012 [P2] Unit test for build_call_tree() constructing proper parent-child hierarchy in src/hierarchy.rs

### Implementation for Call Tree Parsing

- [x] T013 [P2] Implement count_depth(line: &str) -> usize counting `|` characters in src/hierarchy.rs
- [x] T014 [P2] Implement extract_percentage(line: &str) -> Option<f64> from `--XX.XX%--` pattern in src/hierarchy.rs
- [x] T015 [P2] Implement extract_symbol(line: &str) -> Option<String> extracting function name in src/hierarchy.rs
- [x] T016 [P2] Implement parse_call_tree_line(line: &str) -> Option<CallTreeLine> in src/hierarchy.rs
- [x] T017 [P2] Implement build_call_tree(lines: &[CallTreeLine]) -> Vec<CallTreeNode> in src/hierarchy.rs
- [x] T018 [P2] Unit test for parse_file_call_trees() extracting call tree sections from real perf-report.txt
- [x] T019 [P2] Implement parse_file_call_trees(content: &str) -> Vec<(PerfEntry, Vec<CallTreeNode>)> in src/hierarchy.rs
- [x] T020 [P2] Verify all P2 tests pass with `cargo test`

**Checkpoint**: Can parse call tree from perf-report.txt into CallTreeNode hierarchy ✓

---

## Phase 3: Target Relationship Discovery (TDD)

**Purpose**: Find caller-callee relationships between target functions

### Tests for Relationship Discovery (TDD - Write First)

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T021 [P] [P3] Unit test for find_target_in_tree() locating target node in tree in src/hierarchy.rs
- [x] T022 [P] [P3] Unit test for find_target_callees() finding direct target callee in src/hierarchy.rs
- [x] T023 [P] [P3] Unit test for find_target_callees() collapsing intermediate non-target via multiplication in src/hierarchy.rs
- [x] T024 [P] [P3] Unit test for find_target_callees() handling recursive function (stop at first occurrence) in src/hierarchy.rs
- [x] T025 [P] [P3] Unit test for find_target_callees() with multiple targets finding all relationships in src/hierarchy.rs
- [x] T026 [P3] Unit test for compute_call_relations() with real data (rd_optimize_transform → DCT4DBlock) in src/hierarchy.rs

### Implementation for Relationship Discovery

- [x] T027 [P3] Implement find_target_in_tree(tree: &CallTreeNode, target: &str) -> bool in src/hierarchy.rs
- [x] T028 [P3] Implement find_target_callees() with recursion detection using HashSet in src/hierarchy.rs
- [x] T029 [P3] Implement percentage multiplication through intermediate functions in find_target_callees() in src/hierarchy.rs
- [x] T030 [P3] Implement compute_call_relations(trees: &[(PerfEntry, Vec<CallTreeNode>)], targets: &[String]) -> Vec<CallRelation> in src/hierarchy.rs
- [x] T031 [P3] Verify all P3 tests pass with `cargo test`

**Checkpoint**: Can discover caller-callee relationships between targets with correct percentages ✓

---

## Phase 4: Percentage Adjustment (TDD)

**Purpose**: Compute adjusted standalone percentages

### Tests for Percentage Adjustment (TDD - Write First)

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T032 [P] [P4] Unit test for compute_adjusted_percentage() with single caller contribution in src/hierarchy.rs
- [x] T033 [P] [P4] Unit test for compute_adjusted_percentage() with multiple caller contributions in src/hierarchy.rs
- [x] T034 [P] [P4] Unit test for compute_adjusted_percentage() flooring at zero in src/hierarchy.rs
- [x] T035 [P4] Unit test for build_hierarchy_entries() constructing full HierarchyEntry list in src/hierarchy.rs

### Implementation for Percentage Adjustment

- [x] T036 [P4] Implement compute_adjusted_percentage(original: f64, contributions: &[f64]) -> f64 in src/hierarchy.rs
- [x] T037 [P4] Implement build_hierarchy_entries() assembling HierarchyEntry with callees and adjusted % in src/hierarchy.rs
- [x] T038 [P4] Verify all P4 tests pass with `cargo test`

**Checkpoint**: Can compute adjusted standalone percentages correctly ✓

---

## Phase 5: CLI Integration

**Purpose**: Add --hierarchy flag and integrate with main flow

### Tests for CLI Integration (TDD - Write First)

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T039 [P] [CLI] Integration test for --hierarchy without --targets returning error in tests/top_command.rs
- [x] T040 [P] [CLI] Integration test for --hierarchy with --targets producing indented output in tests/top_command.rs
- [x] T041 [P] [CLI] Integration test verifying indented entries show relative % in tests/top_command.rs
- [x] T042 [P] [CLI] Integration test verifying standalone entries show absolute adjusted % in tests/top_command.rs
- [x] T043 [CLI] Integration test with real perf-report.txt (rd_optimize_transform → DCT4DBlock) in tests/top_command.rs

### Implementation for CLI Integration

- [x] T044 [CLI] Add --hierarchy / -H flag parsing to run_top() in src/main.rs
- [x] T045 [CLI] Add validation: --hierarchy requires --targets in src/main.rs
- [x] T046 [CLI] Add HierarchyRequiresTargets error variant to PperfError in src/lib.rs
- [x] T047 [CLI] Implement format_hierarchy_table() for indented output in src/output.rs
- [x] T048 [CLI] Wire hierarchy computation into run_top() when --hierarchy is specified in src/main.rs
- [x] T049 [CLI] Update help text to document --hierarchy flag in src/main.rs
- [x] T050 [CLI] Verify all CLI tests pass with `cargo test`

**Checkpoint**: `pperf top --hierarchy --targets A B file.txt` produces correct output ✓

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Quality checks and final validation

- [x] T051 Run `cargo fmt` and fix any formatting issues
- [x] T052 Run `cargo clippy` and address all warnings
- [x] T053 Run `cargo build --release` and verify no warnings
- [x] T054 Run full test suite `cargo test` and verify all pass
- [x] T055 Manual validation: run quickstart.md examples against real perf-report.txt
- [x] T056 Verify hierarchy output visually with rd_optimize_transform and DCT4DBlock targets
- [x] T057 Verify recursive function handling with rd_optimize_hexadecatree target

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately ✓
- **Call Tree Parsing (Phase 2)**: Depends on Setup completion ✓
- **Relationship Discovery (Phase 3)**: Depends on Phase 2 completion ✓
- **Percentage Adjustment (Phase 4)**: Depends on Phase 3 completion ✓
- **CLI Integration (Phase 5)**: Depends on Phase 4 completion ✓
- **Polish (Phase 6)**: Depends on all phases being complete ✓

### Within Each Phase (TDD Cycle)

1. Write tests FIRST → verify they FAIL
2. Implement minimal code → verify tests PASS
3. Refactor if needed → verify tests still PASS
4. Commit

### Parallel Opportunities

- All Phase 2 tests marked [P] can run in parallel (T006-T011)
- All Phase 3 tests marked [P] can run in parallel (T021-T025)
- All Phase 4 tests marked [P] can run in parallel (T032-T034)
- All CLI tests marked [P] can run in parallel (T039-T042)

---

## Key Test Scenarios from Spec

### Scenario 1: Simple Caller-Callee (T026, T043)

**Targets**: `rd_optimize_transform`, `DCT4DBlock`

**Expected**:
- DCT4DBlock appears indented under rd_optimize_transform at 17.23% (relative)
- DCT4DBlock standalone shows adjusted absolute % (original - 71.80% × 17.23%)

### Scenario 2: Recursive Function (T024, T057)

**Targets**: `rd_optimize_transform`, `rd_optimize_hexadecatree`

**Expected**:
- rd_optimize_hexadecatree appears once under rd_optimize_transform
- Recursion is NOT traversed deeper

### Scenario 3: Intermediate Collapsing (T023)

**Targets**: A, C where A→B→C (B not a target)

**Expected**:
- C appears under A with percentage = B.relative × C.relative / 100
- B is NOT shown in output

### Scenario 4: Multiple Callers (T025)

**Targets**: A, B, C where A→C and B→C

**Expected**:
- C appears indented under A with A's relative %
- C appears indented under B with B's relative %
- C standalone shows adjusted % (original - A.contribution - B.contribution)

---

## Notes

- [P] tasks = different files or test cases, no dependencies on incomplete tasks
- [Phase] label maps task to implementation phase for traceability
- TDD is mandatory per constitution - tests MUST fail before implementation
- Real perf-report.txt from repository MUST be used for integration tests
- No external dependencies - standard library only
- Symbol simplification (feature 002) already handles function name cleanup
- Colors (feature 002) already applied to output
- Verify tests fail for the right reason before implementing
- Commit after each completed task or logical group
