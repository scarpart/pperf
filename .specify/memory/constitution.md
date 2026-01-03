<!--
  Sync Impact Report
  ==================
  Version change: 0.0.0 → 1.0.0 (initial ratification)

  Added principles:
  - I. Test-First Development (NON-NEGOTIABLE)
  - II. Simplicity-First Design
  - III. Real Data Validation
  - IV. Incremental Feature Development

  Added sections:
  - Technology Stack (Rust-specific constraints)
  - Development Workflow (TDD cycle specification)

  Templates requiring updates:
  - .specify/templates/plan-template.md ✅ (already aligned with TDD gates)
  - .specify/templates/spec-template.md ✅ (already aligned with testable scenarios)
  - .specify/templates/tasks-template.md ✅ (already specifies test-before-implement)

  Follow-up TODOs: None
-->

# pperf Constitution

## Core Principles

### I. Test-First Development (NON-NEGOTIABLE)

Every feature MUST follow the TDD Red-Green-Refactor cycle without exception:

1. **Red**: Write tests that define the expected behavior BEFORE any implementation
2. **Green**: Implement the minimal code to make tests pass
3. **Refactor**: Clean up while keeping tests green

This principle is absolute. No feature code may be written until its tests exist and fail.
The Rust compiler provides compile-time guarantees; tests provide runtime correctness guarantees.
Both are required.

### II. Simplicity-First Design

Code MUST be as simple as possible while meeting requirements:

- Avoid abstractions until repetition proves their necessity
- Prefer explicit code over clever code
- Omit comments unless the logic is genuinely non-obvious
- Delete unused code rather than commenting it out
- Reject premature optimization

Rust's type system already documents intent; additional comments are noise unless clarifying
complex algorithms or non-obvious invariants.

### III. Real Data Validation

All features MUST be validated against actual `perf report` output:

- The repository contains real perf-report.txt samples from JPLM encoder profiling
- Tests MUST verify claims about parsed data against these real samples
- Edge cases discovered in real data MUST be encoded as test cases
- Parser behavior MUST match observed perf report format variations

Synthetic test data alone is insufficient. Real-world data exposes format quirks and edge cases
that synthetic data misses.

### IV. Incremental Feature Development

Features are specified and implemented one at a time:

- Each feature has a clear, testable scope
- A feature is complete only when its tests pass against real data
- New features build on proven foundations
- Scope creep within a feature MUST be rejected; new capabilities become new features

## Technology Stack

**Language**: Rust (latest stable)

Rust is chosen for:
- Compile-time safety guarantees reducing runtime bugs
- Strong type system enabling self-documenting code
- Pattern matching well-suited for parsing structured text
- Zero-cost abstractions when performance matters

**Testing**: `cargo test`

All tests run via the standard Rust test framework. Integration tests may use real
perf-report.txt files checked into the repository.

**No external dependencies** unless explicitly justified by a specific feature requirement.
Standard library suffices for text parsing and basic I/O.

## Development Workflow

### TDD Cycle (Mandatory)

```
1. Understand the feature requirement
2. Write failing tests (cargo test → RED)
3. Verify tests fail for the right reason
4. Implement minimal passing code (cargo test → GREEN)
5. Refactor for clarity (cargo test → still GREEN)
6. Commit
```

### Feature Integration Checklist

Before a feature is considered complete:

- [ ] All tests pass (`cargo test`)
- [ ] Tests validate against real perf-report.txt samples
- [ ] No compiler warnings (`cargo build` clean)
- [ ] Code formatted (`cargo fmt`)
- [ ] Lints pass (`cargo clippy`)

## Governance

This constitution supersedes all other development practices for this project.

**Amendments** require:
1. Written justification for the change
2. Update to this file with new version number
3. Propagation of changes to dependent templates

**Versioning**:
- MAJOR: Principle added, removed, or fundamentally redefined
- MINOR: Clarification that changes expected behavior
- PATCH: Typo fixes, formatting, non-semantic edits

**Compliance**: Every PR and code review MUST verify adherence to these principles.
Non-compliance is grounds for rejection regardless of code quality.

**Version**: 1.0.0 | **Ratified**: 2026-01-02 | **Last Amended**: 2026-01-02
