# Tasks: Debug Calculation Path

**Input**: Design documents from `/specs/004-debug-calculation-path/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: REQUIRED per constitution (Test-First Development is NON-NEGOTIABLE)

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root (Rust CLI application)

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Add foundational color support and data model extensions needed by all user stories

- [ ] T001 [P] Add DIM color constant to src/symbol.rs for gray annotation text
- [ ] T002 [P] Add IntermediaryStep struct to src/hierarchy.rs (symbol: String, percentage: f64)
- [ ] T003 Add intermediary_path field to CallRelation struct in src/hierarchy.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T004 Parse --debug/-D flag in src/main.rs argument handling
- [ ] T005 Pass debug_flag parameter through to format_hierarchy_table in src/main.rs
- [ ] T006 Update format_hierarchy_table signature to accept debug: bool parameter in src/output.rs

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - View Calculation Breakdown for Indirect Calls (Priority: P1) 🎯 MVP

**Goal**: Show multiplication chain annotation for calls that traverse non-target intermediary functions

**Independent Test**: Run `pperf top --hierarchy --debug -t rd_optimize DCT4DBlock inner_product perf-report.txt` and verify indirect calls show `(via X% × Y% = Z%)` format on gray line below

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation (TDD Red phase)**

- [ ] T007 [P] [US1] Unit test for IntermediaryStep struct creation in src/hierarchy.rs tests
- [ ] T008 [P] [US1] Unit test for format_debug_annotation with single intermediary in src/output.rs tests
- [ ] T009 [P] [US1] Unit test for format_debug_annotation with multiple intermediaries in src/output.rs tests
- [ ] T010 [US1] Integration test for --hierarchy --debug with indirect call in tests/ (real perf-report.txt)

### Implementation for User Story 1

- [ ] T011 [US1] Modify find_target_callees to track intermediary_path during DFS in src/hierarchy.rs
- [ ] T012 [US1] Implement format_debug_annotation function for indirect calls in src/output.rs
- [ ] T013 [US1] Call format_debug_annotation after each callee entry in display_callees_with_context in src/output.rs
- [ ] T014 [US1] Apply DIM color to annotation output when use_color is true in src/output.rs

**Checkpoint**: User Story 1 should now show indirect call annotations with intermediary chain

---

## Phase 4: User Story 2 - View Confirmation for Direct Calls (Priority: P2)

**Goal**: Show `(direct: X%)` annotation for calls with no intermediaries, ensuring visual consistency

**Independent Test**: Run with --debug and verify direct caller→callee relationships show `(direct: X%)` on gray line

### Tests for User Story 2

- [ ] T015 [P] [US2] Unit test for format_debug_annotation with empty intermediary_path (direct call) in src/output.rs tests
- [ ] T016 [US2] Integration test for --hierarchy --debug with direct call in tests/ (real perf-report.txt)

### Implementation for User Story 2

- [ ] T017 [US2] Handle empty intermediary_path case in format_debug_annotation (return "direct: X%") in src/output.rs
- [ ] T018 [US2] Verify mixed direct/indirect output in integration test

**Checkpoint**: Both direct and indirect calls now have consistent annotation format

---

## Phase 5: User Story 3 - Debug Flag Has No Effect Without Hierarchy (Priority: P3)

**Goal**: Ensure --debug without --hierarchy produces normal output (no annotations, no errors)

**Independent Test**: Run `pperf top --debug file` and verify output matches normal mode exactly

### Tests for User Story 3

- [ ] T019 [US3] Integration test comparing --debug output vs normal output (without --hierarchy) in tests/

### Implementation for User Story 3

- [ ] T020 [US3] Verify debug_flag is only used in hierarchy code path in src/main.rs (no changes needed if already correct)

**Checkpoint**: --debug flag gracefully does nothing when --hierarchy is absent

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Edge cases, code quality, and documentation

- [ ] T021 [P] Test edge case: recursive target in intermediary chain in src/hierarchy.rs tests
- [ ] T022 [P] Test edge case: many intermediaries (>5) in src/output.rs tests
- [ ] T023 [P] Test edge case: 0% intermediary in path in src/output.rs tests
- [ ] T024 [P] Test edge case: --debug with --no-color (plain text annotation) in tests/
- [ ] T025 Run cargo fmt on all modified files
- [ ] T026 Run cargo clippy and fix any warnings
- [ ] T027 Run full cargo test suite and verify all tests pass
- [ ] T028 Update CLAUDE.md with --debug flag documentation
- [ ] T029 Validate against quickstart.md examples

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-5)**: All depend on Foundational phase completion
  - User stories can proceed sequentially in priority order (P1 → P2 → P3)
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Builds on US1's format_debug_annotation but is independently testable
- **User Story 3 (P3)**: Independent - tests boundary condition (no --hierarchy)

### Within Each User Story (TDD Cycle)

1. Tests MUST be written FIRST and FAIL before implementation (Red)
2. Implement minimal code to make tests pass (Green)
3. Refactor for clarity (keep tests Green)
4. Commit after each task or logical group

### Parallel Opportunities

- T001, T002: DIM constant and IntermediaryStep struct (different files)
- T007, T008, T009: Unit tests for different functions
- T015, T016: US2 tests
- T021, T022, T023, T024: Edge case tests (different scenarios)

---

## Parallel Example: Phase 1 Setup

```bash
# Launch setup tasks in parallel (different files):
Task: "Add DIM color constant to src/symbol.rs"
Task: "Add IntermediaryStep struct to src/hierarchy.rs"
```

## Parallel Example: User Story 1 Tests

```bash
# Launch unit tests in parallel (different test cases):
Task: "Unit test for IntermediaryStep struct creation"
Task: "Unit test for format_debug_annotation with single intermediary"
Task: "Unit test for format_debug_annotation with multiple intermediaries"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T003)
2. Complete Phase 2: Foundational (T004-T006)
3. Complete Phase 3: User Story 1 (T007-T014)
4. **STOP and VALIDATE**: Run `pperf top --hierarchy --debug -t ... perf-report.txt`
5. Verify indirect calls show multiplication chain in gray

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add User Story 1 → Test with indirect calls → MVP ready!
3. Add User Story 2 → Test with direct calls → Consistent formatting
4. Add User Story 3 → Test without --hierarchy → Scope boundary verified
5. Polish phase → Production ready

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story is independently testable after completion
- TDD is mandatory per constitution: Red → Green → Refactor
- All tests must validate against real perf-report.txt
- Commit after each task or logical group
