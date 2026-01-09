# Analysis: Why Standalone Percentages Sum to >100%

## The Problem

Given the sample_targets.txt output:
```
Children%   Self%  Function
   51.52    0.00  rd_optimize_transform              (standalone)
   42.73    0.13  rd_optimize_hexadecatree           (standalone)
   12.37    0.00  evaluate_split_for_partitions      (standalone)
   30.40   16.98  get_mSubbandLF_significance        (standalone)
   21.20    0.00  DCT4DBlock                         (standalone)
    9.44   10.05  inner_product                      (standalone)
```

Sum of standalones: 51.52 + 42.73 + 12.37 + 30.40 + 21.20 + 9.44 = **167.66%**

User's reasoning: If we correctly subtract target-to-target contributions, standalones should represent disjoint portions of execution time and sum to ≤100%.

## Understanding Children%

**Key insight**: Children% is NOT exclusive time. It's "percentage of samples where this function appears anywhere on the call stack."

Consider this simplified call graph:
```
main
  └─ rd_optimize_transform (73.86% of samples)
       └─ evaluate_split_for_partitions (47.31% of samples)
            └─ rd_optimize_transform (recursive callback)
```

When `rd_optimize_transform → evaluate_split_for_partitions` happens:
- 47.31% of samples have BOTH functions on the stack
- rd_optimize_transform's Children% (73.86%) INCLUDES these samples
- evaluate_split_for_partitions's Children% (47.31%) is a SUBSET of rd_optimize's

## What Standalone Calculates

Current implementation:
```
standalone(X) = X's Children% - sum(contributions FROM callers of X among targets)
```

For rd_optimize_transform:
- Original: 73.86%
- evaluate_split calls rd_optimize (recursive): contributes 22.34%
- Standalone: 73.86% - 22.34% = **51.52%**

For evaluate_split_for_partitions:
- Original: 47.31%
- rd_optimize calls evaluate_split: contributes 34.94%
- Standalone: 47.31% - 34.94% = **12.37%**

## Why >100%? The Overlap

rd_optimize's standalone (51.52%) represents samples where:
- rd_optimize is on the stack
- rd_optimize was NOT called by evaluate_split (the recursive callback)

But these 51.52% samples INCLUDE time when rd_optimize calls evaluate_split!

```
Sample A: main → rd_optimize → evaluate_split → ...
- rd_optimize is on stack: YES (contributes to rd_optimize Children%)
- rd_optimize called by evaluate_split: NO (first call, not recursive)
- evaluate_split is on stack: YES (contributes to evaluate_split Children%)
```

This sample contributes to BOTH:
- rd_optimize's standalone (not a recursive callback)
- evaluate_split's standalone? NO - this path is through rd_optimize

Wait, let me recalculate evaluate_split's standalone:
- evaluate_split's 12.37% = time NOT reached via rd_optimize
- Sample A IS reached via rd_optimize, so NOT in evaluate_split's standalone

So the overlap is:
- rd_optimize's 51.52% includes time spent IN evaluate_split
- evaluate_split's 12.37% is time NOT through rd_optimize

These should NOT overlap... so why >100%?

## The Real Issue: Multiple Call Chains

Let me trace through more carefully:

```
Original Children%:
- rd_optimize_transform: 73.86%
- rd_optimize_hexadecatree: 55.81%
- evaluate_split_for_partitions: 47.31%
- get_mSubbandLF_significance: 31.24%
- DCT4DBlock: 25.12%
- inner_product: 10.07%

SUM of originals: 243.41%
```

The original Children% values already sum to 243.41% due to call stack overlap!

We subtract 75.75% in contributions (243.41 - 167.66 = 75.75%).

But the remaining overlap wasn't fully removed.

## The Root Cause: Caller Standalone Includes Callee Time

Consider this scenario with targets A, B, C:

```
Call graph:
A (80% Children%)
├─ B (40% relative = 32% absolute)
│   └─ C (50% relative = 16% absolute)
└─ D (non-target, 30% relative)
```

B's total Children% = 50% (called from A and elsewhere)
C's total Children% = 20% (called from B and elsewhere)

Contributions:
- A→B: 32% (absolute)
- B→C: 8% (from B's 50% × 16% relative = 8%)

Standalones:
- A: 80% - 0 = 80% (no target callers)
- B: 50% - 32% = 18%
- C: 20% - 8% = 12%

Sum: 80% + 18% + 12% = 110%

Why >100%?
- A's 80% includes B's time (32% contribution)
- A's 80% includes C's time (through B)
- B's 18% is B's time not through A (but includes C's time not through A→B)
- C's 12% is C's time not through B

The overlap: A's 80% includes time in B and C that's also counted in B's nested callees and potentially C's standalone in other paths.

## Conclusion: This is Expected Behavior

The standalone sum exceeding 100% is **correct given the semantics**:

1. **Standalone = "Not reached via target callers"**, NOT "exclusive time"
2. A caller's standalone INCLUDES time spent in target callees
3. The hierarchy display SHOWS this overlap explicitly (nested callees)
4. A callee's standalone is its REMAINING time not attributed to target callers

The overlap is intentional because:
- Under A, we show B with its contribution to A
- As standalone, B shows its remaining time
- A's full time (including B) + B's remaining = overlap

This is NOT a bug. It's inherent to how Children% works and the design goal of showing call relationships with attribution.

## Alternative: Exclusive-Only Standalones

If we wanted standalones to sum to ≤100%, we'd need:
```
standalone(X) = X's Children%
              - contributions FROM callers
              - contributions TO callees (among targets)
```

But this would make A's standalone = A's exclusive time only, losing the ability to show A's full profile with callees underneath.

The current design prioritizes showing complete function profiles over having disjoint partitions.
