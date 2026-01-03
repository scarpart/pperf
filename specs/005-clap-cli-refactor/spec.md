# Feature Specification: Clap CLI Refactor

**Feature Branch**: `005-clap-cli-refactor`
**Created**: 2026-01-03
**Status**: Draft
**Input**: User description: "Replace ad-hoc CLI argument parsing with Clap for more robust command-line handling without changing functionality"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Existing CLI Behavior Preserved (Priority: P1)

Users continue to use pperf exactly as before with all existing command-line options functioning identically.

**Why this priority**: This is the core requirement - a refactoring should not break any existing functionality. Users must not notice any change in behavior.

**Independent Test**: Can be fully tested by running all existing command combinations and verifying identical output and behavior.

**Acceptance Scenarios**:

1. **Given** a user runs `pperf top perf-report.txt`, **When** the command executes, **Then** the output is identical to the current implementation
2. **Given** a user runs `pperf top --self perf-report.txt`, **When** the command executes, **Then** entries are sorted by Self% as before
3. **Given** a user runs `pperf top -n 5 perf-report.txt`, **When** the command executes, **Then** only 5 entries are shown
4. **Given** a user runs `pperf top --targets func1 func2 file.txt`, **When** the command executes, **Then** only functions matching "func1" or "func2" are shown
5. **Given** a user runs `pperf top --hierarchy --targets func1 func2 file.txt`, **When** the command executes, **Then** hierarchy view is displayed
6. **Given** a user runs `pperf top --hierarchy --debug --targets func1 file.txt`, **When** the command executes, **Then** debug annotations are shown
7. **Given** a user runs `pperf top --no-color file.txt`, **When** the command executes, **Then** output has no ANSI color codes

---

### User Story 2 - Help and Version Information (Priority: P1)

Users can access help and version information using standard CLI conventions.

**Why this priority**: Help is critical for discoverability and must work identically.

**Independent Test**: Can be tested by running `--help` and `--version` and verifying output format.

**Acceptance Scenarios**:

1. **Given** a user runs `pperf --help`, **When** the command executes, **Then** help text is displayed showing all options
2. **Given** a user runs `pperf -h`, **When** the command executes, **Then** help text is displayed (short form)
3. **Given** a user runs `pperf --version`, **When** the command executes, **Then** version "pperf 0.1.0" is displayed
4. **Given** a user runs `pperf top --help`, **When** the command executes, **Then** help for the top subcommand is displayed

---

### User Story 3 - Error Handling (Priority: P1)

Users receive appropriate error messages and exit codes for invalid input.

**Why this priority**: Error behavior is part of the CLI contract and must be preserved.

**Independent Test**: Can be tested by providing invalid inputs and verifying error messages and exit codes.

**Acceptance Scenarios**:

1. **Given** a user runs `pperf top nonexistent.txt`, **When** the file doesn't exist, **Then** error message is shown and exit code is 1
2. **Given** a user runs `pperf top -n 0 file.txt`, **When** count is invalid, **Then** error message is shown and exit code is 3
3. **Given** a user runs `pperf top -n abc file.txt`, **When** count is not a number, **Then** error message is shown and exit code is 3
4. **Given** a user runs `pperf top --hierarchy file.txt`, **When** no targets are specified, **Then** error "hierarchy requires targets" is shown and exit code is 3
5. **Given** a user runs `pperf unknown`, **When** subcommand is invalid, **Then** appropriate error is shown

---

### User Story 4 - Improved Argument Flexibility (Priority: P2)

Users benefit from Clap's standard argument parsing behavior (combined short flags, equals syntax, etc.).

**Why this priority**: This is a nice-to-have improvement that comes naturally with Clap adoption.

**Independent Test**: Can be tested by trying various argument syntaxes supported by Clap.

**Acceptance Scenarios**:

1. **Given** a user runs `pperf top -n=5 file.txt`, **When** using equals syntax, **Then** it works correctly
2. **Given** a user runs `pperf top --number=5 file.txt`, **When** using long form with equals, **Then** it works correctly

---

### Edge Cases

- What happens when no arguments are provided? → Usage message displayed, appropriate exit code
- What happens with unknown options like `--foo`? → Error message with exit code
- What happens when `--targets` is last without values? → File path is required, error message shown
- What happens when file path contains spaces? → Path handled correctly
- What happens with `--debug` without `--hierarchy`? → Flag is accepted but has no effect (current behavior preserved)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST parse all existing CLI options identically: `--self`/`-s`, `-n`/`--number`, `--targets`/`-t`, `--hierarchy`/`-H`, `--debug`/`-D`, `--no-color`
- **FR-002**: System MUST support the `top` subcommand as the only current subcommand
- **FR-003**: System MUST display help via `--help`/`-h` at both top-level and subcommand level
- **FR-004**: System MUST display version via `--version`
- **FR-005**: System MUST preserve all existing exit codes: 1 (file not found), 2 (invalid format), 3 (invalid arguments), 4 (no matches)
- **FR-006**: System MUST accept `--targets` with multiple space-separated values followed by a file path
- **FR-007**: System MUST detect that a targets argument is actually a file path and treat it as the positional file argument
- **FR-008**: System MUST use the Clap crate for all argument parsing
- **FR-009**: System MUST preserve current error message content for domain errors (file not found, no matches, hierarchy requires targets)

### Key Entities

- **CLI Arguments**: Command-line options, flags, and positional arguments parsed by Clap
- **Subcommand**: Currently only `top`, structured for future subcommand additions

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All existing test cases pass without modification
- **SC-002**: All command-line examples from CLAUDE.md Quick Reference section work identically
- **SC-003**: Help output includes all options with descriptions
- **SC-004**: Exit codes remain unchanged for all error conditions
- **SC-005**: main.rs argument parsing code is replaced with Clap declarative definitions

## Assumptions

- Clap will be added as a dependency (this is the first external dependency, breaking the "standard library only" constraint mentioned in CLAUDE.md)
- Clap derive feature will be used for cleaner declarative syntax
- Minor differences in help text formatting (Clap-generated vs hand-written) are acceptable as long as all options are documented
- Minor differences in error message wording for Clap-generated argument parsing errors are acceptable
- The core domain error messages (file not found, no matches, etc.) remain unchanged
