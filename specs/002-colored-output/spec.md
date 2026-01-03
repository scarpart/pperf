# Feature Specification: Colored Output with Simplified Function Names

**Feature Branch**: `002-colored-output`
**Created**: 2026-01-02
**Status**: Draft
**Input**: User description: "Better output presentation with colored function names (blue for user functions, yellow for stdlib, red for undefined) and truncated symbols showing only namespace::function"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - View Color-Coded Function Types (Priority: P1)

A developer analyzing a perf report wants to quickly distinguish between their own code, library code, and unresolved symbols at a glance using color coding.

**Why this priority**: Color differentiation is the primary visual enhancement that immediately improves readability and analysis speed. Users can spot hotspots in their own code vs. library overhead instantly.

**Independent Test**: Run `pperf top perf-report.txt` in a terminal with color support and verify that user functions appear in blue, standard library functions in yellow, and undefined symbols in red.

**Acceptance Scenarios**:

1. **Given** a terminal with color support, **When** running `pperf top report.txt`, **Then** user-defined functions (from the target binary) are displayed in blue
2. **Given** a terminal with color support, **When** running `pperf top report.txt`, **Then** standard library functions (std::, libc, libm, etc.) are displayed in yellow
3. **Given** a terminal with color support, **When** running `pperf top report.txt`, **Then** unresolved symbols (hex addresses like `0x7d4c...`) are displayed in red
4. **Given** a terminal without color support, **When** running `pperf top report.txt`, **Then** output is displayed without color codes (graceful fallback)

---

### User Story 2 - View Simplified Function Names (Priority: P1)

A developer wants to see concise function names without clutter from return types, template parameters, and argument lists, making it easier to identify hotspot functions quickly.

**Why this priority**: Long C++ mangled names with templates and arguments make the output hard to scan. Showing only `Namespace::FunctionName` dramatically improves readability.

**Independent Test**: Run `pperf top perf-report.txt` and verify that function names show only the namespace/class and function name, without return types or parameter lists.

**Acceptance Scenarios**:

1. **Given** a function like `void Hierarchical4DEncoder::get_mSubbandLF_significance(unsigned int, ...)`, **When** displayed, **Then** output shows `Hierarchical4DEncoder::get_mSubbandLF_significance`
2. **Given** a templated function like `double std::inner_product<double*, double const*, double>(...)`, **When** displayed, **Then** output shows `std::inner_product` (template parameters removed)
3. **Given** a lambda or anonymous function, **When** displayed, **Then** a reasonable short form is shown (e.g., `{lambda}` or the enclosing function name)
4. **Given** an unresolved symbol like `0x7d4c47223efe`, **When** displayed, **Then** it remains displayed as-is

---

### User Story 3 - Disable Colors When Needed (Priority: P2)

A developer wants to disable colors when piping output to a file or when colors interfere with their workflow.

**Why this priority**: Color codes in redirected output create unreadable files. Users need control over this behavior.

**Independent Test**: Run `pperf top report.txt --no-color` and verify plain text output without ANSI codes.

**Acceptance Scenarios**:

1. **Given** output is piped to a file, **When** running `pperf top report.txt > out.txt`, **Then** colors are automatically disabled (no ANSI codes in file)
2. **Given** the `--no-color` flag, **When** running `pperf top --no-color report.txt`, **Then** output has no color codes
3. **Given** the `NO_COLOR` environment variable is set, **When** running `pperf top report.txt`, **Then** output has no color codes

---

### Edge Cases

- What happens when a function name cannot be parsed (malformed symbol)?
  - Display the original symbol as-is, colored red as "unknown"
- How are kernel symbols (marked with `[k]`) handled?
  - Display in yellow (treated as system/library code)
- What if the terminal doesn't support colors?
  - Detect automatically and output plain text
- How are clone suffixes (`.cold`, `.part.0`) handled in simplification?
  - Strip them from the display (e.g., `func.cold.123` becomes `func`)

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST color user-defined function names in blue
- **FR-002**: System MUST color standard library and system functions in yellow
- **FR-003**: System MUST color unresolved symbols (hex addresses) in red
- **FR-004**: System MUST detect terminal color support and disable colors when not available
- **FR-005**: System MUST disable colors when output is piped (not a TTY)
- **FR-006**: System MUST support `--no-color` flag to force disable colors
- **FR-007**: System MUST respect the `NO_COLOR` environment variable
- **FR-008**: System MUST simplify function names to show only namespace/class and function name
- **FR-009**: System MUST strip return types from displayed function names
- **FR-010**: System MUST strip argument lists (including parentheses) from displayed function names
- **FR-011**: System MUST strip template parameters from displayed function names
- **FR-012**: System MUST strip clone suffixes (`.cold`, `.part.N`, `.isra.N`) from function names
- **FR-013**: System MUST preserve the existing 100-character truncation with "..." for very long names

### Symbol Classification Rules

The system classifies symbols into three categories:

1. **User-defined (Blue)**:
   - Functions from the target binary's shared object
   - Identified by `[.]` marker in perf report AND matching the command name

2. **Standard Library/System (Yellow)**:
   - Functions with `std::` namespace prefix
   - Functions from known library shared objects (libc, libm, libstdc++, libpthread, etc.)
   - Kernel symbols marked with `[k]`

3. **Unresolved/Unknown (Red)**:
   - Hex addresses (e.g., `0x7d4c47223efe`, `0000000000000000`)
   - Symbols that cannot be parsed or classified

### Key Entities

- **SymbolType**: Classification of a symbol (User, Library, Unresolved)
- **SimplifiedSymbol**: A function name with return type, arguments, and templates stripped
- **ColoredOutput**: Text with ANSI color codes for terminal display

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can identify function origin type (user/library/unresolved) within 1 second of viewing output
- **SC-002**: Function names fit within 60 characters for 90% of typical C++ symbols after simplification
- **SC-003**: Output remains readable when colors are disabled or piped to a file
- **SC-004**: Zero ANSI escape codes appear in output when piped to a file or when `--no-color` is used

## Assumptions

- ANSI color codes are the standard for terminal coloring (works on Linux, macOS, modern Windows terminals)
- The perf report format reliably includes shared object names to distinguish user vs. library code
- Template simplification removes everything between `<` and `>` including nested templates
- The command name in the perf report matches the user's binary (used to identify user functions)
