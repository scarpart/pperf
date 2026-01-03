# Implementation Plan: List Top Functions

**Branch**: `001-list-top-functions` | **Date**: 2026-01-02 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-list-top-functions/spec.md`

## Summary

Implement the `pperf top` CLI subcommand to parse `perf report` text output and display the top N functions sorted by CPU time (Children% or Self%). Supports filtering by function name prefix. Output follows an `ls -l` style tabular format.

## Technical Context

**Language/Version**: Rust (latest stable, 2024 edition)
**Primary Dependencies**: None (standard library only per constitution)
**Storage**: N/A (file-based input, stdout output)
**Testing**: `cargo test` with real perf-report.txt samples
**Target Platform**: Linux (primary), macOS/Windows (secondary)
**Project Type**: Single CLI binary
**Performance Goals**: <1 second for typical perf reports (~1MB)
**Constraints**: 120-column terminal display width
**Scale/Scope**: Single-user CLI tool, files up to ~10MB

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence |
|-----------|--------|----------|
| I. Test-First Development | PASS | Tasks organized as Red-Green-Refactor cycles |
| II. Simplicity-First Design | PASS | No external dependencies, minimal abstractions |
| III. Real Data Validation | PASS | Tests use actual perf-report.txt from repository |
| IV. Incremental Feature Development | PASS | Single feature with clear scope boundaries |

**Feature Integration Checklist** (to be verified at completion):
- [ ] All tests pass (`cargo test`)
- [ ] Tests validate against real perf-report.txt samples
- [ ] No compiler warnings (`cargo build` clean)
- [ ] Code formatted (`cargo fmt`)
- [ ] Lints pass (`cargo clippy`)

## Project Structure

### Documentation (this feature)

```text
specs/001-list-top-functions/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Phase 0: Format analysis
├── data-model.md        # Phase 1: Data structures
├── quickstart.md        # Phase 1: Usage guide
├── contracts/           # Phase 1: CLI interface
│   └── cli.md           # Command-line interface spec
└── tasks.md             # Phase 2: Implementation tasks
```

### Source Code (repository root)

```text
src/
├── main.rs              # CLI entry point and argument parsing
├── lib.rs               # Library root, re-exports
├── parser.rs            # Perf report parsing logic
├── filter.rs            # Function name filtering
└── output.rs            # Table formatting

tests/
├── integration/
│   └── top_command.rs   # End-to-end CLI tests
└── fixtures/
    └── perf-report.txt  # Symlink to repo root sample
```

**Structure Decision**: Single Rust binary crate with library modules. Tests directory contains integration tests using the real perf-report.txt sample. Unit tests are inline with modules.

## Complexity Tracking

No violations. Design follows all constitution principles without exception.
