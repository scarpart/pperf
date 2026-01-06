# Implementation Plan: Multi-Report Averaging

**Branch**: `006-multi-report-averaging` | **Date**: 2026-01-05 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/006-multi-report-averaging/spec.md`

## Summary

Enable pperf to analyze multiple perf report files simultaneously, averaging Children% and Self% metrics across all reports for each function. Functions are matched by their full symbol signature. Debug mode (`-D`) extends to show per-report breakdowns. Single-file usage remains backward compatible.

## Technical Context

**Language/Version**: Rust 2024 edition (stable)
**Primary Dependencies**: clap v4 (derive feature) for CLI argument parsing
**Storage**: N/A (file-based CLI tool, reads perf reports from filesystem)
**Testing**: cargo test (unit tests in modules, integration tests in tests/)
**Target Platform**: Linux (primary), portable CLI
**Project Type**: Single CLI application
**Performance Goals**: Process 10 report files without noticeable delay beyond file I/O
**Constraints**: Memory-efficient for typical perf reports (hundreds of functions per file)
**Scale/Scope**: Up to 10 perf report files per invocation, thousands of functions per file

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Test-First Development (NON-NEGOTIABLE) ✅

- **Compliance Plan**: All new functionality will be implemented via TDD
  - Averaging logic: Unit tests with synthetic PerfEntry data before implementation
  - Multi-file parsing: Integration tests with real perf reports from `examples/` before implementation
  - Debug output: Tests verify per-report breakdown format before implementation
  - Clap argument changes: Tests verify multiple file acceptance before CLI modification

### II. Simplicity-First Design ✅

- **Compliance Plan**: Minimal changes to existing architecture
  - Extend existing `PerfEntry` or create thin wrapper for per-report data
  - Reuse existing `parse_file` for each input file
  - Average computation is straightforward arithmetic mean
  - No new abstractions until pattern repetition proves necessity

### III. Real Data Validation ✅

- **Compliance Plan**: Use actual perf reports from `examples/` directory
  - Test files: `Bikes_005_rep0.txt`, `Bikes_005_rep1.txt`, `Bikes_005_rep2.txt`
  - Verify averaged percentages match manual calculation
  - Verify function matching across reports works with real symbol signatures

### IV. Incremental Feature Development ✅

- **Compliance Plan**: Feature has clear, testable scope
  - Core: Multi-file input with averaging
  - Extension: Debug mode per-report breakdown
  - Constraint: Single-file behavior unchanged

## Project Structure

### Documentation (this feature)

```text
specs/006-multi-report-averaging/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (N/A for CLI - no API)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src/
├── main.rs       # CLI entry point, clap argument definitions, orchestration
├── lib.rs        # Library root, PperfError enum
├── parser.rs     # Perf report parsing (parse_file, parse_line, PerfEntry)
├── filter.rs     # Target substring matching
├── symbol.rs     # Symbol simplification and color classification
├── output.rs     # Table formatting (format_table, format_hierarchy_table)
├── hierarchy.rs  # Call tree parsing and relationship discovery
└── averaging.rs  # NEW: Multi-report aggregation and averaging logic

tests/
├── fixtures/
│   └── perf-report.txt  # Existing test fixture
└── top_command.rs       # Integration tests for top subcommand

examples/
├── Bikes_005_rep0.txt   # Real perf report sample (run 1)
├── Bikes_005_rep1.txt   # Real perf report sample (run 2)
└── Bikes_005_rep2.txt   # Real perf report sample (run 3)
```

**Structure Decision**: Single project layout. New averaging logic in dedicated `averaging.rs` module to maintain separation of concerns while avoiding over-abstraction.

## Complexity Tracking

> No constitution violations. All gates pass.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | N/A | N/A |
