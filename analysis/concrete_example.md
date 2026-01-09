# Concrete Example: Sample-by-Sample Analysis

## Setup: 100 Samples Total

Consider 3 target functions: A, B, C

### Sample Distribution

| Samples | Call Stack | A on stack? | B on stack? | C on stack? |
|---------|------------|-------------|-------------|-------------|
| 30      | X → A      | ✓           |             |             |
| 40      | X → A → B  | ✓           | ✓           |             |
| 10      | X → A → B → C | ✓        | ✓           | ✓           |
| 15      | Y → B      |             | ✓           |             |
| 5       | Y → B → C  |             | ✓           | ✓           |

### Children% Calculation

- A: appears in 30 + 40 + 10 = **80 samples → 80%**
- B: appears in 40 + 10 + 15 + 5 = **70 samples → 70%**
- C: appears in 10 + 5 = **15 samples → 15%**

Sum of Children%: 80 + 70 + 15 = **165%** (due to overlap)

### Call Relations (from perf report)

From A's call tree (A has 80% Children%):
- A → B: 50/80 = 62.5% relative → 50% absolute (samples where A calls B)

From B's call tree (B has 70% Children%):
- B → C: 15/70 = 21.4% relative → 15% absolute (samples where B calls C)

### Current Contribution Calculation

Contributions TO each function (from target callers):

**A**: No target calls A → contributions = 0%
**B**: A calls B → contribution = **50%** (absolute)
**C**: B calls C → contribution = **15%** (absolute)

### Current Standalone Calculation

- A standalone: 80% - 0% = **80%**
- B standalone: 70% - 50% = **20%**
- C standalone: 15% - 15% = **0%**

Sum of standalones: 80 + 20 + 0 = **100%** ✓

### Wait - This Sums to Exactly 100%!

Let me trace through what each standalone represents:

**A's 80%**: Samples 1-80 (all A samples)
- Includes 50 samples where A→B
- Includes 10 samples where A→B→C

**B's 20%**: 70% - 50% = samples where B is on stack but A is NOT on stack
- Samples: Y → B (15) + Y → B → C (5) = 20 samples ✓
- Does NOT include any A→B samples

**C's 0%**: 15% - 15% = samples where C is on stack but B is NOT on stack
- There are no such samples (C is only ever called through B)
- C's time is fully attributed to B

### Verification

Let's partition all 100 samples:
- A standalone (30 + 40 + 10 = 80): includes all A samples
- B standalone (15 + 5 = 20): B samples where A not on stack
- C standalone (0): no C samples where B not on stack

But wait - A's 80 includes samples 41-50 (A→B) and 71-80 (A→B→C).
These samples also have B on stack.

The issue: A's standalone includes B's time!

**True partition should be:**
- A exclusive: 30 samples (just X→A)
- B's contribution under A: 40 samples (X→A→B)
- C's contribution under A→B: 10 samples (X→A→B→C)
- B not via A: 15 samples (Y→B)
- C's contribution under B not via A: 5 samples (Y→B→C)

Sum: 30 + 40 + 10 + 15 + 5 = 100 ✓

### What the Hierarchy Output Shows

```
80%  A                          ← A's full Children%
  62.5%  B (50% absolute)       ← B's contribution under A
    21.4%  C (10% absolute)     ← C's contribution under A→B
20%  B (standalone)             ← B not via A
  21.4%  C (5% absolute)        ← C's contribution under standalone B
0%   C (standalone)             ← C not via A or B (none exist)
```

The display is consistent! The 80% + 20% + 0% = 100% represents:
- 80%: All execution paths through A (including those that go through B and C)
- 20%: Execution paths through B that don't go through A
- 0%: Execution paths through C that don't go through B

## Why Does the Real Data Sum to >100%?

The real data has mutual recursion:
```
rd_optimize_transform → evaluate_split → rd_optimize_transform (recursive)
```

This creates a scenario where:
- A calls B which calls A again
- Some samples have A→B→A on the stack
- A appears TWICE on these stacks

In perf's Children%, each sample is counted once per function regardless of how many times the function appears on the stack. But the CONTRIBUTION calculation may be double-counting in recursive scenarios.

Let me create a recursive example:

### Recursive Example: 100 Samples

| Samples | Call Stack     | A count | B count |
|---------|----------------|---------|---------|
| 20      | X → A          | 1       | 0       |
| 30      | X → A → B      | 1       | 1       |
| 40      | X → A → B → A  | 2       | 1       |
| 10      | Y → B          | 0       | 1       |

Children% (samples where function appears, regardless of count):
- A: 20 + 30 + 40 = **90%**
- B: 30 + 40 + 10 = **80%**

Relations from A's tree:
- A → B: (30+40)/90 = 77.8% relative → 70% absolute

Relations from B's tree:
- B → A: 40/80 = 50% relative → 40% absolute

Contributions:
- A: B calls A → 40%
- B: A calls B → 70%

Standalones:
- A: 90% - 40% = **50%**
- B: 80% - 70% = **10%**

Sum: 50% + 10% = **60%** < 100%

Hmm, this is LESS than 100%, not more!

### Analysis

The recursive case gives <100% because the mutual calling creates higher contributions that get subtracted.

So why does the real data give 167%?

The real data has MORE targets with LESS mutual calling. Functions like:
- get_mSubbandLF_significance (31%) - called by rd_optimize but also called directly
- DCT4DBlock (25%) - called by rd_optimize but also has other callers

These functions have high original Children% but low contributions from target callers, leading to high standalones.

## Final Conclusion

The >100% sum happens when:
1. Multiple targets have overlapping call trees (A→B means both appear in same samples)
2. But the contribution from target callers is small relative to total Children%
3. Because targets have significant time from NON-target callers

In the real data:
- Many targets are called from non-target functions
- So their standalones represent "time not via OTHER targets"
- But the caller targets' standalones include time spent IN callees
- This overlap is intentional per the design

**The >100% is expected behavior, not a bug.**

## The Key Insight: Caller Standalones Include Callee Time

The fundamental reason for >100% is:

```
rd_optimize_transform standalone (51.52%) INCLUDES:
├─ rd_optimize exclusive time
├─ time spent in evaluate_split (when rd_optimize calls it)
├─ time spent in hexadecatree (when rd_optimize calls it)
├─ time spent in DCT4DBlock (when rd_optimize calls it)
└─ ... all callee time
```

Meanwhile:
```
evaluate_split standalone (12.37%) = evaluate_split time NOT via rd_optimize
hexadecatree standalone (42.73%) = hexadecatree time NOT via rd_optimize
DCT4DBlock standalone (21.20%) = DCT4DBlock time NOT via rd_optimize
```

The callee standalones are DISJOINT from each other (different call paths).
But the caller's standalone INCLUDES its callee time by design.

### Why This Design?

The hierarchy output is meant to show:
1. **Full function profiles** - rd_optimize shows its total 51.52% including all its callees
2. **Attribution breakdown** - callees are shown indented with their contribution
3. **Remaining time** - callee standalones show time from OTHER call paths

If we wanted sum = 100%, we'd need to show only EXCLUSIVE time for callers. But that would hide the call relationships.

### Verification with Numbers

```
rd_optimize standalone:    51.52%   (includes 13.08% in hexadecatree)
hexadecatree standalone:   42.73%   (NOT via rd_optimize)
                          -------
Sum:                       94.25%

hexadecatree total:        55.81%   = 13.08% (via rd_optimize) + 42.73% (standalone)
```

The 13.08% appears in BOTH rd_optimize's 51.52% AND is accounted for separately.
This is intentional - rd_optimize shows its full profile, hexadecatree shows its remainder.
