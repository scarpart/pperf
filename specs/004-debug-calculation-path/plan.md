# Implementation Plan: Debug Calculation Path

**Branch**: `004-debug-calculation-path` | **Date**: 2026-01-03 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/004-debug-calculation-path/spec.md`

## Summary

Add a `--debug` flag that displays the calculation path for each hierarchy entry, showing how percentages were derived through intermediary functions. When enabled with `--hierarchy`, each callee entry will have a gray annotation line below it showing either `(direct: X.XX%)` for direct calls or `(via IntermediaryA X% × IntermediaryB Y% × ... = Z%)` for indirect calls.

## Technical Context

**Language/Version**: Rust (stable, as per constitution)
**Primary Dependencies**: None (standard library only, per constitution)
**Storage**: N/A (CLI tool, no persistent storage)
**Testing**: `cargo test` (TDD per constitution)
**Target Platform**: Linux CLI (same as existing pperf)
**Project Type**: Single CLI application
**Performance Goals**: Same as existing tool (negligible overhead from debug output)
**Constraints**: No external dependencies (constitution)
**Scale/Scope**: Small feature addition to existing hierarchy module

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Test-First Development | ✅ WILL COMPLY | Tests will be written before implementation |
| II. Simplicity-First Design | ✅ WILL COMPLY | Minimal changes to existing structures |
| III. Real Data Validation | ✅ WILL COMPLY | Tests will use perf-report.txt |
| IV. Incremental Feature Development | ✅ WILL COMPLY | Single focused feature |

**No violations detected.** Proceed to Phase 0.

## Project Structure

### Documentation (this feature)

```text
specs/004-debug-calculation-path/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (via /speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── main.rs          # Add --debug/-D flag parsing
├── lib.rs           # No changes needed
├── parser.rs        # No changes needed
├── filter.rs        # No changes needed
├── symbol.rs        # Add gray/dim color formatting function
├── hierarchy.rs     # Extend CallRelation with intermediary path data
└── output.rs        # Add debug annotation line rendering

tests/
└── (integration tests via cargo test)
```

**Structure Decision**: Existing single-project Rust CLI structure. Changes are localized to:
1. `hierarchy.rs` - Add path tracking to CallRelation
2. `output.rs` - Render debug annotations
3. `symbol.rs` - Gray color helper
4. `main.rs` - Parse --debug flag

## Complexity Tracking

> No violations to justify - design follows constitution principles.
