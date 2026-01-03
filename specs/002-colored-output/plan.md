# Implementation Plan: Colored Output with Simplified Function Names

**Branch**: `002-colored-output` | **Date**: 2026-01-02 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/002-colored-output/spec.md`

## Summary

Add color-coded output and simplified function names to pperf. Functions will be colored by type (blue=user, yellow=library, red=unresolved) and symbol names will be stripped of return types, arguments, and template parameters to show only `Namespace::FunctionName`.

## Technical Context

**Language/Version**: Rust (2024 edition, latest stable)
**Primary Dependencies**: None (std library only, per constitution)
**Storage**: N/A (CLI tool, no persistent storage)
**Testing**: cargo test (TDD per constitution)
**Target Platform**: Linux (primary), macOS, Windows with ANSI terminal support
**Project Type**: Single CLI application
**Performance Goals**: No degradation from current performance; symbol parsing <1ms per entry
**Constraints**: No external dependencies; ANSI color codes only; graceful fallback when colors unavailable
**Scale/Scope**: Process perf reports with 100+ entries efficiently

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Test-First Development | ✅ PASS | Tests will be written before implementation for symbol parsing, classification, and color output |
| II. Simplicity-First Design | ✅ PASS | Using std library only; ANSI codes are simple escape sequences |
| III. Real Data Validation | ✅ PASS | Will validate against existing perf-report.txt samples |
| IV. Incremental Feature Development | ✅ PASS | Building on existing parser/output modules |
| No external dependencies | ✅ PASS | ANSI colors via std; TTY detection via std::io::IsTerminal |

## Project Structure

### Documentation (this feature)

```text
specs/002-colored-output/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
│   └── cli.md           # CLI contract updates
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── lib.rs               # Module exports + PperfError (existing)
├── main.rs              # CLI entry point (existing, will add --no-color)
├── parser.rs            # PerfEntry parsing (existing)
├── filter.rs            # Function filtering (existing)
├── output.rs            # Table formatting (MODIFY: add color + simplification)
└── symbol.rs            # NEW: Symbol classification and simplification

tests/
├── fixtures/
│   └── perf-report.txt  # Real test data (existing symlink)
└── top_command.rs       # Integration tests (existing, add color tests)
```

**Structure Decision**: Extend existing single-project structure. Add new `symbol.rs` module for symbol classification and simplification logic. Modify `output.rs` for colored output.

## Complexity Tracking

No constitution violations. Design uses std library only and follows existing patterns.
