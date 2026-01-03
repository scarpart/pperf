# Tasks: Clap CLI Refactor

**Input**: Design documents from `/specs/005-clap-cli-refactor/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md

**Tests**: Following TDD per constitution - existing tests verify behavior, no new test tasks needed as existing tests validate the refactor.

**Organization**: Tasks organized for sequential implementation with parallel opportunities where marked.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root
- Paths follow existing structure per plan.md

---

## Phase 1: Setup

**Purpose**: Add Clap dependency and prepare for refactor

- [x] T001 Add clap dependency with derive feature to Cargo.toml
- [x] T002 Verify project compiles with new dependency: `cargo build`

**Checkpoint**: Clap is available for use in main.rs

---

## Phase 2: Foundational (CLI Struct Definitions)

**Purpose**: Define Clap-derived CLI structures before refactoring main()

**⚠️ CRITICAL**: These structs must be defined before any parsing logic can be updated

- [x] T003 Define Cli root struct with Parser derive in src/main.rs
- [x] T004 Define Commands enum with Subcommand derive in src/main.rs
- [x] T005 Define TopArgs struct with all options in src/main.rs

**Checkpoint**: CLI structs defined, ready to wire into main()

---

## Phase 3: User Story 1 - Existing CLI Behavior Preserved (Priority: P1) 🎯 MVP

**Goal**: Replace manual argument parsing with Clap while maintaining identical behavior

**Independent Test**: Run existing tests - all must pass: `cargo test`

### Implementation for User Story 1

- [x] T006 [US1] Update main() to use Cli::try_parse() with exit code 3 for parse errors in src/main.rs
- [x] T007 [US1] Update run_top() signature to accept TopArgs instead of &[String] in src/main.rs
- [x] T008 [US1] Map TopArgs fields to existing local variables (sort_order, count, etc.) in run_top() in src/main.rs
- [x] T009 [US1] Add post-parse validation for --hierarchy requiring --targets in src/main.rs
- [x] T010 [US1] Remove old print_help() function from src/main.rs
- [x] T011 [US1] Remove manual argument parsing loop from run_top() in src/main.rs
- [x] T012 [US1] Run cargo test to verify all existing tests pass

**Checkpoint**: All existing functionality preserved, tests pass

---

## Phase 4: User Story 2 - Help and Version Information (Priority: P1)

**Goal**: Verify Clap-generated help and version work correctly

**Independent Test**: Run `pperf --help`, `pperf -h`, `pperf --version`, `pperf top --help`

### Implementation for User Story 2

- [x] T013 [US2] Verify pperf --help shows all subcommands and global options
- [x] T014 [US2] Verify pperf top --help shows all top subcommand options with descriptions
- [x] T015 [US2] Verify pperf --version outputs "pperf 0.1.0"

**Checkpoint**: Help and version output working

---

## Phase 5: User Story 3 - Error Handling (Priority: P1)

**Goal**: Verify error handling and exit codes are preserved

**Independent Test**: Test various error conditions and verify exit codes

### Implementation for User Story 3

- [x] T016 [US3] Verify pperf top nonexistent.txt returns exit code 1
- [x] T017 [US3] Verify pperf top -n 0 file.txt returns exit code 3 (Clap validates range)
- [x] T018 [US3] Verify pperf top --hierarchy file.txt (no targets) returns exit code 3
- [x] T019 [US3] Verify pperf unknown returns appropriate error

**Checkpoint**: All error conditions handled correctly with proper exit codes

---

## Phase 6: User Story 4 - Improved Argument Flexibility (Priority: P2)

**Goal**: Verify Clap's standard argument parsing features work

**Independent Test**: Test equals syntax and other Clap conveniences

### Implementation for User Story 4

- [x] T020 [US4] Verify pperf top -n=5 file.txt works (equals syntax)
- [x] T021 [US4] Verify pperf top --number=5 file.txt works (long form equals)

**Checkpoint**: Clap flexibility features working

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Final cleanup and documentation updates

- [x] T022 Run cargo clippy to check for any new warnings in src/main.rs
- [x] T023 Run cargo fmt to ensure consistent formatting
- [x] T024 Update CLAUDE.md if any CLI behavior notes need updating

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion
- **User Story 1 (Phase 3)**: Depends on Foundational - this is the core refactor
- **User Stories 2-4 (Phases 4-6)**: Verification tasks, depend on US1 completion
- **Polish (Phase 7)**: Depends on all user stories complete

### User Story Dependencies

- **User Story 1 (P1)**: Core refactor - must complete first
- **User Story 2 (P1)**: Verification only - depends on US1
- **User Story 3 (P1)**: Verification only - depends on US1
- **User Story 4 (P2)**: Verification only - depends on US1

### Within User Story 1

- T006 must complete before T007-T011 (main() must use Clap before run_top can change)
- T007 must complete before T008 (signature before implementation)
- T010 and T011 can run in parallel (independent removals)
- T012 must be last (final verification)

### Parallel Opportunities

- T003, T004, T005 are sequential (each builds on previous)
- T010, T011 can run [P] in parallel (different code sections)
- T013, T014, T015 can run [P] in parallel (independent verifications)
- T016, T017, T018, T019 can run [P] in parallel (independent verifications)
- T020, T021 can run [P] in parallel (independent verifications)
- T022, T023 can run [P] in parallel (different tools)

---

## Parallel Example: User Story 1 Removal Tasks

```bash
# These can run in parallel (different code sections):
Task: "T010 [US1] Remove old print_help() function from src/main.rs"
Task: "T011 [US1] Remove manual argument parsing loop from run_top() in src/main.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (add Clap dependency)
2. Complete Phase 2: Foundational (define CLI structs)
3. Complete Phase 3: User Story 1 (refactor main.rs)
4. **STOP and VALIDATE**: `cargo test` - all tests must pass
5. If tests fail, debug before proceeding

### Incremental Delivery

1. Setup + Foundational → Clap available
2. User Story 1 → Core refactor complete, tests pass (MVP!)
3. User Stories 2-4 → Verification of help, errors, flexibility
4. Polish → Cleanup and documentation

---

## Notes

- [P] tasks = different files or code sections, no dependencies
- [Story] label maps task to specific user story for traceability
- All verification tasks (T013-T021) are manual checks, not code changes
- Commit after completing each phase
- T012 is critical gate - do not proceed if tests fail
- Focus on US1 - it contains all the actual code changes
