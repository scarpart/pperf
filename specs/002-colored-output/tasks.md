# Tasks: Colored Output with Simplified Function Names

**Input**: Design documents from `/specs/002-colored-output/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/cli.md

**Tests**: REQUIRED per constitution (TDD is NON-NEGOTIABLE)

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root
- Rust crate structure per plan.md

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: New module creation and ANSI color constants

- [ ] T001 Create src/symbol.rs with module declaration and ANSI color constants (BLUE, YELLOW, RED, RESET)
- [ ] T002 Add `pub mod symbol;` declaration in src/lib.rs

**Checkpoint**: New module compiles with `cargo build`

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core types and color detection that ALL user stories depend on

**CRITICAL**: No user story work can begin until this phase is complete

### Tests for Foundational

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T003 [P] Unit test for SymbolType enum variants (User, Library, Unresolved) in src/symbol.rs
- [ ] T004 [P] Unit test for should_use_color() with TTY detection mock in src/symbol.rs
- [ ] T005 [P] Unit test for should_use_color() with NO_COLOR env var in src/symbol.rs

### Implementation for Foundational

- [ ] T006 Define SymbolType enum (User, Library, Unresolved) in src/symbol.rs
- [ ] T007 Implement should_use_color(no_color_flag: bool) -> bool using std::io::IsTerminal in src/symbol.rs
- [ ] T008 Implement color_for_type(symbol_type: SymbolType) -> &'static str returning ANSI codes in src/symbol.rs
- [ ] T009 Verify all foundational tests pass with `cargo test`

**Checkpoint**: Foundation ready - user story implementation can now begin

---

## Phase 3: User Story 1 - Color-Coded Function Types (Priority: P1)

**Goal**: Display functions in different colors based on their type (user=blue, library=yellow, unresolved=red)

**Independent Test**: Run `pperf top perf-report.txt` in terminal and verify colored output

### Tests for User Story 1 (TDD - Write First)

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T010 [P] [US1] Unit test for classify_symbol() with hex addresses returning Unresolved in src/symbol.rs
- [ ] T011 [P] [US1] Unit test for classify_symbol() with std:: prefix returning Library in src/symbol.rs
- [ ] T012 [P] [US1] Unit test for classify_symbol() with __ prefix returning Library in src/symbol.rs
- [ ] T013 [P] [US1] Unit test for classify_symbol() with libc functions (malloc, free, memset) returning Library in src/symbol.rs
- [ ] T014 [P] [US1] Unit test for classify_symbol() with user functions returning User in src/symbol.rs
- [ ] T015 [P] [US1] Unit test for format_colored_symbol() producing correct ANSI codes in src/symbol.rs
- [ ] T016 [US1] Integration test for colored output in terminal (check ANSI codes present) in tests/top_command.rs

### Implementation for User Story 1

- [ ] T017 [US1] Implement is_hex_address() to detect 0x... and all-hex patterns in src/symbol.rs
- [ ] T018 [US1] Implement is_library_symbol() with std::, __, libc function patterns in src/symbol.rs
- [ ] T019 [US1] Implement classify_symbol(symbol: &str) -> SymbolType in src/symbol.rs
- [ ] T020 [US1] Implement format_colored_symbol(symbol: &str, use_color: bool) -> String in src/symbol.rs
- [ ] T021 [US1] Modify format_table() in src/output.rs to accept use_color parameter
- [ ] T022 [US1] Update format_table() to apply colors to each entry's symbol in src/output.rs
- [ ] T023 [US1] Update run_top() in src/main.rs to compute use_color and pass to format_table()
- [ ] T024 [US1] Verify all US1 tests pass with `cargo test`

**Checkpoint**: `pperf top perf-report.txt` shows colored output in terminal

---

## Phase 4: User Story 2 - Simplified Function Names (Priority: P1)

**Goal**: Strip return types, arguments, and template parameters to show only `Namespace::Function`

**Independent Test**: Run `pperf top perf-report.txt` and verify clean function names without clutter

### Tests for User Story 2 (TDD - Write First)

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T025 [P] [US2] Unit test for simplify_symbol() stripping return types in src/symbol.rs
- [ ] T026 [P] [US2] Unit test for simplify_symbol() stripping argument lists in src/symbol.rs
- [ ] T027 [P] [US2] Unit test for simplify_symbol() stripping template parameters in src/symbol.rs
- [ ] T028 [P] [US2] Unit test for simplify_symbol() stripping nested templates in src/symbol.rs
- [ ] T029 [P] [US2] Unit test for simplify_symbol() stripping clone suffixes (.cold, .part.N) in src/symbol.rs
- [ ] T030 [P] [US2] Unit test for simplify_symbol() collapsing lambda syntax in src/symbol.rs
- [ ] T031 [P] [US2] Unit test for simplify_symbol() preserving hex addresses unchanged in src/symbol.rs
- [ ] T032 [US2] Unit test for simplify_symbol() with real symbols from perf-report.txt in src/symbol.rs

### Implementation for User Story 2

- [ ] T033 [US2] Implement strip_return_type() removing leading type before function name in src/symbol.rs
- [ ] T034 [US2] Implement strip_template_params() with bracket counting for nested templates in src/symbol.rs
- [ ] T035 [US2] Implement strip_arguments() with parenthesis counting for nested args in src/symbol.rs
- [ ] T036 [US2] Implement strip_clone_suffix() removing .cold, .part.N, .isra.N patterns in src/symbol.rs
- [ ] T037 [US2] Implement collapse_lambda() converting {lambda...} to {lambda} in src/symbol.rs
- [ ] T038 [US2] Implement simplify_symbol(symbol: &str) -> String combining all strip functions in src/symbol.rs
- [ ] T039 [US2] Update format_colored_symbol() to call simplify_symbol() before applying color in src/symbol.rs
- [ ] T040 [US2] Verify all US2 tests pass with `cargo test`

**Checkpoint**: `pperf top perf-report.txt` shows clean `Namespace::Function` names

---

## Phase 5: User Story 3 - Disable Colors When Needed (Priority: P2)

**Goal**: Disable colors when piped, with --no-color flag, or with NO_COLOR env var

**Independent Test**: Run `pperf top --no-color perf-report.txt` and verify no ANSI codes in output

### Tests for User Story 3 (TDD - Write First)

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T041 [P] [US3] Unit test for --no-color flag parsing in src/main.rs (inline test)
- [ ] T042 [P] [US3] Integration test for `pperf top --no-color perf-report.txt` having no ANSI codes in tests/top_command.rs
- [ ] T043 [P] [US3] Integration test for piped output having no ANSI codes in tests/top_command.rs

### Implementation for User Story 3

- [ ] T044 [US3] Add --no-color flag parsing to run_top() argument handling in src/main.rs
- [ ] T045 [US3] Update should_use_color() to accept no_color_flag parameter in src/symbol.rs
- [ ] T046 [US3] Wire --no-color flag into use_color computation in src/main.rs
- [ ] T047 [US3] Update help text to document --no-color flag in src/main.rs
- [ ] T048 [US3] Verify all US3 tests pass with `cargo test`

**Checkpoint**: Colors disabled correctly via flag, env var, and pipe detection

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Quality checks and final validation

- [ ] T049 Run `cargo fmt` and fix any formatting issues
- [ ] T050 Run `cargo clippy` and address all warnings
- [ ] T051 Run `cargo build --release` and verify no warnings
- [ ] T052 Run full test suite `cargo test` and verify all pass
- [ ] T053 Manual validation: run quickstart.md examples against real perf-report.txt
- [ ] T054 Verify color output visually in terminal
- [ ] T055 Verify piped output has no ANSI escape sequences

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-5)**: All depend on Foundational phase completion
  - US1 (P1): Core coloring, no story dependencies
  - US2 (P1): Symbol simplification, depends on US1 format_colored_symbol integration
  - US3 (P2): Color control, depends on US1 color infrastructure
- **Polish (Phase 6)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational - Core color classification
- **User Story 2 (P1)**: Depends on US1 (uses format_colored_symbol as integration point)
- **User Story 3 (P2)**: Depends on US1 (adds control over color output)

### Within Each User Story (TDD Cycle)

1. Write tests FIRST → verify they FAIL
2. Implement minimal code → verify tests PASS
3. Refactor if needed → verify tests still PASS
4. Commit

### Parallel Opportunities

- All Foundational tests marked [P] can run in parallel (T003, T004, T005)
- All US1 tests marked [P] can run in parallel (T010-T015)
- All US2 tests marked [P] can run in parallel (T025-T031)
- All US3 tests marked [P] can run in parallel (T041-T043)

---

## Parallel Example: User Story 2 Tests

```bash
# Launch all US2 unit tests in parallel:
Task: T025 "Unit test for simplify_symbol() stripping return types"
Task: T026 "Unit test for simplify_symbol() stripping argument lists"
Task: T027 "Unit test for simplify_symbol() stripping template parameters"
Task: T028 "Unit test for simplify_symbol() stripping nested templates"
Task: T029 "Unit test for simplify_symbol() stripping clone suffixes"
Task: T030 "Unit test for simplify_symbol() collapsing lambda syntax"
Task: T031 "Unit test for simplify_symbol() preserving hex addresses"

# Then run integration test (depends on unit tests):
Task: T032 "Unit test for simplify_symbol() with real symbols from perf-report.txt"
```

---

## Implementation Strategy

### MVP First (User Story 1 + 2 Combined)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational
3. Complete Phase 3: User Story 1 (colored output)
4. Complete Phase 4: User Story 2 (simplified names)
5. **STOP and VALIDATE**: Output should show clean, colored function names
6. Commit/tag as v0.2.0-alpha

### Incremental Delivery

1. Setup + Foundational → New module compiles
2. US1 → Color-coded output works (visible improvement!)
3. US2 → Simplified names (major readability boost!)
4. US3 → Color control (polish)
5. Polish → Production ready v0.2.0

### TDD Discipline (Per Constitution)

For EVERY task in US1-US3:
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
- ANSI codes: `\x1b[34m` (blue), `\x1b[33m` (yellow), `\x1b[31m` (red), `\x1b[0m` (reset)
- Verify tests fail for the right reason before implementing
- Commit after each completed task or logical group
