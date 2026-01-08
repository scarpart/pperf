# Research: Exact Function Signature Target File

**Feature**: 007-exact-target-file
**Date**: 2026-01-08

## Research Questions

### 1. How should exact matching work against perf report symbols?

**Decision**: Use substring containment for matching (target signature must be a substring of the raw symbol), not strict equality.

**Rationale**:
- Perf report symbols often have prefixes/suffixes not visible in the main display (e.g., `[.] ` prefix, `[clone .isra.0]` suffix)
- Users will copy signatures from the visible output, which may not include these artifacts
- The key requirement is that a signature matches **exactly one** entry - containment achieves this while being user-friendly
- Example: `DCT4DBlock::DCT4DBlock(Block4D const&, double)` should match the entry even if the raw symbol is `[.] DCT4DBlock::DCT4DBlock(Block4D const&, double) [clone .constprop.0]`

**Alternatives considered**:
- Strict equality: Rejected because raw symbols contain artifacts users don't see
- Regex matching: Rejected as overly complex and error-prone for users

### 2. What constitutes "ambiguity" requiring detection?

**Decision**: Ambiguity exists when a single target signature matches more than one **unique** perf entry.

**Rationale**:
- The same function can appear multiple times in the perf report (e.g., recursive calls, different call contexts)
- These are the same function - not ambiguous
- True ambiguity is when `get_rd_for_below` matches both `get_rd_for_below_inferior_bit_plane` and `get_rd_for_below_superior_bit_plane`
- We detect this by checking if the signature matches entries with **different** raw symbols

**Example from real data**:
- `DCT4DBlock::DCT4DBlock(Block4D const&, double)` appears 3+ times in the call tree - NOT ambiguous (same function)
- `DCT4DBlock` matches `DCT4DBlock::DCT4DBlock(...)` and potentially other `DCT4DBlock::*` methods - AMBIGUOUS

**Alternatives considered**:
- Count all matching lines: Rejected because same function appears multiple times legitimately
- Use simplified symbols: Rejected because users specify raw signatures

### 3. How should the target file format be defined?

**Decision**: Plain text, one signature per line, with support for comments and blank lines.

**Rationale**:
- Simple to create and edit with any text editor
- Easy to version control and diff
- Comments allow documenting why certain functions are tracked
- Blank lines improve readability for grouping

**Format specification**:
```
# This is a comment
Hierarchical4DEncoder::get_rd_for_below_inferior_bit_plane(LightfieldCoordinate<unsigned int> const&, LightfieldDimension<unsigned int, true> const&)

# DCT functions
DCT4DBlock::DCT4DBlock(Block4D const&, double)
```

**Alternatives considered**:
- JSON/YAML: Rejected as overkill for a simple list
- CSV: Rejected as unnecessary structure

### 4. Which perf report symbol representation should be matched against?

**Decision**: Match against the **raw symbol** from parser, before simplification.

**Rationale**:
- Users need to specify exact signatures, which include template parameters, const qualifiers, etc.
- Simplified symbols strip valuable disambiguating information
- The raw symbol from the parser is what appears in the perf report output
- Example: Raw `DCT4DBlock::DCT4DBlock(Block4D const&, double)` vs Simplified `DCT4DBlock::DCT4DBlock`

**Implementation note**:
- The `PerfEntry.symbol` field contains the raw symbol
- `simplify_symbol()` is called for display only
- Matching uses `entry.symbol.contains(target_signature)`

**Alternatives considered**:
- Match against simplified symbols: Rejected because it loses disambiguation power
- Match against both: Rejected as unnecessarily complex

### 5. How should we handle the mutual exclusivity of `--target-file` and `-t`?

**Decision**: Use Clap's built-in conflict detection with `conflicts_with` attribute.

**Rationale**:
- Clap provides native support for mutually exclusive arguments
- Generates clear, consistent error messages
- No custom validation code needed

**Implementation**:
```rust
#[arg(long = "target-file", conflicts_with = "targets")]
target_file: Option<PathBuf>,

#[arg(short = 't', long = "targets")]
targets: Vec<String>,
```

**Alternatives considered**:
- Manual validation: Rejected as Clap does this better

### 6. What new error types are needed?

**Decision**: Add three new variants to `PperfError`:

1. `TargetFileNotFound(String)` - When `--target-file` path doesn't exist
2. `AmbiguousTarget { signature: String, matches: Vec<String> }` - When signature matches multiple entries
3. `UnmatchedTargets(Vec<String>)` - When signatures match zero entries
4. `EmptyTargetFile` - When file contains no valid signatures
5. `ConflictingTargetArgs` - When both `--target-file` and `-t` specified (if not handled by Clap)

**Rationale**:
- Each error case requires distinct information for the user
- Structured errors enable specific exit codes per spec
- Matches exist well with existing error pattern in `lib.rs`

### 7. What exit codes should be used for new errors?

**Decision**: Extend existing exit code scheme:

| Exit Code | Error Type |
|-----------|------------|
| 1 | FileNotFound (existing) |
| 2 | InvalidFormat (existing) |
| 3 | InvalidCount / HierarchyRequiresTargets / ConflictingArgs (existing pattern) |
| 4 | NoMatches (existing) |
| 5 | AmbiguousTarget (new) |
| 6 | UnmatchedTargets (new) |

**Rationale**:
- Follows existing pattern in `main.rs`
- Distinct exit codes allow scripted error handling
- Ambiguity is more severe than "no matches" hence separate code

## Key Implementation Insights

### Matching Strategy
The matching algorithm should:
1. Parse target file into list of signatures
2. For each signature, find all entries where `entry.symbol.contains(signature)`
3. Group matches by their raw symbol (deduplicate same-function entries)
4. If any signature matches multiple unique symbols → AmbiguityError
5. If any signature matches zero entries → UnmatchedTargets error
6. Otherwise, proceed with exact-matched entries

### Integration Points
1. **main.rs**: Add `--target-file` argument to `TopArgs`, add validation
2. **lib.rs**: Add new error variants
3. **filter.rs**: Add `filter_entries_exact()` function for exact matching mode
4. **hierarchy.rs**: Update `find_target_in_tree()` and `find_target_callees()` to support exact matching mode

### Test Data from examples/
Real signatures for testing (from `Bikes_005_rep0.txt`):
- `Hierarchical4DEncoder::get_rd_for_below_inferior_bit_plane(LightfieldCoordinate<unsigned int> const&, LightfieldDimension<unsigned int, true> const&)`
- `Hierarchical4DEncoder::get_mSubbandLF_significance(unsigned int, LightfieldCoordinate<unsigned int> const&, LightfieldDimension<unsigned int, true> const&) const`
- `DCT4DBlock::DCT4DBlock(Block4D const&, double)`
- `TransformPartition::rd_optimize_transform(Block4D const&)`

Ambiguous patterns for testing:
- `get_rd_for_below` - should match only one function (only `inferior` variant in this data)
- `DCT4DBlock` - matches constructor, should error if other DCT4DBlock methods exist
- `get_mSubband` - would match `get_mSubbandLF_significance` plus any other `get_mSubband*` methods
