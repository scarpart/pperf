# Feature Specification: Multi-Report Averaging

**Feature Branch**: `006-multi-report-averaging`
**Created**: 2026-01-05
**Status**: Draft
**Input**: User description: "Average profiling metrics across multiple perf report files from N executions, with debug mode showing per-report breakdowns"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Analyze Averaged Performance Metrics (Priority: P1)

A performance analyst wants to get statistically meaningful profiling data by averaging metrics across multiple execution runs of the same program, rather than relying on a single run which may have outlier behavior.

**Why this priority**: This is the core value proposition of the feature. Single-run profiling can be misleading due to system noise, caching effects, or transient conditions. Averaged data provides more reliable insights for optimization decisions.

**Independent Test**: Can be fully tested by providing multiple perf report files and verifying the output shows averaged percentages that correctly represent the arithmetic mean across all input files.

**Acceptance Scenarios**:

1. **Given** 3 perf report files from the same program, **When** user runs `pperf top file1.txt file2.txt file3.txt`, **Then** output displays averaged Children% and Self% for each function
2. **Given** functions with varying percentages across reports (e.g., function X: 73.86%, 73.60%, 70.40%), **When** analyzing these files together, **Then** output shows the arithmetic mean (72.62%)
3. **Given** a function exists in all N reports, **When** averaging, **Then** both Children% and Self% are independently averaged

---

### User Story 2 - Function Matching Across Reports (Priority: P1)

A developer needs functions to be correctly matched across different report files using their unique full signatures, even though simplified names are displayed in output.

**Why this priority**: Correct function identification is fundamental to accurate averaging. Without reliable matching, averaged data would be incorrect and potentially harmful for optimization decisions.

**Independent Test**: Can be tested by providing reports where the same function has different percentages and verifying the system correctly identifies and groups them by full signature for averaging.

**Acceptance Scenarios**:

1. **Given** reports containing function `TransformPartition::rd_optimize_transform(Block4D const&)` with different percentages, **When** averaging, **Then** the function is matched by its full signature across all reports
2. **Given** two functions with similar simplified names but different signatures, **When** averaging, **Then** they are treated as distinct functions
3. **Given** a function present in only some reports (not all), **When** averaging, **Then** the function is included using only the reports where it appears, with the count reflected appropriately

---

### User Story 3 - Debug Mode Per-Report Breakdown (Priority: P2)

A performance analyst using debug mode wants to see the individual percentages from each report file alongside the averaged values, to verify the averaging is correct and to understand variance across runs.

**Why this priority**: Provides transparency and verification capability. While not required for basic functionality, it significantly improves confidence in results and helps identify high-variance functions.

**Independent Test**: Can be tested by running with `--debug` flag and multiple files, verifying that each function's averaged percentages are accompanied by the individual per-report values.

**Acceptance Scenarios**:

1. **Given** 3 report files and `--debug` flag enabled, **When** displaying a function's metrics, **Then** show an additional line with individual values from each report (e.g., `(values: 73.86%, 73.60%, 70.40%)`)
2. **Given** hierarchy mode with debug enabled, **When** showing caller-callee relationships, **Then** per-report breakdowns appear for both direct and indirect call percentages
3. **Given** a function missing from one report, **When** debug mode displays values, **Then** indicate which reports contained the function (e.g., `(values: 73.86%, 73.60%, -)`)

---

### User Story 4 - Backward Compatibility with Single File (Priority: P2)

An existing user of pperf who provides only a single file should experience identical behavior to the current implementation.

**Why this priority**: Ensures existing workflows continue to work without modification. No learning curve for current users.

**Independent Test**: Can be tested by comparing output of single-file analysis before and after this feature, verifying identical results.

**Acceptance Scenarios**:

1. **Given** a single perf report file, **When** running `pperf top file.txt`, **Then** output is identical to current behavior (no averaging annotation)
2. **Given** single file with debug mode, **When** running analysis, **Then** debug annotations match current behavior without per-report breakdown

---

### Edge Cases

- What happens when provided files have no functions in common? → Display functions that appear in at least one report, with averaging applied only to reports containing each function
- What happens when a file cannot be parsed or is not a valid perf report? → Report error for the invalid file and do not proceed with analysis
- What happens when the same file is provided multiple times? → Treat as multiple separate inputs (counts in averaging); user is responsible for not duplicating
- What happens with only one file? → Behave identically to current single-file mode
- What happens when no files are provided? → Display usage error with help text
- How is the order of files handled? → Order does not affect averaging results; debug output shows values in file order provided

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST accept multiple perf report file paths as positional arguments
- **FR-002**: System MUST parse all provided files using the existing perf report format parser
- **FR-003**: System MUST identify functions by their full symbol signature for matching across reports
- **FR-004**: System MUST compute arithmetic mean of Children% across all reports containing each function
- **FR-005**: System MUST compute arithmetic mean of Self% across all reports containing each function
- **FR-006**: System MUST perform all existing calculations (hierarchy, relative percentages, standalone adjustments) on the averaged values
- **FR-007**: System MUST support existing flags (`--self`, `--number`, `--targets`, `--hierarchy`, `--debug`, `--no-color`) with multi-file input
- **FR-008**: When `--debug` flag is used with multiple files, system MUST display per-report percentage values on a separate annotation line
- **FR-009**: System MUST display simplified function names in output while using full signatures for internal matching
- **FR-010**: System MUST fail with a descriptive error if any provided file cannot be parsed
- **FR-011**: System MUST maintain backward compatibility: single-file input produces identical output to current behavior

### Key Entities

- **PerfEntry**: Represents a function's profiling data; extended conceptually to hold averaged percentages with optional per-report breakdown metadata
- **Report Set**: Collection of parsed reports to be averaged; functions matched by full symbol signature
- **Averaged Entry**: A function's metrics after averaging across all reports where it appears

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can analyze up to 10 report files in a single command without noticeable delay beyond file I/O
- **SC-002**: Averaged percentages are mathematically correct (verifiable by manual calculation on known test data)
- **SC-003**: 100% of existing single-file workflows produce identical output after this feature is implemented
- **SC-004**: Debug mode clearly displays per-report values in a readable format that users can manually verify
- **SC-005**: Functions are correctly matched across reports 100% of the time when using identical signatures
- **SC-006**: Users can identify high-variance functions by examining debug mode per-report breakdowns
