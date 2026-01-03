# Feature Specification: Debug Calculation Path

**Feature Branch**: `004-debug-calculation-path`
**Created**: 2026-01-03
**Status**: Draft
**Input**: Add --debug flag to show percentage calculation breakdown for hierarchy output

## Overview

When using `--hierarchy` mode, percentages shown for nested callees may skip intermediate functions that are not in the target list. This feature adds a `--debug` flag that displays the calculation path for each entry, showing how the final percentage was derived from the chain of intermediary function percentages.

This transparency allows users to:
- Validate that percentage calculations are correct
- Understand the call path even when intermediaries are omitted
- Trust the tool's output by seeing the underlying math

## Clarifications

### Session 2026-01-03

- Q: Annotation placement (same line or separate line below)? → A: Separate line below
- Q: Should direct calls show annotation for consistency? → A: Yes, direct calls also show annotation
- Q: Intermediary name format (full or simplified)? → A: Simplified names, but all intermediaries shown

## User Scenarios & Testing

### User Story 1 - View Calculation Breakdown for Indirect Calls (Priority: P1)

A developer analyzing performance data wants to understand how a nested callee's percentage was calculated when the call path goes through non-target intermediary functions.

**Why this priority**: Core value proposition - transparency into calculations that skip intermediaries.

**Independent Test**: Run `pperf top --hierarchy --debug -t target1 target2 file` and verify indirect calls show the multiplication chain on a separate gray line below the entry.

**Acceptance Scenarios**:

1. **Given** a hierarchy where target A calls intermediary X which calls target B, **When** running with `--hierarchy --debug`, **Then** target B's entry shows a gray line below with the format: `(via X 50.00% × 30.00% = 15.00%)`

2. **Given** a hierarchy where target A calls X → Y → Z → target B (multiple intermediaries), **When** running with `--hierarchy --debug`, **Then** target B's entry shows all intermediaries: `(via X 50.00% × Y 40.00% × Z 20.00% × 10.00% = 0.40%)`

---

### User Story 2 - View Confirmation for Direct Calls (Priority: P2)

A developer wants consistent formatting across all entries, including direct calls that don't skip any intermediaries.

**Why this priority**: Ensures visual consistency and confirms when no calculation adjustment occurred.

**Independent Test**: Run with `--debug` and verify direct caller→callee relationships show `(direct: X%)` on a separate gray line.

**Acceptance Scenarios**:

1. **Given** a hierarchy where target A directly calls target B with no intermediaries, **When** running with `--hierarchy --debug`, **Then** target B's entry shows a gray line below: `(direct: 17.23%)`

2. **Given** mixed direct and indirect calls in the same output, **When** running with `--debug`, **Then** each entry has exactly one gray annotation line (either "direct" or "via" format)

---

### User Story 3 - View Calculation Breakdown for Standalone Entries (Priority: P1)

A developer wants to understand how a standalone entry's adjusted percentage was calculated, showing the original percentage minus the contributions from being called by other target functions.

**Why this priority**: Core value proposition - transparency into why standalone entries have different percentages than their original Children%.

**Independent Test**: Run `pperf top --hierarchy --debug -t rd_optimize DCT4DBlock file` and verify standalone DCT4DBlock shows subtraction breakdown.

**Acceptance Scenarios**:

1. **Given** a hierarchy where target B is called by target A (contributing 12.37% of B's time), **When** running with `--hierarchy --debug`, **Then** B's standalone entry shows a gray line below with format: `(standalone: 38.00% - 12.37% (rd_optimize_transform) = 25.63%)`

2. **Given** a target that is called by multiple other targets, **When** running with `--hierarchy --debug`, **Then** the standalone entry shows all caller contributions: `(standalone: 50.00% - 20.00% (CallerA) - 15.00% (CallerB) = 15.00%)`

3. **Given** a target that is NOT called by any other targets, **When** running with `--hierarchy --debug`, **Then** the entry shows: `(standalone: 38.00% - (no callers) = 38.00%)` OR no annotation (since original = adjusted)

---

### User Story 4 - Debug Flag Has No Effect Without Hierarchy (Priority: P3)

The `--debug` flag only applies to hierarchy mode; without `--hierarchy`, it has no visible effect.

**Why this priority**: Defines scope boundary - prevents confusion about when debug output appears.

**Independent Test**: Run `pperf top --debug file` (without --hierarchy) and verify output is unchanged from normal mode.

**Acceptance Scenarios**:

1. **Given** a perf report file, **When** running `pperf top --debug file` without `--hierarchy`, **Then** output is identical to running without `--debug`

---

### Edge Cases

- What happens when a recursive target appears in the intermediary chain? Show simplified name; recursion doesn't break the calculation display.
- What happens when the calculation involves many intermediaries (>5)? Show all intermediaries; let the line wrap naturally.
- What happens with 0% intermediaries? Show the 0.00% value; the final result will be 0.00%.
- What happens when `--no-color` is combined with `--debug`? Show the annotation line without gray styling (plain text).

## Requirements

### Functional Requirements

- **FR-001**: System MUST accept `--debug` and `-D` as command-line flags for the `top` subcommand
- **FR-002**: When `--debug` is present with `--hierarchy`, system MUST display a calculation annotation line below each callee entry
- **FR-003**: For direct calls (no intermediaries), annotation MUST use format: `(direct: X.XX%)`
- **FR-004**: For indirect calls, annotation MUST use format: `(via IntermediaryA X.XX% × IntermediaryB Y.YY% × ... = Z.ZZ%)`
- **FR-005**: Intermediary names MUST be simplified (same simplification as main symbol display)
- **FR-006**: All intermediaries in the calculation path MUST be shown (no truncation)
- **FR-007**: Annotation lines MUST be rendered in gray/dim color when color output is enabled
- **FR-008**: When `--no-color` is active, annotation lines MUST appear as plain text (no ANSI codes)
- **FR-009**: When `--debug` is used without `--hierarchy`, system MUST produce normal output (no error, no change)
- **FR-010**: Annotation lines MUST be indented to align with the parent entry's function name column
- **FR-011**: For standalone entries with caller contributions, annotation MUST use format: `(standalone: X.XX% - Y.YY% (CallerA) - Z.ZZ% (CallerB) = W.WW%)`
- **FR-012**: Caller names in standalone annotations MUST be simplified (same simplification as main symbol display)
- **FR-013**: All caller contributions MUST be shown in the standalone annotation (no truncation)
- **FR-014**: For standalone entries where original equals adjusted (no contributions), system MAY omit the annotation

### Key Entities

- **CalculationPath**: Represents the chain of intermediary functions and their percentages from caller to callee
  - Intermediaries: ordered list of (simplified_name, percentage) pairs
  - Final percentage: the computed result
  - Is direct: boolean indicating if path has no intermediaries

- **ContributionBreakdown**: Represents a caller's contribution to a standalone entry's adjusted percentage
  - Caller: simplified name of the calling target function
  - Absolute percentage: the contribution amount subtracted from original

## Success Criteria

### Measurable Outcomes

- **SC-001**: Users can verify percentage calculations by visually comparing the multiplication chain to the final percentage shown
- **SC-002**: All hierarchy entries include exactly one annotation line when `--debug` is active
- **SC-003**: Annotation output matches the actual internal calculation used to derive percentages
- **SC-004**: Debug output adds clarity without disrupting the primary data display (annotation on separate line, visually distinct via gray color)

## Assumptions

- The `--hierarchy` feature (003) is complete and provides the intermediary path data needed for this feature
- Gray/dim color is available via existing color infrastructure in `symbol.rs`
- Simplified symbol names are already computed and accessible during output formatting
