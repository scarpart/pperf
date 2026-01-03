# Tasks: List Top Functions

**Input**: Design documents from `/specs/001-list-top-functions/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/cli.md

**Tests**: REQUIRED per constitution (TDD is NON-NEGOTIABLE)

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root
- Rust crate structure per plan.md

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [x] T001 Initialize Rust project with `cargo init --name pperf` in repository root
- [x] T002 Configure Cargo.toml with edition = "2024", version = "0.1.0", no external dependencies
- [x] T003 [P] Create src/lib.rs with module declarations (parser, filter, output)
- [x] T004 [P] Create empty module files: src/parser.rs, src/filter.rs, src/output.rs
- [x] T005 Create tests/integration directory structure
- [x] T006 [P] Create symlink tests/fixtures/perf-report.txt pointing to repository root perf-report.txt

**Checkpoint**: Project compiles with `cargo build`, module structure in place

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core data types and error handling that ALL user stories depend on

**CRITICAL**: No user story work can begin until this phase is complete

### Tests for Foundational

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T007 [P] Unit test for PerfEntry struct creation in src/parser.rs (inline #[cfg(test)])
- [x] T008 [P] Unit test for SortOrder enum variants in src/parser.rs (inline #[cfg(test)])
- [x] T009 [P] Unit test for error types (FileNotFound, InvalidFormat) in src/lib.rs (inline #[cfg(test)])

### Implementation for Foundational

- [x] T010 Define PerfEntry struct (children_pct: f64, self_pct: f64, symbol: String) in src/parser.rs
- [x] T011 [P] Define SortOrder enum (Children, Self) in src/parser.rs
- [x] T012 [P] Define PperfError enum (FileNotFound, InvalidFormat, InvalidCount, NoMatches) in src/lib.rs
- [x] T013 Implement Display trait for PperfError with user-friendly messages in src/lib.rs
- [x] T014 Verify all foundational tests pass with `cargo test`

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - View Top Functions by Children Time (Priority: P1)

**Goal**: Parse perf report and display top 10 functions sorted by Children%

**Independent Test**: `pperf top perf-report.txt` outputs sorted table with correct percentages

### Tests for User Story 1 (TDD - Write First)

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [x] T015 [P] [US1] Unit test for parse_line() recognizing valid data lines in src/parser.rs
- [x] T016 [P] [US1] Unit test for parse_line() skipping comment lines in src/parser.rs
- [x] T017 [P] [US1] Unit test for parse_line() skipping call tree indented lines in src/parser.rs
- [x] T018 [P] [US1] Unit test for parse_file() extracting entries from real perf-report.txt in src/parser.rs
- [x] T019 [P] [US1] Unit test for format_table() producing aligned output in src/output.rs
- [x] T020 [US1] Integration test for `pperf top perf-report.txt` in tests/top_command.rs

### Implementation for User Story 1

- [x] T021 [US1] Implement parse_line() to extract Children%, Self%, Symbol from data lines in src/parser.rs
- [x] T022 [US1] Implement parse_file() to read file and collect PerfEntry vec in src/parser.rs
- [x] T023 [US1] Implement sort_entries() to sort by Children% descending (default) in src/parser.rs
- [x] T024 [US1] Implement format_table() for aligned column output (Children%, Self%, Function) in src/output.rs
- [x] T025 [US1] Implement truncate_symbol() for 100-char limit with "..." in src/output.rs
- [x] T026 [US1] Implement main.rs CLI with `top` subcommand accepting FILE argument
- [x] T027 [US1] Wire up: read file → parse → sort → take 10 → format → print in src/main.rs
- [x] T028 [US1] Implement file not found error handling with exit code 1 in src/main.rs
- [x] T029 [US1] Implement invalid format error handling with exit code 2 in src/main.rs
- [x] T030 [US1] Verify all US1 tests pass with `cargo test`

**Checkpoint**: `pperf top perf-report.txt` works and shows top 10 by Children%

---

## Phase 4: User Story 2 - View Top Functions by Self Time (Priority: P2)

**Goal**: Add --self flag to sort by Self% instead of Children%

**Independent Test**: `pperf top --self perf-report.txt` outputs sorted by Self%

### Tests for User Story 2 (TDD - Write First)

- [x] T031 [P] [US2] Unit test for sort_entries() with SortOrder::Self in src/parser.rs
- [x] T032 [P] [US2] Unit test for tie-breaking (equal Self% → secondary sort by Children%) in src/parser.rs
- [x] T033 [US2] Integration test for `pperf top --self perf-report.txt` in tests/integration/top_command.rs

### Implementation for User Story 2

- [x] T034 [US2] Add --self/-s flag to CLI argument parsing in src/main.rs
- [x] T035 [US2] Update sort_entries() to accept SortOrder and implement Self sorting in src/parser.rs
- [x] T036 [US2] Wire --self flag to sort_entries() call in src/main.rs
- [x] T037 [US2] Verify all US2 tests pass with `cargo test`

**Checkpoint**: `pperf top --self perf-report.txt` works independently

---

## Phase 5: User Story 3 - Limit Number of Results (Priority: P2)

**Goal**: Add -n flag to control result count (default 10)

**Independent Test**: `pperf top -n 5 perf-report.txt` outputs exactly 5 functions

### Tests for User Story 3 (TDD - Write First)

- [x] T038 [P] [US3] Unit test for take_n() limiting results in src/parser.rs
- [x] T039 [P] [US3] Unit test for -n 0 returning InvalidCount error in src/main.rs (inline test)
- [x] T040 [US3] Integration test for `pperf top -n 5 perf-report.txt` in tests/integration/top_command.rs
- [x] T041 [US3] Integration test for `pperf top -n 0` error message in tests/integration/top_command.rs

### Implementation for User Story 3

- [x] T042 [US3] Add -n/--number flag to CLI argument parsing in src/main.rs
- [x] T043 [US3] Validate count > 0, return InvalidCount error if not in src/main.rs
- [x] T044 [US3] Update result pipeline to take N entries instead of hardcoded 10 in src/main.rs
- [x] T045 [US3] Implement exit code 3 for invalid arguments in src/main.rs
- [x] T046 [US3] Verify all US3 tests pass with `cargo test`

**Checkpoint**: `-n` flag works independently with US1 and US2

---

## Phase 6: User Story 4 - Filter by Function Names (Priority: P3)

**Goal**: Add --targets flag to filter functions by name prefix

**Independent Test**: `pperf top --targets DCT4D perf-report.txt` shows only matching functions

### Tests for User Story 4 (TDD - Write First)

- [x] T047 [P] [US4] Unit test for matches_prefix() exact match in src/filter.rs
- [x] T048 [P] [US4] Unit test for matches_prefix() prefix match in src/filter.rs
- [x] T049 [P] [US4] Unit test for filter_entries() with single target in src/filter.rs
- [x] T050 [P] [US4] Unit test for filter_entries() with multiple targets in src/filter.rs
- [x] T051 [P] [US4] Unit test for add_disambiguation_suffix() adding #1, #2 to duplicates in src/output.rs
- [x] T052 [US4] Integration test for `pperf top --targets DCT4D perf-report.txt` in tests/integration/top_command.rs
- [x] T053 [US4] Integration test for no matches returning exit code 4 in tests/integration/top_command.rs

### Implementation for User Story 4

- [x] T054 [US4] Implement matches_prefix() for exact and prefix matching in src/filter.rs
- [x] T055 [US4] Implement filter_entries() to apply targets filter to entry list in src/filter.rs
- [x] T056 [US4] Implement add_disambiguation_suffix() for duplicate base names in src/output.rs
- [x] T057 [US4] Add --targets/-t flag accepting multiple values to CLI in src/main.rs
- [x] T058 [US4] Wire filter into pipeline: parse → filter → sort → take N → format in src/main.rs
- [x] T059 [US4] Return NoMatches error with exit code 4 when filter yields empty results in src/main.rs
- [x] T060 [US4] Verify all US4 tests pass with `cargo test`

**Checkpoint**: All user stories complete and working together

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Quality checks and final validation

- [x] T061 Run `cargo fmt` and fix any formatting issues
- [x] T062 Run `cargo clippy` and address all warnings
- [x] T063 Run `cargo build --release` and verify no warnings
- [x] T064 Run full test suite `cargo test` and verify all pass
- [x] T065 Manual validation: run quickstart.md examples against real perf-report.txt
- [x] T066 Verify output matches expected format from data-model.md

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion
  - US1 (P1): Core functionality, no story dependencies
  - US2 (P2): Depends on US1 sort infrastructure
  - US3 (P2): Depends on US1 pipeline
  - US4 (P3): Depends on US1 pipeline, independent filter module
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational - Foundation for all other stories
- **User Story 2 (P2)**: Depends on US1 (uses same sort function with different order)
- **User Story 3 (P2)**: Depends on US1 (modifies take count in pipeline)
- **User Story 4 (P3)**: Depends on US1 (adds filter step to pipeline)

### Within Each User Story (TDD Cycle)

1. Write tests FIRST → verify they FAIL
2. Implement minimal code → verify tests PASS
3. Refactor if needed → verify tests still PASS
4. Commit

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel (T003, T004, T006)
- All Foundational tests marked [P] can run in parallel (T007, T008, T009)
- All US1 tests marked [P] can run in parallel (T015-T019)
- All US2 tests marked [P] can run in parallel (T031, T032)
- All US3 tests marked [P] can run in parallel (T038, T039)
- All US4 tests marked [P] can run in parallel (T047-T051)

---

## Parallel Example: User Story 1 Tests

```bash
# Launch all US1 unit tests in parallel:
Task: T015 "Unit test for parse_line() recognizing valid data lines"
Task: T016 "Unit test for parse_line() skipping comment lines"
Task: T017 "Unit test for parse_line() skipping call tree lines"
Task: T018 "Unit test for parse_file() extracting entries"
Task: T019 "Unit test for format_table() producing aligned output"

# Then run integration test (depends on unit tests):
Task: T020 "Integration test for pperf top perf-report.txt"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: `pperf top perf-report.txt` produces correct output
5. Commit/tag as v0.1.0-alpha

### Incremental Delivery

1. Setup + Foundational → Project compiles
2. US1 → Basic `top` command works (MVP!)
3. US2 → Add `--self` sorting
4. US3 → Add `-n` count control
5. US4 → Add `--targets` filtering
6. Polish → Production ready v0.1.0

### TDD Discipline (Per Constitution)

For EVERY task in US1-US4:
1. Write the test first
2. Run `cargo test` → test FAILS (RED)
3. Implement the feature
4. Run `cargo test` → test PASSES (GREEN)
5. Refactor if needed
6. Run `cargo test` → still PASSES
7. Commit

---

## Notes

- [P] tasks = different files, no dependencies on incomplete tasks
- [USn] label maps task to specific user story for traceability
- TDD is mandatory per constitution - tests MUST fail before implementation
- Real perf-report.txt from repository MUST be used for integration tests
- No external dependencies - standard library only
- Verify tests fail for the right reason before implementing
- Commit after each completed task or logical group
