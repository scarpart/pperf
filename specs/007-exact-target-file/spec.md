# Feature Specification: Exact Function Signature Target File

**Feature Branch**: `007-exact-target-file`
**Created**: 2026-01-08
**Status**: Draft
**Input**: Replace substring-based target matching with exact function signature matching from a target file to eliminate ambiguity in hierarchy analysis.

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Provide Target Functions via File (Priority: P1)

Users want to analyze specific functions in their perf report by providing a file containing exact function signatures, one per line. This eliminates the current ambiguity where `-t get_rd_for_below` could match both `get_rd_for_below_inferior_bit_plane` and `get_rd_for_below_superior_bit_plane`.

**Why this priority**: This is the core feature that enables unambiguous target selection. Without this, all other ambiguity detection is moot.

**Independent Test**: Can be fully tested by creating a targets file with exact function signatures and verifying that only those specific functions are matched in the output.

**Acceptance Scenarios**:

1. **Given** a perf report file and a targets file containing exact function signatures, **When** the user runs `pperf top --target-file targets.txt perf-report.txt`, **Then** only functions with exact signature matches are included in the analysis.

2. **Given** a targets file with the signature `Hierarchical4DEncoder::get_rd_for_below_inferior_bit_plane(LightfieldCoordinate<unsigned int> const&, LightfieldDimension<unsigned int, true> const&)`, **When** the user runs the command, **Then** only that exact function is matched, not `get_rd_for_below_superior_bit_plane` or any other similarly-named function.

3. **Given** a targets file with empty lines or comment lines (starting with #), **When** the user runs the command, **Then** empty lines and comments are ignored during parsing.

---

### User Story 2 - Detect Ambiguous Function Signatures (Priority: P1)

When users provide a function signature that matches multiple entries in the perf report, the tool must detect this and report an error, showing which entries matched and asking for a more specific signature.

**Why this priority**: Equally critical as the first story - ambiguity detection is the safety net that ensures users get correct analysis results.

**Independent Test**: Can be tested by providing a partial/ambiguous signature and verifying the error message lists all matching entries.

**Acceptance Scenarios**:

1. **Given** a targets file containing partial signature `get_rd_for_below`, **When** the user runs the command and this matches multiple functions in the perf report, **Then** the tool displays an error listing all matched functions and exits with a non-zero status.

2. **Given** a targets file containing `DCT4DBlock`, **When** this matches entries like `DCT4DBlock::DCT4DBlock(Block4D const&, double)` and other DCT4DBlock methods, **Then** the tool shows all matches and instructs the user to use the complete signature.

3. **Given** a targets file where each signature matches exactly one function, **When** the user runs the command, **Then** no ambiguity error is raised and analysis proceeds normally.

---

### User Story 3 - Backward Compatibility with Existing -t Flag (Priority: P2)

Users who prefer the current substring-based matching for quick exploration can still use the existing `-t` / `--targets` flag. The new `--target-file` flag is an alternative, not a replacement.

**Why this priority**: Important for maintaining existing workflows, but the new file-based approach is preferred for precise analysis.

**Independent Test**: Can be tested by running existing commands with `-t` flag and verifying behavior is unchanged.

**Acceptance Scenarios**:

1. **Given** a user running `pperf top -t DCT4D perf-report.txt`, **When** executing the command, **Then** substring matching works exactly as it does today (matches any function containing "DCT4D").

2. **Given** both `--target-file` and `-t` flags provided, **When** the user runs the command, **Then** the tool displays an error indicating these flags are mutually exclusive.

---

### User Story 4 - Helpful Error Messages for Missing Targets (Priority: P2)

When a function signature in the targets file doesn't match any entry in the perf report, users receive a clear error message identifying the unmatched signature.

**Why this priority**: Good UX for helping users debug why their targets aren't being found.

**Independent Test**: Can be tested by providing a targets file with a non-existent function signature.

**Acceptance Scenarios**:

1. **Given** a targets file containing a signature that doesn't exist in the perf report, **When** the user runs the command, **Then** the tool displays an error identifying the unmatched signature(s).

2. **Given** a targets file with multiple signatures where some exist and some don't, **When** the user runs the command, **Then** the tool reports all unmatched signatures before exiting.

---

### Edge Cases

- What happens when the targets file doesn't exist or is unreadable?
  - The tool displays a file-not-found error with the path.

- What happens when the targets file is empty (no valid signatures after filtering comments/blank lines)?
  - The tool displays an error indicating no valid targets were found in the file.

- How does the tool handle function signatures with special characters (templates, operators)?
  - Function signatures are matched exactly as written, including template parameters like `<unsigned int>`, `const&`, and operator overloads.

- What happens when a signature has trailing/leading whitespace?
  - Leading and trailing whitespace is trimmed from each line before matching.

- How does symbol simplification interact with exact matching?
  - When using `--target-file`, matching is performed against the **raw** (unsimplified) symbols from the perf report, since users specify exact signatures. The simplified symbols are still used for display.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST accept a `--target-file <path>` CLI argument that specifies a file containing function signatures, one per line.

- **FR-002**: System MUST parse the targets file, ignoring empty lines and lines starting with `#` (comments).

- **FR-003**: System MUST trim leading and trailing whitespace from each function signature line.

- **FR-004**: System MUST perform exact string matching between each target signature and the raw function symbols in the perf report.

- **FR-005**: System MUST detect when a single target signature matches multiple entries in the perf report (ambiguity).

- **FR-006**: System MUST display an error when ambiguity is detected, listing all matching entries and the problematic signature.

- **FR-007**: System MUST exit with a distinct non-zero exit code when ambiguity is detected.

- **FR-008**: System MUST display an error when a target signature matches zero entries, listing all unmatched signatures.

- **FR-009**: System MUST reject commands that specify both `--target-file` and `-t`/`--targets`, displaying a mutual exclusivity error.

- **FR-010**: System MUST preserve the existing `-t`/`--targets` substring matching behavior unchanged for backward compatibility.

- **FR-011**: System MUST validate that the targets file exists and is readable before processing.

- **FR-012**: System MUST require that `--hierarchy` with `--target-file` produces one and only one standalone/caller branch per target signature.

### Key Entities

- **TargetFile**: A text file containing one function signature per line, with support for comments and blank lines.

- **FunctionSignature**: An exact function signature string as it appears in the perf report (e.g., `Hierarchical4DEncoder::get_rd_for_below_inferior_bit_plane(LightfieldCoordinate<unsigned int> const&, LightfieldDimension<unsigned int, true> const&)`).

- **AmbiguityError**: An error type that contains the ambiguous signature and the list of all matching perf entries.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can successfully analyze perf reports with exact function targeting, eliminating all false positive matches from substring collisions.

- **SC-002**: 100% of ambiguous target signatures are detected and reported before analysis proceeds.

- **SC-003**: Users can create and maintain target files for repeated analysis of the same functions across different perf runs.

- **SC-004**: Error messages for ambiguity include sufficient detail (signature provided, all matches found) for users to correct their targets file.

- **SC-005**: Existing users of the `-t` flag experience no change in behavior or output format.

## Assumptions

- Function signatures in perf reports are stable across runs of the same binary (same compilation).
- Users can obtain exact function signatures from an initial `pperf top` run without targets, or from other tools like `nm` or the perf report itself.
- The perf report format for function symbols remains consistent with the current expected format.
- Template parameters and const qualifiers in C++ signatures are meaningful differentiators between functions.
