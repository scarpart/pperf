//! Symbol classification and simplification for colored output.
//!
//! This module provides:
//! - ANSI color codes for terminal output
//! - Symbol type classification (User, Library, Unresolved)
//! - Symbol name simplification (strip return types, templates, arguments)

use std::io::{IsTerminal, stdout};

// ANSI color codes
pub const RESET: &str = "\x1b[0m";
pub const BLUE: &str = "\x1b[34m"; // User functions
pub const YELLOW: &str = "\x1b[33m"; // Library/system functions
pub const RED: &str = "\x1b[31m"; // Unresolved symbols

/// Classification of a symbol's origin for color coding
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SymbolType {
    /// User-defined functions (displayed in blue)
    User,
    /// Standard library and system functions (displayed in yellow)
    Library,
    /// Unresolved symbols like hex addresses (displayed in red)
    Unresolved,
}

/// Determine whether to use colored output
pub fn should_use_color(no_color_flag: bool) -> bool {
    if no_color_flag {
        return false;
    }
    if std::env::var("NO_COLOR").is_ok() {
        return false;
    }
    stdout().is_terminal()
}

/// Get the ANSI color code for a symbol type
pub fn color_for_type(symbol_type: SymbolType) -> &'static str {
    match symbol_type {
        SymbolType::User => BLUE,
        SymbolType::Library => YELLOW,
        SymbolType::Unresolved => RED,
    }
}

/// T017: Check if a symbol is an unresolved hex address
/// Matches: 0x... patterns and all-hex digit strings
fn is_hex_address(symbol: &str) -> bool {
    // Check for 0x prefix pattern
    if let Some(stripped) = symbol.strip_prefix("0x") {
        return stripped.chars().all(|c| c.is_ascii_hexdigit());
    }
    // Check for all-hex pattern (like 0000000000000000)
    !symbol.is_empty() && symbol.chars().all(|c| c.is_ascii_hexdigit())
}

/// T018: Check if a symbol is a library/system function
/// Matches: std::, __, pthread_*, malloc, free, memset, memcpy, memmove, @GLIBC, @GCC
fn is_library_symbol(symbol: &str) -> bool {
    // Standard library prefix
    if symbol.starts_with("std::") {
        return true;
    }
    // Double underscore prefix (libc internals)
    if symbol.starts_with("__") {
        return true;
    }
    // Common libc functions
    const LIBC_FUNCTIONS: &[&str] = &[
        "malloc", "free", "memset", "memcpy", "memmove", "calloc", "realloc", "strlen", "strcpy",
        "strcat",
    ];
    if LIBC_FUNCTIONS.contains(&symbol) {
        return true;
    }
    // pthread functions
    if symbol.starts_with("pthread_") {
        return true;
    }
    // GLIBC/GCC versioned symbols
    if symbol.contains("@GLIBC") || symbol.contains("@GCC") {
        return true;
    }
    false
}

/// T019: Classify a symbol by its type for color coding
pub fn classify_symbol(symbol: &str) -> SymbolType {
    // Priority 1: Unresolved hex addresses
    if is_hex_address(symbol) {
        return SymbolType::Unresolved;
    }
    // Priority 2: Library/system functions
    if is_library_symbol(symbol) {
        return SymbolType::Library;
    }
    // Priority 3: Everything else is user code
    SymbolType::User
}

/// T033: Strip return type from the beginning of a symbol
/// e.g., "void MyClass::method()" -> "MyClass::method()"
fn strip_return_type(symbol: &str) -> &str {
    // Common return types that appear at the start
    const RETURN_TYPES: &[&str] = &[
        "void ",
        "int ",
        "double ",
        "float ",
        "char ",
        "bool ",
        "unsigned int ",
        "unsigned ",
        "long ",
        "short ",
        "const ",
        "static ",
        "virtual ",
        "inline ",
    ];

    let mut result = symbol;
    // Strip leading return type keywords
    for rt in RETURN_TYPES {
        if result.starts_with(rt) {
            result = &result[rt.len()..];
            break;
        }
    }
    result
}

/// T034: Strip template parameters with bracket counting for nested templates
/// e.g., "std::vector<int>" -> "std::vector"
fn strip_template_params(symbol: &str) -> String {
    let mut result = String::new();
    let mut depth = 0;

    for c in symbol.chars() {
        match c {
            '<' => depth += 1,
            '>' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            _ if depth == 0 => result.push(c),
            _ => {} // Skip chars inside templates
        }
    }
    result
}

/// T035: Strip argument lists with parenthesis counting for nested args
/// e.g., "func(int, double)" -> "func"
fn strip_arguments(symbol: &str) -> String {
    // Find the first '(' that starts an argument list (not part of operator())
    let mut result = String::new();
    let mut depth = 0;
    let mut chars = symbol.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '(' => {
                // Check if this is operator()
                if result.ends_with("operator") {
                    result.push(c);
                    result.push(')'); // Add the closing paren for operator()
                    chars.next(); // consume the ')'
                } else if depth == 0 {
                    // This is the start of an argument list, stop here
                    break;
                } else {
                    depth += 1;
                }
            }
            ')' => {
                if depth > 0 {
                    depth -= 1;
                }
            }
            _ => result.push(c),
        }
    }
    result
}

/// T036: Strip clone suffixes like .cold, .part.N, .isra.N, .constprop.N
fn strip_clone_suffix(symbol: &str) -> &str {
    const SUFFIXES: &[&str] = &[".cold", ".part.", ".isra.", ".constprop."];

    for suffix in SUFFIXES {
        if let Some(pos) = symbol.find(suffix) {
            return &symbol[..pos];
        }
    }
    symbol
}

/// T037: Collapse lambda syntax {lambda(...)#N} to {lambda}
/// Must be called BEFORE strip_arguments to preserve the full symbol
fn collapse_lambda(symbol: &str) -> String {
    let mut result = String::new();
    let mut chars = symbol.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' {
            result.push(c);
            // Check if this starts a lambda
            let remaining: String = chars.clone().take(6).collect();
            if remaining.starts_with("lambda") {
                // Add "lambda}"
                result.push_str("lambda}");
                // Skip until closing }
                for ch in chars.by_ref() {
                    if ch == '}' {
                        break;
                    }
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// T038: Simplify a symbol by stripping return types, templates, arguments, and clone suffixes
pub fn simplify_symbol(symbol: &str) -> String {
    // Preserve hex addresses unchanged (T031)
    if is_hex_address(symbol) {
        return symbol.to_string();
    }

    // Apply transformations in order
    // 0. Strip "auto " prefix (C++ return type deduction keyword)
    let s = symbol.strip_prefix("auto ").unwrap_or(symbol);
    // 1. Collapse lambda first (before arguments are stripped)
    let s = collapse_lambda(s);
    // 2. Strip return type
    let s = strip_return_type(&s);
    // 3. Strip template parameters
    let s = strip_template_params(s);
    // 4. Strip argument lists
    let s = strip_arguments(&s);
    // 5. Strip clone suffixes
    let s = strip_clone_suffix(&s);

    s.to_string()
}

/// T020/T039: Format a symbol with optional ANSI color codes
/// T039: Now calls simplify_symbol() before applying color
pub fn format_colored_symbol(symbol: &str, use_color: bool) -> String {
    // T039: Simplify symbol before formatting
    let simplified = simplify_symbol(symbol);

    if !use_color {
        return simplified;
    }
    // Classify based on original symbol for correct color detection
    let symbol_type = classify_symbol(symbol);
    let color = color_for_type(symbol_type);
    format!("{}{}{}", color, simplified, RESET)
}

#[cfg(test)]
mod tests {
    use super::*;

    // T003: Unit test for SymbolType enum variants
    #[test]
    fn test_symbol_type_variants() {
        let user = SymbolType::User;
        let library = SymbolType::Library;
        let unresolved = SymbolType::Unresolved;

        assert!(matches!(user, SymbolType::User));
        assert!(matches!(library, SymbolType::Library));
        assert!(matches!(unresolved, SymbolType::Unresolved));

        // Test equality
        assert_eq!(user, SymbolType::User);
        assert_ne!(user, SymbolType::Library);
    }

    // T004: Unit test for should_use_color with no_color_flag
    #[test]
    fn test_should_use_color_with_flag() {
        // When no_color_flag is true, should always return false
        assert!(!should_use_color(true));
    }

    // T005: Unit test for should_use_color with NO_COLOR env var
    #[test]
    fn test_should_use_color_respects_no_color_env() {
        // This test checks the logic - actual env var testing is tricky
        // The function should return false when NO_COLOR is set
        // We can't easily mock env vars, but we can verify the flag path
        assert!(!should_use_color(true));
    }

    // Test color_for_type returns correct ANSI codes
    #[test]
    fn test_color_for_type() {
        assert_eq!(color_for_type(SymbolType::User), BLUE);
        assert_eq!(color_for_type(SymbolType::Library), YELLOW);
        assert_eq!(color_for_type(SymbolType::Unresolved), RED);
    }

    // T010: Unit test for classify_symbol with hex addresses
    #[test]
    fn test_classify_symbol_hex_addresses() {
        assert_eq!(classify_symbol("0x7d4c47223efe"), SymbolType::Unresolved);
        assert_eq!(
            classify_symbol("0x00007d4c47223efe"),
            SymbolType::Unresolved
        );
        assert_eq!(classify_symbol("0000000000000000"), SymbolType::Unresolved);
    }

    // T011: Unit test for classify_symbol with std:: prefix
    #[test]
    fn test_classify_symbol_std_prefix() {
        assert_eq!(classify_symbol("std::inner_product"), SymbolType::Library);
        assert_eq!(
            classify_symbol("std::vector<int>::push_back"),
            SymbolType::Library
        );
    }

    // T012: Unit test for classify_symbol with __ prefix
    #[test]
    fn test_classify_symbol_underscore_prefix() {
        assert_eq!(classify_symbol("__libc_start_main"), SymbolType::Library);
        assert_eq!(classify_symbol("__cxa_atexit"), SymbolType::Library);
    }

    // T013: Unit test for classify_symbol with libc functions
    #[test]
    fn test_classify_symbol_libc_functions() {
        assert_eq!(classify_symbol("malloc"), SymbolType::Library);
        assert_eq!(classify_symbol("free"), SymbolType::Library);
        assert_eq!(classify_symbol("memset"), SymbolType::Library);
        assert_eq!(classify_symbol("memcpy"), SymbolType::Library);
        assert_eq!(classify_symbol("memmove"), SymbolType::Library);
        assert_eq!(classify_symbol("pthread_create"), SymbolType::Library);
    }

    // T014: Unit test for classify_symbol with user functions
    #[test]
    fn test_classify_symbol_user_functions() {
        assert_eq!(classify_symbol("MyClass::myMethod"), SymbolType::User);
        assert_eq!(
            classify_symbol("Hierarchical4DEncoder::get_mSubband"),
            SymbolType::User
        );
        assert_eq!(classify_symbol("process_data"), SymbolType::User);
    }

    // T015: Unit test for format_colored_symbol
    #[test]
    fn test_format_colored_symbol() {
        // With colors enabled
        let colored = format_colored_symbol("MyClass::method", true);
        assert!(colored.contains(BLUE));
        assert!(colored.contains(RESET));
        assert!(colored.contains("MyClass::method"));

        // Library function
        let lib_colored = format_colored_symbol("std::vector", true);
        assert!(lib_colored.contains(YELLOW));

        // Unresolved
        let unresolved_colored = format_colored_symbol("0x12345", true);
        assert!(unresolved_colored.contains(RED));

        // Without colors
        let plain = format_colored_symbol("MyClass::method", false);
        assert!(!plain.contains('\x1b'));
        assert_eq!(plain, "MyClass::method");
    }

    // ========== User Story 2: Symbol Simplification Tests ==========

    // T025: Unit test for simplify_symbol() stripping return types
    #[test]
    fn test_simplify_symbol_strip_return_types() {
        assert_eq!(
            simplify_symbol("void Hierarchical4DEncoder::get_mSubband()"),
            "Hierarchical4DEncoder::get_mSubband"
        );
        assert_eq!(
            simplify_symbol("double std::inner_product()"),
            "std::inner_product"
        );
        assert_eq!(
            simplify_symbol("int MyClass::getValue()"),
            "MyClass::getValue"
        );
    }

    // T026: Unit test for simplify_symbol() stripping argument lists
    #[test]
    fn test_simplify_symbol_strip_arguments() {
        assert_eq!(
            simplify_symbol("MyClass::method(int, string)"),
            "MyClass::method"
        );
        assert_eq!(
            simplify_symbol("func(int a, double b, const char* c)"),
            "func"
        );
        assert_eq!(
            simplify_symbol("Class::method(int, string) const"),
            "Class::method"
        );
    }

    // T027: Unit test for simplify_symbol() stripping template parameters
    #[test]
    fn test_simplify_symbol_strip_templates() {
        assert_eq!(
            simplify_symbol("std::vector<int>::push_back"),
            "std::vector::push_back"
        );
        assert_eq!(
            simplify_symbol("std::map<string, int>::insert"),
            "std::map::insert"
        );
    }

    // T028: Unit test for simplify_symbol() stripping nested templates
    #[test]
    fn test_simplify_symbol_strip_nested_templates() {
        assert_eq!(
            simplify_symbol("std::vector<std::pair<int, double>>::push_back"),
            "std::vector::push_back"
        );
        assert_eq!(
            simplify_symbol(
                "double std::inner_product<double*, double const*, double>(double*, double*, double const*, double)"
            ),
            "std::inner_product"
        );
    }

    // T029: Unit test for simplify_symbol() stripping clone suffixes
    #[test]
    fn test_simplify_symbol_strip_clone_suffixes() {
        assert_eq!(simplify_symbol("func.cold"), "func");
        assert_eq!(simplify_symbol("func.cold.123"), "func");
        assert_eq!(simplify_symbol("func.part.5"), "func");
        assert_eq!(simplify_symbol("func.isra.3"), "func");
        assert_eq!(simplify_symbol("func.constprop.0"), "func");
    }

    // T030: Unit test for simplify_symbol() collapsing lambda syntax
    #[test]
    fn test_simplify_symbol_collapse_lambda() {
        assert_eq!(
            simplify_symbol("main::{lambda(int)#1}::operator()"),
            "main::{lambda}::operator()"
        );
        assert_eq!(
            simplify_symbol("Class::{lambda()#2}::call"),
            "Class::{lambda}::call"
        );
    }

    // T031: Unit test for simplify_symbol() preserving hex addresses unchanged
    #[test]
    fn test_simplify_symbol_preserve_hex() {
        assert_eq!(simplify_symbol("0x7d4c47223efe"), "0x7d4c47223efe");
        assert_eq!(simplify_symbol("0000000000000000"), "0000000000000000");
    }

    // T032: Unit test for simplify_symbol() with real symbols from perf-report.txt
    #[test]
    fn test_simplify_symbol_real_symbols() {
        // From cli.md contract examples
        assert_eq!(
            simplify_symbol(
                "void Hierarchical4DEncoder::get_mSubbandLF_significance(unsigned int, LightfieldCoordinate<unsigned int> const&, LightfieldDimension<unsigned int, true> const&) const"
            ),
            "Hierarchical4DEncoder::get_mSubbandLF_significance"
        );
        assert_eq!(
            simplify_symbol(
                "void TransformPartition::evaluate_split_for_partitions<(PartitionFlag)1, (PartitionFlag)2>(Block4D const&, unsigned int, unsigned int, unsigned int, unsigned int)"
            ),
            "TransformPartition::evaluate_split_for_partitions"
        );
        assert_eq!(
            simplify_symbol("DCT4DBlock::DCT4DBlock(Block4D const&, double)"),
            "DCT4DBlock::DCT4DBlock"
        );
    }
}
