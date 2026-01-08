# Implementation Plan: Exact Function Signature Target File

**Branch**: `007-exact-target-file` | **Date**: 2026-01-08 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/007-exact-target-file/spec.md`

## Summary

Replace the current substring-based target matching (`-t`) with a file-based exact function signature matching approach (`--target-file`). The new flag accepts a file containing one function signature per line, performs exact matching against raw perf report symbols, and detects/reports ambiguity when signatures match multiple entries. Backward compatibility with existing `-t` flag is preserved.

## Technical Context

**Language/Version**: Rust (stable, edition 2024)
**Primary Dependencies**: clap v4 (with derive feature) for CLI parsing
**Storage**: N/A (CLI tool, file-based input)
**Testing**: cargo test (unit + integration tests with real perf report samples)
**Target Platform**: Unix/Linux CLI (Darwin/Linux)
**Project Type**: Single CLI application
**Performance Goals**: Parse target files instantly (< 10ms for typical file with < 100 signatures)
**Constraints**: Must validate all signatures before processing begins; clear error messages for ambiguity
**Scale/Scope**: Target files typically contain 1-50 function signatures

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Test-First Development (NON-NEGOTIABLE)
- [ ] Tests written BEFORE implementation for:
  - Target file parsing (comments, blank lines, whitespace trimming)
  - Exact matching logic against raw symbols
  - Ambiguity detection (single signature matches multiple entries)
  - Missing signature detection (signature matches zero entries)
  - Mutual exclusivity validation (`--target-file` vs `-t`)
  - Error formatting and exit codes

### II. Simplicity-First Design
- [x] No premature abstractions - extend existing `filter.rs` module
- [x] Reuse existing `PperfError` enum for new error variants
- [x] File parsing is straightforward line-by-line with standard library
- [x] No external dependencies needed beyond existing clap

### III. Real Data Validation
- [ ] Tests MUST use real perf report samples from `examples/` directory
- [ ] Validate exact matching with functions like:
  - `Hierarchical4DEncoder::get_rd_for_below_inferior_bit_plane(...)`
  - `DCT4DBlock::DCT4DBlock(Block4D const&, double)`
  - `Hierarchical4DEncoder::get_mSubbandLF_significance(...)`
- [ ] Verify ambiguity detection with partial patterns like `get_rd_for_below`, `DCT4DBlock`

### IV. Incremental Feature Development
- [x] Feature scope is bounded: target file parsing + exact matching + ambiguity detection
- [x] Builds on existing filter/hierarchy infrastructure
- [x] No scope creep: substring `-t` preserved unchanged

**GATE STATUS**: Ready to proceed. All design constraints satisfied.

## Project Structure

### Documentation (this feature)

```text
specs/007-exact-target-file/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (N/A - CLI tool)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src/
├── main.rs          # CLI entry point - add --target-file arg, validation
├── lib.rs           # Error types - add new error variants
├── parser.rs        # Perf report parsing (unchanged)
├── filter.rs        # Target matching - add exact matching mode
├── symbol.rs        # Symbol simplification (unchanged for display)
├── output.rs        # Table formatting (unchanged)
└── hierarchy.rs     # Call hierarchy - update target matching calls

tests/
└── top_command.rs   # Integration tests - add target file test cases

examples/
├── Bikes_005_rep0.txt   # Real perf report for testing
├── Bikes_005_rep1.txt
└── Bikes_005_rep2.txt
```

**Structure Decision**: Single project structure. This feature extends existing modules without adding new crates or major architectural changes.

## Complexity Tracking

> No constitution violations requiring justification.

| Violation | Why Needed | Simpler Alternative Rejected Because |
|-----------|------------|-------------------------------------|
| N/A | N/A | N/A |
