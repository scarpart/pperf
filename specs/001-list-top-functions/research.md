# Research: Perf Report Format Analysis

**Feature**: 001-list-top-functions
**Date**: 2026-01-02

## Overview

This document analyzes the `perf report` text output format to inform parser implementation.

## Format Structure

### Header Section

Lines starting with `#` are comments/metadata:

```text
# To display the perf.data header info, please use --header/--header-only options.
#
#
# Total Lost Samples: 0
#
# Samples: 5K of event 'cycles'
# Event count (approx.): 274024838576
#
# Children      Self  Command          Shared Object        Symbol
# ........  ........  ...............  ...................  ......
#
```

**Decision**: Skip all lines starting with `#`.
**Rationale**: These are metadata, not profiling data.

### Data Line Format

Top-level entries follow this pattern:

```text
    90.74%     0.00%  jpl-encoder-bin  jpl-encoder-bin      [.] void parallel_for_with_progress<...>
```

**Column Layout** (whitespace-separated, left-aligned):

| Column | Width | Content | Notes |
|--------|-------|---------|-------|
| Children | 8 chars | `XX.XX%` | Right-aligned within field |
| Self | 8 chars | `XX.XX%` | Right-aligned within field |
| Command | 15 chars | Process name | Left-aligned |
| Shared Object | 19 chars | Library/binary | Left-aligned |
| Symbol | Variable | Function name | Contains `[.]` or `[k]` prefix |

**Decision**: Parse using fixed column positions, not whitespace splitting.
**Rationale**: Symbol column contains spaces (C++ templates, lambdas).

### Symbol Column Format

The Symbol column has a type prefix:
- `[.]` - User-space function
- `[k]` - Kernel function

Examples:
```text
[.] void parallel_for_with_progress<...>
[k] 0000000000000000
```

**Decision**: Strip the `[.]` or `[k]` prefix from output.
**Rationale**: Prefix is metadata, not part of function name.

### Call Tree Lines

Indented lines show the call hierarchy:

```text
            |
            ---void parallel_for_with_progress<...>
               JPLM4DTransformModeLightFieldEncoder<...>
               |
               |--71.80%--TransformPartition::rd_optimize_transform(...)
```

Pattern indicators:
- `|` - Vertical continuation
- `---` - Call relationship
- `--XX.XX%--` - Percentage with call relationship

**Decision**: Skip call tree lines for `top` command.
**Rationale**: Top command shows flat list, not hierarchy. Call tree lines don't have the full column format.

### Identifying Top-Level Entries

Top-level data lines can be identified by:
1. Not starting with `#`
2. Starting with whitespace followed by a percentage pattern: `^\s+\d+\.\d+%`
3. Having exactly the 5-column format

**Decision**: Match lines with regex `^\s+(\d+\.\d+)%\s+(\d+\.\d+)%` to identify data lines.
**Rationale**: Robust identification of lines containing both Children and Self percentages.

### Function Name Variations

Observed patterns in Symbol column:

1. **Simple C++ method**: `Block4D::get_linear_position`
2. **Templated class**: `JPLM4DTransformModeLightFieldEncoder<unsigned short>::run_for_block_4d`
3. **Lambda**: `JPLM4DTransformModeLightFieldCodec<unsigned short>::run()::{lambda(auto:1 const&)#2}`
4. **Clone suffix**: `[clone ._omp_fn.0]` or `[clone .isra.0]`
5. **Unresolved address**: `0x7d4c47223efe`
6. **System function**: `__memset_avx2_unaligned_erms`

**Decision**: Preserve full symbol names including clone suffixes.
**Rationale**: Clone suffixes distinguish different compiled versions of the same function.

### Duplicate Function Names

The same function may appear multiple times in the call tree with different percentages. In the top-level flat view, each unique signature appears once.

When filtering by prefix, multiple overloads may match:
```text
TransformPartition::rd_optimize_transform(Block4D const&)
TransformPartition::rd_optimize_transform(Block4D const&, Block4D&, ...)
```

**Decision**: Use `#1`, `#2` suffixes when displaying multiple matches with same base name.
**Rationale**: Spec requirement FR-010. Extract base name up to first `(` for grouping.

## Parsing Strategy

### Algorithm

```
1. Read file line by line
2. Skip lines starting with '#'
3. For lines matching '^\s+(\d+\.\d+)%\s+(\d+\.\d+)%':
   a. Extract Children% (chars 0-8, trimmed)
   b. Extract Self% (chars 8-16, trimmed)
   c. Extract remaining columns by finding '[.]' or '[k]' marker
   d. Symbol is everything after '[.] ' or '[k] '
4. Collect into PerfEntry structs
5. Sort by Children% (default) or Self%
6. Apply filters if --targets specified
7. Take first N entries
8. Format and output
```

### Column Extraction

Given the variable-width nature of the Command and Shared Object columns, the most reliable approach:

1. Parse Children% and Self% from the start (fixed positions)
2. Find `[.]` or `[k]` marker to locate Symbol start
3. Skip Command and Shared Object (not needed for `top` output)

**Decision**: Only extract Children%, Self%, and Symbol.
**Rationale**: Simplicity-First principle. Command and Shared Object are not displayed.

## Alternatives Considered

### Full CSV/TSV Parsing
- Rejected: Symbol column contains spaces, making delimiter-based parsing unreliable.

### Using External Crates (regex, nom)
- Rejected: Constitution mandates minimal dependencies. Standard library string operations suffice.

### Parsing Call Tree for Hierarchy
- Deferred: Not needed for `top` command. Future feature.

## Test Data Requirements

The repository contains `perf-report.txt` (~1MB) with:
- ~5K samples
- Multiple function types (templates, lambdas, clones)
- Unresolved symbols (hex addresses)
- Deep call trees

**Validation Points**:
1. Top entry: `parallel_for_with_progress` at 90.74% Children, 0.00% Self
2. High Self%: `get_mSubbandLF_significance` at 11.94% Self
3. Unresolved: `0x7d4c47223efe` appears in output
4. Clone suffix: `[clone ._omp_fn.0]` preserved

## Summary

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Line identification | Regex `^\s+\d+\.\d+%` | Robust, handles all variations |
| Column extraction | Find `[.]`/`[k]` marker | Avoids whitespace ambiguity |
| Symbol handling | Preserve full name | Clone suffixes matter for profiling |
| Duplicate disambiguation | `#N` suffix | Per FR-010 specification |
| Dependencies | None (std only) | Constitution requirement |
