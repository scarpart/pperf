# Implementation Plan: Clap CLI Refactor

**Branch**: `005-clap-cli-refactor` | **Date**: 2026-01-03 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/005-clap-cli-refactor/spec.md`

## Summary

Replace the ad-hoc command-line argument parsing in `main.rs` with Clap's derive-based declarative approach. This refactor maintains 100% backward compatibility while gaining Clap's robust parsing, automatic help generation, and standard CLI conventions. The core business logic remains unchanged; only the argument parsing layer is replaced.

## Technical Context

**Language/Version**: Rust (stable, edition 2024)
**Primary Dependencies**: clap (with derive feature) - first external dependency
**Storage**: N/A (CLI tool, no persistent storage)
**Testing**: cargo test (existing integration tests in tests/top_command.rs)
**Target Platform**: Linux (primary), cross-platform compatible
**Project Type**: Single CLI application
**Performance Goals**: N/A (argument parsing overhead negligible)
**Constraints**: Must preserve all existing exit codes and error messages for domain errors
**Scale/Scope**: ~180 lines of argument parsing code to replace in main.rs

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Test-First Development | PASS | Existing tests validate CLI behavior; new tests will verify Clap integration |
| II. Simplicity-First Design | PASS | Clap derive is simpler than hand-rolled parsing |
| III. Real Data Validation | PASS | Tests use real perf-report.txt fixtures |
| IV. Incremental Feature Development | PASS | Single focused refactor, no scope creep |
| No External Dependencies | **VIOLATION** | Clap is an external dependency |

**Violation Justification**: The constitution states "No external dependencies unless explicitly justified by a specific feature requirement." This feature explicitly requires Clap (FR-008). The justification is:
- Hand-rolled argument parsing is error-prone and inflexible
- Clap is the de-facto standard for Rust CLI applications
- Clap provides automatic help, version, and error handling
- The complexity saved outweighs the dependency cost
- Clap has no transitive runtime dependencies when using minimal features

## Project Structure

### Documentation (this feature)

```text
specs/005-clap-cli-refactor/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output (CLI struct definitions)
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src/
├── main.rs      # CLI entry point - PRIMARY CHANGE TARGET
├── lib.rs       # Library root, error types
├── parser.rs    # Perf report parsing (unchanged)
├── filter.rs    # Target filtering (unchanged)
├── symbol.rs    # Symbol simplification (unchanged)
├── output.rs    # Table formatting (unchanged)
└── hierarchy.rs # Call tree parsing (unchanged)

tests/
├── fixtures/    # Real perf-report.txt samples
└── top_command.rs  # Integration tests (may need minor updates)
```

**Structure Decision**: Single project structure. Only `main.rs` requires significant changes. The library modules remain untouched. The Cargo.toml will be updated to add the clap dependency.

## Complexity Tracking

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| External dependency (clap) | FR-008 explicitly requires Clap for robust CLI handling | Hand-rolled parsing is already 110+ lines, error-prone, and lacks standard features like `=` syntax and combined flags |

## Change Analysis

### Files to Modify

1. **Cargo.toml**: Add clap dependency with derive feature
2. **main.rs**: Replace ~110 lines of manual parsing with ~50 lines of Clap structs
3. **tests/top_command.rs**: May need adjustment if test helpers invoke CLI differently

### Files Unchanged

- lib.rs, parser.rs, filter.rs, symbol.rs, output.rs, hierarchy.rs
- All error types and exit codes remain in lib.rs

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Behavioral regression | Low | High | Comprehensive existing test suite |
| Exit code mismatch | Medium | Medium | Explicit exit code mapping post-Clap parsing |
| Help text format change | High | Low | Acceptable per spec assumptions |
| --targets file detection | Medium | Medium | Use Clap's trailing_var_arg or custom validation |

## Constitution Check (Post-Design)

*Re-evaluation after Phase 1 design completion.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Test-First Development | PASS | Design maintains testability; existing tests remain valid |
| II. Simplicity-First Design | PASS | Clap derive (~50 lines) simpler than manual (~110 lines) |
| III. Real Data Validation | PASS | No changes to data parsing; fixtures remain valid |
| IV. Incremental Feature Development | PASS | Scope remains focused on CLI refactor only |
| No External Dependencies | **JUSTIFIED** | Violation documented with explicit justification in Complexity Tracking |

**Post-Design Assessment**: Design is constitution-compliant. The single dependency violation is explicitly justified by the feature requirement (FR-008) and documented in the Complexity Tracking table. Proceed to task generation.
