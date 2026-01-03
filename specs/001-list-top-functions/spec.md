# Feature Specification: List Top Functions

**Feature Branch**: `001-list-top-functions`
**Created**: 2026-01-02
**Status**: Draft
**Input**: User description: "CLI subcommand to list most time-consuming functions from perf report, with Children/Self ordering and function filtering"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - View Top Functions by Children Time (Priority: P1)

A developer runs pperf with a perf report file to quickly see which functions consume the most CPU time (inclusive of callees). The output shows a table of the top 10 functions sorted by Children percentage.

**Why this priority**: This is the core use case - identifying optimization targets requires seeing which functions dominate execution time.

**Independent Test**: Can be fully tested by running `pperf top <perf-report.txt>` and verifying the output shows a sorted table of functions with correct percentages.

**Acceptance Scenarios**:

1. **Given** a valid perf report file, **When** running `pperf top report.txt`, **Then** output displays the top 10 functions sorted by Children percentage in descending order
2. **Given** a valid perf report file, **When** running `pperf top report.txt`, **Then** each row shows: Children%, Self%, and function name
3. **Given** a valid perf report file with fewer than 10 unique functions, **When** running `pperf top report.txt`, **Then** all functions are displayed without error

---

### User Story 2 - View Top Functions by Self Time (Priority: P2)

A developer wants to identify functions that consume CPU cycles directly (excluding callees) to find optimization hotspots within function bodies.

**Why this priority**: Self time reveals where actual computation happens, complementing the Children view.

**Independent Test**: Can be tested by running `pperf top --self report.txt` and verifying sorting is by Self percentage.

**Acceptance Scenarios**:

1. **Given** a valid perf report file, **When** running `pperf top --self report.txt`, **Then** output displays functions sorted by Self percentage in descending order
2. **Given** functions with identical Self percentages, **When** sorted by Self time, **Then** secondary sort is by Children percentage

---

### User Story 3 - Limit Number of Results (Priority: P2)

A developer wants to control how many functions appear in the output, either expanding to see more or limiting to focus on critical ones.

**Why this priority**: Flexibility in output size improves usability across different analysis needs.

**Independent Test**: Can be tested by running `pperf top -n 5 report.txt` and verifying exactly 5 functions appear.

**Acceptance Scenarios**:

1. **Given** a valid perf report file, **When** running `pperf top -n 5 report.txt`, **Then** exactly 5 functions are displayed
2. **Given** a valid perf report with 3 functions, **When** running `pperf top -n 10 report.txt`, **Then** all 3 functions are displayed without error
3. **Given** an invalid count value, **When** running `pperf top -n 0 report.txt`, **Then** an error message is displayed

---

### User Story 4 - Filter by Function Names (Priority: P3)

A developer wants to see statistics only for specific functions they are interested in, using full names or prefixes.

**Why this priority**: Targeted analysis of known functions speeds up performance investigation.

**Independent Test**: Can be tested by running `pperf top --targets DCT4DBlock report.txt` and verifying only matching functions appear.

**Acceptance Scenarios**:

1. **Given** a valid perf report file, **When** running `pperf top --targets "DCT4DBlock::DCT4DBlock" report.txt`, **Then** only exact matches are displayed
2. **Given** a valid perf report file, **When** running `pperf top --targets DCT4D report.txt`, **Then** all functions with names starting with "DCT4D" are displayed
3. **Given** multiple targets, **When** running `pperf top --targets DCT4D Hierarchical report.txt`, **Then** functions matching any target prefix are displayed
4. **Given** a prefix matching multiple function signatures, **When** filtering, **Then** each matching signature is shown with a distinguishing suffix (e.g., `#1`, `#2`)
5. **Given** a target with no matches, **When** running `pperf top --targets nonexistent report.txt`, **Then** output shows "No matching functions found"

---

### Edge Cases

- What happens when the perf report file does not exist?
  - Display error: "File not found: <path>"
- What happens when the perf report format is invalid or corrupted?
  - Display error: "Invalid perf report format" with line number if available
- What happens when a function name contains special characters (templates, lambdas)?
  - Parse and display the full symbol name correctly
- How are unresolved symbols (hex addresses like `0x7d4c47223efe`) handled?
  - Include them in output, displayed as-is
- What happens when `-n` receives a non-numeric value?
  - Display error: "Invalid value for -n: expected positive integer"

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST parse the standard `perf report` text format including header comments and data lines
- **FR-002**: System MUST extract Children%, Self%, and Symbol columns from each function entry
- **FR-003**: System MUST sort results by Children% by default, descending
- **FR-004**: System MUST support `--self` flag to sort by Self% instead
- **FR-005**: System MUST default to displaying 10 results
- **FR-006**: System MUST support `-n <number>` flag to change result count (positive integers only)
- **FR-007**: System MUST support `--targets <name>...` to filter functions by name prefix
- **FR-008**: System MUST match targets against function names using prefix matching
- **FR-009**: System MUST display all matches when a prefix matches multiple signatures
- **FR-010**: System MUST distinguish functions with identical base names by appending a numeric suffix (e.g., `#1`, `#2`)
- **FR-011**: System MUST output results in a tabular format similar to `ls -l` (aligned columns, no borders)
- **FR-012**: System MUST read perf report from a file path argument
- **FR-013**: System MUST provide clear error messages for file not found, invalid format, and invalid arguments

### Key Entities

- **PerfEntry**: A single function's profiling data containing Children%, Self%, Command, Shared Object, and Symbol
- **PerfReport**: A collection of PerfEntries parsed from a perf report file
- **FunctionFilter**: A mechanism to match function names by exact match or prefix

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: User can identify the top 10 time-consuming functions within 1 second of running the command on a typical perf report
- **SC-002**: Parsed percentages match the values in the source perf report file exactly
- **SC-003**: Output table is readable without horizontal scrolling on a standard 120-column terminal
- **SC-004**: All acceptance scenarios pass when validated against the sample perf-report.txt in the repository

## Assumptions

- Input files are text output from `perf report` command, not binary `perf.data` files
- Function names may contain C++ template syntax, lambdas, and clone markers (e.g., `[clone .isra.0]`)
- The perf report uses the default column order: Children, Self, Command, Shared Object, Symbol
- Percentage values are always formatted as `XX.XX%` with two decimal places
