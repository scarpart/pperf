# Tasks: Multi-Report Averaging

**Input**: Design documents from `/specs/006-multi-report-averaging/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md

**Tests**: Constitution mandates TDD (Test-First Development is NON-NEGOTIABLE). All tests must be written FIRST and FAIL before implementation.

**Organization**: Tasks grouped by user story. US1 and US2 are combined (both P1, tightly coupled - can't average without correct matching).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Module scaffolding and CLI changes

- [X] T001 Create averaging module skeleton in src/averaging.rs with AveragedPerfEntry and ReportSet structs
- [X] T002 Export averaging module from src/lib.rs
- [X] T003 Update TopArgs struct to accept Vec<PathBuf> instead of single PathBuf in src/main.rs

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core data structures that MUST be complete before user stories can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [X] T004 Implement PerfEntry to AveragedPerfEntry conversion (single-file case) in src/averaging.rs
- [X] T005 [P] Add integration test fixture: copy examples/Bikes_005_rep*.txt to tests/fixtures/

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1+2 - Core Averaging with Function Matching (Priority: P1) 🎯 MVP

**Goal**: Parse multiple perf report files, match functions by full signature, compute averaged percentages

**Independent Test**: Provide 3 perf reports, verify output shows correctly averaged Children% and Self%

### Tests for User Story 1+2 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T006 [P] [US1] Unit test: average_entries aggregates by symbol in src/averaging.rs tests
- [X] T007 [P] [US1] Unit test: arithmetic mean calculation is correct in src/averaging.rs tests
- [X] T008 [P] [US2] Unit test: functions with same simplified name but different signatures are distinct in src/averaging.rs tests
- [X] T009 [P] [US2] Unit test: function present in only some reports averages over present count in src/averaging.rs tests
- [X] T010 [US1] Integration test: multi-file CLI invocation parses all files in tests/top_command.rs
- [X] T011 [US1] Integration test: averaged output matches manual calculation on real examples in tests/top_command.rs

**Checkpoint**: Tests written and FAILING - proceed to implementation

### Implementation for User Story 1+2

- [X] T012 [US1] Implement ReportSet::parse_all() to parse multiple files in src/averaging.rs
- [X] T013 [US2] Implement symbol-keyed HashMap aggregation in ReportSet in src/averaging.rs
- [X] T014 [US1] Implement ReportSet::average() to compute arithmetic mean per function in src/averaging.rs
- [X] T015 [US1] Wire multi-file parsing in run_top() to use ReportSet in src/main.rs
- [X] T016 [US1] Update filter_entries to work with AveragedPerfEntry in src/filter.rs
- [X] T017 [US1] Update format_table to accept AveragedPerfEntry in src/output.rs
- [X] T018 [US1] Verify all T006-T011 tests now PASS

**Checkpoint**: User Story 1+2 complete - multi-file averaging works with correct function matching

---

## Phase 4: User Story 3 - Debug Mode Per-Report Breakdown (Priority: P2)

**Goal**: Show individual percentage values from each report file alongside averaged values in debug mode

**Independent Test**: Run with `--debug` flag on multiple files, verify per-report values annotation appears

### Tests for User Story 3 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T019 [P] [US3] Unit test: format debug annotation with per-report values in src/output.rs tests
- [X] T020 [P] [US3] Unit test: missing report shows dash (-) in per-report values in src/output.rs tests
- [X] T021 [US3] Integration test: --debug with multi-file shows (values: ...) annotation in tests/top_command.rs

**Checkpoint**: Tests written and FAILING - proceed to implementation

### Implementation for User Story 3

- [X] T022 [US3] Implement format_per_report_values helper in src/output.rs
- [X] T023 [US3] Update format_table to output per-report annotation line when debug=true and report_count>1 in src/output.rs
- [X] T024 [US3] Update format_hierarchy_table to include per-report annotations for hierarchy entries in src/output.rs
- [X] T025 [US3] Verify all T019-T021 tests now PASS

**Checkpoint**: User Story 3 complete - debug mode shows per-report breakdown

---

## Phase 5: User Story 4 - Backward Compatibility (Priority: P2)

**Goal**: Single-file input produces identical output to current behavior (no regression)

**Independent Test**: Compare single-file output before and after feature, verify identical

### Tests for User Story 4 ⚠️

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [X] T026 [P] [US4] Integration test: single-file output unchanged from baseline in tests/top_command.rs
- [X] T027 [P] [US4] Integration test: single-file with --debug shows no per-report annotation in tests/top_command.rs

**Checkpoint**: Tests written and FAILING - proceed to implementation (may already pass)

### Implementation for User Story 4

- [X] T028 [US4] Add conditional in format_table: skip per-report annotation when report_count==1 in src/output.rs
- [X] T029 [US4] Verify T026-T027 tests now PASS

**Checkpoint**: User Story 4 complete - backward compatibility verified

---

## Phase 6: Hierarchy Integration

**Goal**: Extend hierarchy mode to work with averaged data

### Tests for Hierarchy Integration ⚠️

- [X] T030 [P] Unit test: averaged call relations computed correctly in src/hierarchy.rs tests
- [X] T031 Integration test: --hierarchy with multi-file shows averaged relationships in tests/top_command.rs

### Implementation for Hierarchy Integration

- [X] T032 Parse call trees from each file and merge caller→callee relations in src/hierarchy.rs
- [X] T033 Update build_hierarchy_entries to accept AveragedPerfEntry in src/hierarchy.rs
- [X] T034 Wire hierarchy mode to use averaged call relations in src/main.rs
- [X] T035 Verify T030-T031 tests now PASS

**Checkpoint**: Hierarchy mode works with multi-file averaging

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final cleanup and validation

- [X] T036 [P] Update CLAUDE.md Quick Reference to show multi-file usage examples
- [X] T037 [P] Run cargo fmt and cargo clippy, fix any warnings
- [X] T038 Run full test suite: cargo test
- [X] T039 Manual validation: run quickstart.md examples with real data
- [X] T040 Verify all acceptance scenarios from spec.md

---

## Dependencies & Execution Order

### Phase Dependencies

```
Phase 1 (Setup) ─────────────► Phase 2 (Foundational)
                                      │
                                      ▼
                              Phase 3 (US1+2: Core)
                                      │
                    ┌─────────────────┼─────────────────┐
                    ▼                 ▼                 ▼
           Phase 4 (US3)      Phase 5 (US4)     Phase 6 (Hierarchy)
                    │                 │                 │
                    └─────────────────┴─────────────────┘
                                      │
                                      ▼
                              Phase 7 (Polish)
```

### User Story Dependencies

- **US1+2 (P1)**: Can start after Phase 2 - No dependencies on other stories
- **US3 (P2)**: Depends on US1+2 completion (needs AveragedPerfEntry with per_report_values)
- **US4 (P2)**: Can start after US1+2 (tests backward compat after change)
- **Hierarchy**: Can start after US1+2 (extends with averaged data)

### Parallel Opportunities

Within Phase 3 (US1+2):
- T006, T007, T008, T009 can run in parallel (different test cases)

Within Phase 4 (US3):
- T019, T020 can run in parallel (different test cases)

Within Phase 5 (US4):
- T026, T027 can run in parallel (different test cases)

After Phase 3 completes:
- Phase 4, Phase 5, Phase 6 can run in parallel (independent stories)

---

## Parallel Example: Phase 3 Tests

```bash
# Launch all unit tests for US1+2 together:
Task: "Unit test: average_entries aggregates by symbol in src/averaging.rs tests"
Task: "Unit test: arithmetic mean calculation is correct in src/averaging.rs tests"
Task: "Unit test: functions with same simplified name but different signatures are distinct"
Task: "Unit test: function present in only some reports averages over present count"
```

---

## Implementation Strategy

### MVP First (User Story 1+2 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: US1+2 (Core Averaging)
4. **STOP and VALIDATE**: Test multi-file averaging independently
5. Deploy/demo if ready - basic averaging works!

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. Add US1+2 → Test with real examples → MVP complete!
3. Add US3 → Debug mode shows per-report values → Enhanced visibility
4. Add US4 → Verify backward compatibility → Safe for existing users
5. Add Hierarchy → Full feature parity with single-file mode
6. Polish → Documentation, cleanup

---

## Summary

| Phase | Story | Task Count | Parallel Tasks |
|-------|-------|------------|----------------|
| 1. Setup | - | 3 | 0 |
| 2. Foundational | - | 2 | 1 |
| 3. US1+2 Core | P1 | 13 | 6 |
| 4. US3 Debug | P2 | 7 | 2 |
| 5. US4 Compat | P2 | 4 | 2 |
| 6. Hierarchy | - | 6 | 1 |
| 7. Polish | - | 5 | 2 |
| **Total** | | **40** | **14** |

**MVP Scope**: Phases 1-3 (18 tasks) delivers core multi-file averaging

**TDD Compliance**: 14 test tasks (T006-T011, T019-T021, T026-T027, T030-T031) written before implementation
