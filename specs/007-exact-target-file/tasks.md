# Tasks: Exact Function Signature Target File

**Input**: Design documents from `/specs/007-exact-target-file/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md

**Tests**: Required per constitution (Test-First Development is NON-NEGOTIABLE)

**Organization**: Tasks are grouped by user story to enable independent implementation and testing.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root
- Paths follow existing project structure from plan.md

---

## Phase 1: Setup (Error Types & CLI Extension)

**Purpose**: Add new error types and CLI argument for target file support

- [x] T001 [P] Add `TargetFileNotFound(String)` error variant in src/lib.rs
- [x] T002 [P] Add `EmptyTargetFile` error variant in src/lib.rs
- [x] T003 [P] Add `AmbiguousTarget { signature: String, matches: Vec<String> }` error variant in src/lib.rs
- [x] T004 [P] Add `UnmatchedTargets(Vec<String>)` error variant in src/lib.rs
- [x] T005 Implement Display for new error variants with clear user messages in src/lib.rs
- [x] T006 Add `--target-file <path>` argument with `conflicts_with = "targets"` in src/main.rs

---

## Phase 2: Foundational (Target File Parsing)

**Purpose**: Core infrastructure for parsing target files - blocks all user stories

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

### Tests for Target File Parsing

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T007 [P] Test parse_target_file returns signatures from valid file in src/filter.rs
- [x] T008 [P] Test parse_target_file ignores comment lines starting with # in src/filter.rs
- [x] T009 [P] Test parse_target_file ignores empty and whitespace-only lines in src/filter.rs
- [x] T010 [P] Test parse_target_file trims leading/trailing whitespace from signatures in src/filter.rs
- [x] T011 [P] Test parse_target_file returns EmptyTargetFile error for file with only comments in src/filter.rs

### Implementation for Target File Parsing

- [x] T012 Implement `parse_target_file(path: &Path) -> Result<Vec<String>, PperfError>` in src/filter.rs

**Checkpoint**: Target file parsing ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Provide Target Functions via File (Priority: P1) 🎯 MVP

**Goal**: Accept a target file and perform exact matching against raw perf report symbols

**Independent Test**: Run `pperf top --target-file targets.txt perf-report.txt` with exact signatures and verify only those functions are matched

### Tests for User Story 1

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T013 [P] [US1] Test filter_entries_exact matches entry when signature is exact substring of symbol in src/filter.rs
- [x] T014 [P] [US1] Test filter_entries_exact returns empty for non-matching signature in src/filter.rs
- [x] T015 [P] [US1] Test filter_entries_exact with real signature `DCT4DBlock::DCT4DBlock(Block4D const&, double)` against examples/Bikes_005_rep0.txt in tests/top_command.rs
- [x] T016 [P] [US1] Test end-to-end: --target-file with exact signatures produces correct output in tests/top_command.rs

### Implementation for User Story 1

- [x] T017 [US1] Implement `filter_entries_exact(entries: &[PerfEntry], signatures: &[String]) -> Vec<PerfEntry>` in src/filter.rs
- [x] T018 [US1] Wire --target-file flag in run_top() to call parse_target_file and filter_entries_exact in src/main.rs
- [x] T019 [US1] Add exit code mapping for TargetFileNotFound (exit 1) in src/main.rs

**Checkpoint**: User Story 1 complete - can filter perf entries using exact signatures from file

---

## Phase 4: User Story 2 - Detect Ambiguous Function Signatures (Priority: P1)

**Goal**: Detect when a signature matches multiple distinct functions and report clear error

**Independent Test**: Run with a partial signature like `DCT4DBlock` and verify error message lists all matches

### Tests for User Story 2

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T020 [P] [US2] Test validate_unique_matches returns Ok when each signature matches exactly one unique symbol in src/filter.rs
- [x] T021 [P] [US2] Test validate_unique_matches returns AmbiguousTarget when signature matches multiple distinct symbols in src/filter.rs
- [x] T022 [P] [US2] Test AmbiguousTarget error message includes signature and all matching symbols in src/lib.rs
- [x] T023 [P] [US2] Test end-to-end: --target-file with ambiguous signature shows error with matches in tests/top_command.rs

### Implementation for User Story 2

- [x] T024 [US2] Implement `validate_unique_matches(entries: &[PerfEntry], signatures: &[String]) -> Result<(), PperfError>` in src/filter.rs
- [x] T025 [US2] Call validate_unique_matches before filter_entries_exact in run_top() in src/main.rs
- [x] T026 [US2] Add exit code 5 for AmbiguousTarget error in src/main.rs

**Checkpoint**: User Story 2 complete - ambiguous signatures are detected and reported

---

## Phase 5: User Story 3 - Backward Compatibility with -t Flag (Priority: P2)

**Goal**: Preserve existing `-t` substring matching, validate mutual exclusivity

**Independent Test**: Run `pperf top -t DCT4D perf-report.txt` and verify substring matching works as before

### Tests for User Story 3

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T027 [P] [US3] Test -t flag continues to use substring matching (existing behavior) in tests/top_command.rs
- [x] T028 [P] [US3] Test --target-file and -t together produces conflict error in tests/top_command.rs

### Implementation for User Story 3

- [x] T029 [US3] Verify Clap conflicts_with handles mutual exclusivity (no code change, just validation) in src/main.rs
- [x] T030 [US3] Update --hierarchy check to work with both target modes in src/main.rs

**Checkpoint**: User Story 3 complete - backward compatibility verified

---

## Phase 6: User Story 4 - Helpful Error Messages for Missing Targets (Priority: P2)

**Goal**: Report clear errors when signatures match no entries

**Independent Test**: Run with a non-existent signature and verify error lists unmatched signatures

### Tests for User Story 4

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T031 [P] [US4] Test detect_unmatched_targets returns empty vec when all signatures match in src/filter.rs
- [x] T032 [P] [US4] Test detect_unmatched_targets returns list of unmatched signatures in src/filter.rs
- [x] T033 [P] [US4] Test UnmatchedTargets error message lists all unmatched signatures in src/lib.rs
- [x] T034 [P] [US4] Test end-to-end: --target-file with non-existent signature shows error in tests/top_command.rs

### Implementation for User Story 4

- [x] T035 [US4] Implement `detect_unmatched_targets(entries: &[PerfEntry], signatures: &[String]) -> Vec<String>` in src/filter.rs
- [x] T036 [US4] Call detect_unmatched_targets and return UnmatchedTargets error if non-empty in run_top() in src/main.rs
- [x] T037 [US4] Add exit code 6 for UnmatchedTargets and EmptyTargetFile errors in src/main.rs

**Checkpoint**: User Story 4 complete - missing targets are clearly reported

---

## Phase 7: Hierarchy Integration

**Purpose**: Ensure --hierarchy works correctly with --target-file mode

- [x] T038 [P] Test --target-file with --hierarchy produces hierarchy output with exact matches in tests/top_command.rs
- [x] T039 Update find_target_in_tree to use exact matching when in exact mode in src/hierarchy.rs
- [x] T040 Update find_target_callees to use exact matching when in exact mode in src/hierarchy.rs
- [x] T041 Pass target mode through hierarchy computation in src/main.rs

---

## Phase 8: Polish & Documentation

**Purpose**: Final cleanup and documentation

- [x] T042 [P] Run cargo clippy and fix any warnings
- [x] T043 [P] Run cargo fmt to ensure consistent formatting
- [x] T044 [P] Update CLAUDE.md with feature 007 in Features Implemented section
- [x] T045 Run quickstart.md validation - test all documented commands work

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 (error types) - BLOCKS all user stories
- **User Story 1 (Phase 3)**: Depends on Phase 2 (parsing infrastructure)
- **User Story 2 (Phase 4)**: Depends on Phase 3 (needs filter_entries_exact)
- **User Story 3 (Phase 5)**: Depends on Phase 1 only (just validation)
- **User Story 4 (Phase 6)**: Depends on Phase 3 (needs filtering logic)
- **Hierarchy (Phase 7)**: Depends on Phases 3-4 (needs exact matching)
- **Polish (Phase 8)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Core feature - other stories build on it
- **User Story 2 (P1)**: Depends on US1's filter_entries_exact function
- **User Story 3 (P2)**: Independent - just validates existing behavior
- **User Story 4 (P2)**: Depends on US1's matching logic

### Within Each User Story

- Tests MUST be written and FAIL before implementation
- Implementation follows test definitions
- Story complete before moving to next priority

### Parallel Opportunities

- Phase 1: T001-T004 can all run in parallel (different error variants)
- Phase 2: T007-T011 tests can all run in parallel
- Phase 3: T013-T016 tests can all run in parallel
- Phase 4: T020-T023 tests can all run in parallel
- Phase 5: T027-T028 tests can run in parallel
- Phase 6: T031-T034 tests can run in parallel
- Phase 8: T042-T044 can all run in parallel

---

## Parallel Example: Phase 1 Setup

```bash
# Launch all error type additions together:
Task: "Add TargetFileNotFound(String) error variant in src/lib.rs"
Task: "Add EmptyTargetFile error variant in src/lib.rs"
Task: "Add AmbiguousTarget error variant in src/lib.rs"
Task: "Add UnmatchedTargets(Vec<String>) error variant in src/lib.rs"
```

## Parallel Example: User Story 1 Tests

```bash
# Launch all US1 tests together before implementation:
Task: "Test filter_entries_exact matches entry when signature is exact substring in src/filter.rs"
Task: "Test filter_entries_exact returns empty for non-matching signature in src/filter.rs"
Task: "Test filter_entries_exact with real signature against examples/ in tests/top_command.rs"
Task: "Test end-to-end --target-file with exact signatures in tests/top_command.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (error types, CLI arg)
2. Complete Phase 2: Foundational (target file parsing)
3. Complete Phase 3: User Story 1 (exact matching)
4. **STOP and VALIDATE**: Test `--target-file` with exact signatures
5. Can ship MVP at this point

### Incremental Delivery

1. Setup + Foundational → Parsing ready
2. Add User Story 1 → Exact matching works → MVP ready!
3. Add User Story 2 → Ambiguity detection → Safer usage
4. Add User Story 3 → Backward compatibility verified
5. Add User Story 4 → Better error messages
6. Add Hierarchy Integration → Full feature complete

---

## Notes

- Constitution requires TDD: Write failing tests first, then implement
- Real data validation: Use examples/Bikes_005_rep0.txt for integration tests
- Exit codes: 5 for ambiguity, 6 for unmatched/empty target file
- Matching: Use `entry.symbol.contains(signature)` not equality
- Ambiguity: Detected by counting distinct matching symbols, not entry count
