# Research: Debug Calculation Path

**Feature**: 004-debug-calculation-path
**Date**: 2026-01-03

## Research Questions

### 1. How to capture intermediary path during call tree traversal?

**Decision**: Extend `CallRelation` struct with a new field `intermediary_path: Vec<(String, f64)>` containing ordered (simplified_name, percentage) pairs.

**Rationale**:
- The existing `find_target_callees` function already tracks cumulative percentages through non-target nodes
- Adding a path accumulator parameter follows the same pattern as the existing `target_stack`
- Storing path in `CallRelation` makes it available at output time without re-traversing

**Alternatives considered**:
1. Reconstruct path at output time from call tree → Rejected: would require re-traversal and complex state management
2. Store full CallTreeNode references → Rejected: ownership issues, over-complicated for just names and percentages
3. Log during traversal → Rejected: not testable, breaks clean output model

### 2. How to implement gray/dim text in terminal?

**Decision**: Add `DIM` ANSI code (`\x1b[2m`) to `symbol.rs` alongside existing color codes.

**Rationale**:
- ANSI code `\x1b[2m` is the standard dim/faint attribute, widely supported
- Follows existing pattern in `symbol.rs` for color constants
- Can be combined with existing RESET code

**Alternatives considered**:
1. Use dark gray foreground color (`\x1b[90m`) → Acceptable but less semantic
2. Use reduced brightness via true color → Rejected: compatibility concerns
3. Plain text without styling → Rejected: spec requires gray for visual distinction

### 3. How to format the debug annotation line?

**Decision**: Create a new function `format_debug_annotation` in `output.rs` that:
- Takes the intermediary path and final percentage
- Returns formatted string with proper indentation
- Handles both direct and indirect cases

**Format specifications**:
- Direct: `(direct: 17.23%)`
- Indirect: `(via do_4d_transform 12.84% × inner_product_helper 5.00% = 0.64%)`
- All percentages formatted as `X.XX%` (2 decimal places)
- Multiplication sign: `×` (Unicode U+00D7)

**Rationale**:
- Consistent formatting with existing percentage display
- Clear visual distinction between calculation components
- Unicode multiplication sign is more readable than `*` or `x`

### 4. How to handle the --debug flag in argument parsing?

**Decision**: Add `--debug` and `-D` flags following existing pattern for `--hierarchy`/`-H`.

**Implementation**:
- Add `debug_flag: bool` variable in `run_top`
- Check for `"--debug" | "-D"` in argument matching
- Pass `debug_flag` to `format_hierarchy_table`

**Rationale**: Follows existing CLI pattern established for other flags.

### 5. Where to collect intermediary path data?

**Decision**: Modify `find_target_callees` to:
1. Add new parameter `intermediary_path: &mut Vec<(String, f64)>`
2. Push each non-target node's (name, percentage) when traversing down
3. Pop when backtracking
4. Clone current path into CallRelation when recording a target

**Rationale**:
- Minimal change to existing function signature
- Path naturally accumulated during existing DFS traversal
- Same pattern as existing `target_stack` parameter

## Summary of Changes

| File | Change |
|------|--------|
| `src/symbol.rs` | Add `DIM` constant (`\x1b[2m`) |
| `src/hierarchy.rs` | Add `intermediary_path: Vec<(String, f64)>` field to `CallRelation` |
| `src/hierarchy.rs` | Modify `find_target_callees` to track and record path |
| `src/output.rs` | Add `format_debug_annotation` function |
| `src/output.rs` | Modify `format_hierarchy_table` to accept `debug` flag |
| `src/output.rs` | Call `format_debug_annotation` after each callee entry |
| `src/main.rs` | Parse `--debug`/`-D` flag, pass to format function |

## Dependencies

- Feature 003 (call-hierarchy) must be complete and merged first
- No external crate dependencies required
