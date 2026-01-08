# Data Model: Exact Function Signature Target File

**Feature**: 007-exact-target-file
**Date**: 2026-01-08

## Entities

### TargetFile

A text file containing function signatures for exact matching.

**Format**:
```
# Comment lines start with #
# Blank lines are ignored

Hierarchical4DEncoder::get_rd_for_below_inferior_bit_plane(LightfieldCoordinate<unsigned int> const&, LightfieldDimension<unsigned int, true> const&)
DCT4DBlock::DCT4DBlock(Block4D const&, double)
```

**Parsing Rules**:
- Lines starting with `#` are comments (ignored)
- Empty lines and whitespace-only lines are ignored
- Leading/trailing whitespace is trimmed from each signature
- Each non-comment, non-empty line is a function signature

### TargetSignature

A single function signature string used for exact matching.

**Attributes**:
- `signature: String` - The exact function signature text (after trimming)
- Source: Parsed from a single line of the target file

**Validation**:
- Must be non-empty after trimming
- No format validation beyond non-empty (signatures are arbitrary strings)

### MatchResult

Result of matching a signature against perf report entries.

**Variants**:
```rust
enum MatchResult {
    /// Signature matches exactly one unique function
    Unique(PerfEntry),

    /// Signature matches multiple different functions (ambiguous)
    Ambiguous {
        signature: String,
        matches: Vec<String>,  // List of distinct matching symbols
    },

    /// Signature matches no entries
    NoMatch(String),  // The unmatched signature
}
```

### ExactMatchMode

Enum distinguishing between substring and exact matching modes.

```rust
enum TargetMode {
    /// Legacy substring matching via -t flag
    Substring(Vec<String>),

    /// Exact matching via --target-file
    Exact(Vec<String>),

    /// No targets specified
    None,
}
```

## Extended Error Types

### PperfError (additions to existing enum)

```rust
pub enum PperfError {
    // ... existing variants ...

    /// Target file specified but not found or unreadable
    TargetFileNotFound(String),

    /// Target file contains no valid signatures
    EmptyTargetFile,

    /// A signature matches multiple distinct functions
    AmbiguousTarget {
        signature: String,
        matches: Vec<String>,
    },

    /// One or more signatures match no entries
    UnmatchedTargets(Vec<String>),

    /// Both --target-file and -t specified (if Clap doesn't catch it)
    ConflictingTargetArgs,
}
```

## Relationships

```
TargetFile (1) ────────────> (*) TargetSignature
                                      │
                                      │ matches against
                                      ▼
                               PerfEntry.symbol
                                      │
                                      │ produces
                                      ▼
                               MatchResult
```

## State Transitions

### Target File Processing Flow

```
┌─────────────────┐
│   File Path     │
└────────┬────────┘
         │ read_to_string()
         ▼
┌─────────────────┐
│   Raw Content   │
└────────┬────────┘
         │ parse lines
         ▼
┌─────────────────┐
│  Target Sigs    │──── Empty? ──▶ EmptyTargetFile error
└────────┬────────┘
         │ match against entries
         ▼
┌─────────────────┐
│  Match Results  │
└────────┬────────┘
         │
    ┌────┴────────────┐
    │                 │
    ▼                 ▼
All Unique?      Any Ambiguous?
    │                 │
    ▼                 ▼
Continue         AmbiguousTarget error
    │
    ▼
Any NoMatch?
    │
    ▼
UnmatchedTargets error (if any)
```

## CLI Argument Model

### TopArgs (extended)

```rust
struct TopArgs {
    /// Sort by Self% instead of Children%
    sort_self: bool,

    /// Number of functions to display
    number: usize,

    /// Filter by function name substrings (legacy mode)
    #[arg(conflicts_with = "target_file")]
    targets: Vec<String>,

    /// File containing exact function signatures
    #[arg(conflicts_with = "targets")]
    target_file: Option<PathBuf>,

    /// Display call relationships between targets
    hierarchy: bool,

    /// Show calculation path for hierarchy percentages
    debug: bool,

    /// Disable colored output
    no_color: bool,

    /// Perf report file to analyze
    file: PathBuf,
}
```

## Exit Code Mapping

| Error Variant | Exit Code | Description |
|---------------|-----------|-------------|
| FileNotFound | 1 | Perf report file not found |
| TargetFileNotFound | 1 | Target file not found |
| InvalidFormat | 2 | Invalid perf report format |
| InvalidCount | 3 | Invalid -n value |
| HierarchyRequiresTargets | 3 | --hierarchy without targets |
| ConflictingTargetArgs | 3 | Both --target-file and -t |
| NoMatches | 4 | No functions match targets |
| AmbiguousTarget | 5 | Signature matches multiple functions |
| UnmatchedTargets | 6 | Signatures match no functions |
| EmptyTargetFile | 6 | Target file has no valid signatures |
