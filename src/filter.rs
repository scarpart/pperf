use std::fs;
use std::path::Path;

use crate::PperfError;
use crate::parser::PerfEntry;

pub fn filter_entries(entries: &[PerfEntry], targets: &[String]) -> Vec<PerfEntry> {
    if targets.is_empty() {
        return entries.to_vec();
    }

    entries
        .iter()
        .filter(|entry| targets.iter().any(|t| matches_pattern(&entry.symbol, t)))
        .cloned()
        .collect()
}

pub fn matches_pattern(symbol: &str, pattern: &str) -> bool {
    symbol.contains(pattern)
}

/// Parse a target file containing function signatures (one per line).
/// Ignores empty lines, whitespace-only lines, and comment lines (starting with #).
/// Trims leading/trailing whitespace from each signature.
pub fn parse_target_file(path: &Path) -> Result<Vec<String>, PperfError> {
    let content = fs::read_to_string(path)
        .map_err(|_| PperfError::TargetFileNotFound(path.display().to_string()))?;

    let signatures: Vec<String> = content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.to_string())
        .collect();

    if signatures.is_empty() {
        return Err(PperfError::EmptyTargetFile);
    }

    Ok(signatures)
}

/// Detect signatures that don't match any entries.
/// Returns a list of signatures that have no matches in the entries.
pub fn detect_unmatched_targets(entries: &[PerfEntry], signatures: &[String]) -> Vec<String> {
    signatures
        .iter()
        .filter(|sig| !entries.iter().any(|e| e.symbol == sig.as_str()))
        .cloned()
        .collect()
}

/// Validate that each signature matches exactly one unique symbol.
/// Returns Ok(()) if all signatures are unambiguous, or AmbiguousTarget error
/// for the first signature that matches multiple distinct symbols.
pub fn validate_unique_matches(
    entries: &[PerfEntry],
    signatures: &[String],
) -> Result<(), PperfError> {
    use std::collections::HashSet;

    for sig in signatures {
        // Collect all distinct symbols that exactly match this signature
        let matching_symbols: HashSet<&str> = entries
            .iter()
            .filter(|e| e.symbol == *sig)
            .map(|e| e.symbol.as_str())
            .collect();

        // With exact matching, we should have at most 1 unique symbol
        // (could have 0 if no match, which is handled by detect_unmatched_targets)
        if matching_symbols.len() > 1 {
            return Err(PperfError::AmbiguousTarget {
                signature: sig.clone(),
                matches: matching_symbols.into_iter().map(String::from).collect(),
            });
        }
    }

    Ok(())
}

/// Filter entries using exact signature matching (equality).
/// Returns entries where the raw symbol exactly equals one of the target signatures.
pub fn filter_entries_exact(entries: &[PerfEntry], signatures: &[String]) -> Vec<PerfEntry> {
    if signatures.is_empty() {
        return entries.to_vec();
    }

    entries
        .iter()
        .filter(|entry| signatures.iter().any(|sig| entry.symbol == sig.as_str()))
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        write!(file, "{}", content).unwrap();
        file
    }

    // T007: Test parse_target_file returns signatures from valid file
    #[test]
    fn test_parse_target_file_returns_signatures() {
        let file =
            create_temp_file("DCT4DBlock::DCT4DBlock(Block4D const&, double)\nOther::func()");
        let result = parse_target_file(file.path()).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "DCT4DBlock::DCT4DBlock(Block4D const&, double)");
        assert_eq!(result[1], "Other::func()");
    }

    // T008: Test parse_target_file ignores comment lines starting with #
    #[test]
    fn test_parse_target_file_ignores_comments() {
        let file = create_temp_file("# This is a comment\nDCT4DBlock::func()\n# Another comment");
        let result = parse_target_file(file.path()).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "DCT4DBlock::func()");
    }

    // T009: Test parse_target_file ignores empty and whitespace-only lines
    #[test]
    fn test_parse_target_file_ignores_empty_lines() {
        let file = create_temp_file("DCT4DBlock::func()\n\n   \n\t\nOther::func()");
        let result = parse_target_file(file.path()).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0], "DCT4DBlock::func()");
        assert_eq!(result[1], "Other::func()");
    }

    // T010: Test parse_target_file trims leading/trailing whitespace from signatures
    #[test]
    fn test_parse_target_file_trims_whitespace() {
        let file = create_temp_file("  DCT4DBlock::func()  \n\tOther::func()\t");
        let result = parse_target_file(file.path()).unwrap();
        assert_eq!(result[0], "DCT4DBlock::func()");
        assert_eq!(result[1], "Other::func()");
    }

    // T011: Test parse_target_file returns EmptyTargetFile error for file with only comments
    #[test]
    fn test_parse_target_file_empty_file_error() {
        let file = create_temp_file("# Only comments\n# Nothing else\n   ");
        let result = parse_target_file(file.path());
        assert!(matches!(result, Err(PperfError::EmptyTargetFile)));
    }

    #[test]
    fn test_parse_target_file_not_found() {
        let result = parse_target_file(Path::new("/nonexistent/path/targets.txt"));
        assert!(matches!(result, Err(PperfError::TargetFileNotFound(_))));
    }

    // T013: Test filter_entries_exact matches entry when signature is exact substring of symbol
    #[test]
    fn test_filter_entries_exact_matches_substring() {
        let entries = vec![
            PerfEntry {
                children_pct: 90.0,
                self_pct: 1.0,
                symbol: "DCT4DBlock::DCT4DBlock(Block4D const&, double)".to_string(),
            },
            PerfEntry {
                children_pct: 50.0,
                self_pct: 5.0,
                symbol: "Other::function()".to_string(),
            },
        ];
        let signatures = vec!["DCT4DBlock::DCT4DBlock(Block4D const&, double)".to_string()];
        let filtered = filter_entries_exact(&entries, &signatures);
        assert_eq!(filtered.len(), 1);
        assert!(filtered[0].symbol.contains("DCT4DBlock"));
    }

    // T014: Test filter_entries_exact returns empty for non-matching signature
    #[test]
    fn test_filter_entries_exact_no_match() {
        let entries = vec![PerfEntry {
            children_pct: 90.0,
            self_pct: 1.0,
            symbol: "DCT4DBlock::DCT4DBlock(Block4D const&, double)".to_string(),
        }];
        let signatures = vec!["NonExistent::function()".to_string()];
        let filtered = filter_entries_exact(&entries, &signatures);
        assert!(filtered.is_empty());
    }

    #[test]
    fn test_filter_entries_exact_multiple_signatures() {
        let entries = vec![
            PerfEntry {
                children_pct: 90.0,
                self_pct: 1.0,
                symbol: "DCT4DBlock::DCT4DBlock(Block4D const&, double)".to_string(),
            },
            PerfEntry {
                children_pct: 50.0,
                self_pct: 5.0,
                symbol: "Other::function()".to_string(),
            },
            PerfEntry {
                children_pct: 30.0,
                self_pct: 3.0,
                symbol: "Unrelated::method()".to_string(),
            },
        ];
        let signatures = vec![
            "DCT4DBlock::DCT4DBlock(Block4D const&, double)".to_string(),
            "Other::function()".to_string(),
        ];
        let filtered = filter_entries_exact(&entries, &signatures);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_matches_pattern_exact() {
        assert!(matches_pattern("DCT4DBlock", "DCT4DBlock"));
    }

    #[test]
    fn test_matches_pattern_prefix() {
        assert!(matches_pattern("DCT4DBlock::transform", "DCT4D"));
        assert!(matches_pattern("std::inner_product", "std::"));
    }

    #[test]
    fn test_matches_pattern_substring() {
        // Key feature: match anywhere in the symbol
        assert!(matches_pattern(
            "Hierarchical4DEncoder::get_mSubband",
            "get_mSubband"
        ));
        assert!(matches_pattern(
            "std::inner_product<double>",
            "inner_product"
        ));
        assert!(matches_pattern(
            "Block4D::get_linear_position",
            "linear_position"
        ));
    }

    #[test]
    fn test_matches_pattern_no_match() {
        assert!(!matches_pattern("Block4D", "DCT4D"));
        assert!(!matches_pattern("transform", "mSubband"));
    }

    #[test]
    fn test_filter_entries_single_target() {
        let entries = vec![
            PerfEntry {
                children_pct: 90.0,
                self_pct: 1.0,
                symbol: "DCT4DBlock::new".to_string(),
            },
            PerfEntry {
                children_pct: 50.0,
                self_pct: 5.0,
                symbol: "Block4D::get".to_string(),
            },
            PerfEntry {
                children_pct: 30.0,
                self_pct: 3.0,
                symbol: "DCT4DBlock::transform".to_string(),
            },
        ];
        let targets = vec!["DCT4D".to_string()];
        let filtered = filter_entries(&entries, &targets);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|e| e.symbol.starts_with("DCT4D")));
    }

    #[test]
    fn test_filter_entries_multiple_targets() {
        let entries = vec![
            PerfEntry {
                children_pct: 90.0,
                self_pct: 1.0,
                symbol: "DCT4DBlock::new".to_string(),
            },
            PerfEntry {
                children_pct: 50.0,
                self_pct: 5.0,
                symbol: "Block4D::get".to_string(),
            },
            PerfEntry {
                children_pct: 30.0,
                self_pct: 3.0,
                symbol: "std::sort".to_string(),
            },
        ];
        let targets = vec!["DCT4D".to_string(), "std::".to_string()];
        let filtered = filter_entries(&entries, &targets);

        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|e| e.symbol.starts_with("DCT4D")));
        assert!(filtered.iter().any(|e| e.symbol.starts_with("std::")));
    }

    #[test]
    fn test_filter_entries_empty_targets() {
        let entries = vec![
            PerfEntry {
                children_pct: 90.0,
                self_pct: 1.0,
                symbol: "foo".to_string(),
            },
            PerfEntry {
                children_pct: 50.0,
                self_pct: 5.0,
                symbol: "bar".to_string(),
            },
        ];
        let targets: Vec<String> = vec![];
        let filtered = filter_entries(&entries, &targets);

        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_filter_entries_no_matches() {
        let entries = vec![
            PerfEntry {
                children_pct: 90.0,
                self_pct: 1.0,
                symbol: "foo".to_string(),
            },
            PerfEntry {
                children_pct: 50.0,
                self_pct: 5.0,
                symbol: "bar".to_string(),
            },
        ];
        let targets = vec!["NonExistent".to_string()];
        let filtered = filter_entries(&entries, &targets);

        assert!(filtered.is_empty());
    }

    // T020: Test validate_unique_matches returns Ok when each signature matches exactly one unique symbol
    #[test]
    fn test_validate_unique_matches_ok_single_match() {
        let entries = vec![
            PerfEntry {
                children_pct: 90.0,
                self_pct: 1.0,
                symbol: "DCT4DBlock::DCT4DBlock(Block4D const&, double)".to_string(),
            },
            PerfEntry {
                children_pct: 50.0,
                self_pct: 5.0,
                symbol: "Other::function()".to_string(),
            },
        ];
        let signatures = vec!["DCT4DBlock::DCT4DBlock(Block4D const&, double)".to_string()];
        let result = validate_unique_matches(&entries, &signatures);
        assert!(result.is_ok());
    }

    // T020: Test validate_unique_matches returns Ok for multiple entries with same symbol
    #[test]
    fn test_validate_unique_matches_ok_multiple_entries_same_symbol() {
        // Same symbol appearing multiple times (not ambiguous - same function)
        let entries = vec![
            PerfEntry {
                children_pct: 90.0,
                self_pct: 1.0,
                symbol: "DCT4DBlock::transform()".to_string(),
            },
            PerfEntry {
                children_pct: 50.0,
                self_pct: 5.0,
                symbol: "DCT4DBlock::transform()".to_string(),
            },
        ];
        let signatures = vec!["DCT4DBlock::transform()".to_string()];
        let result = validate_unique_matches(&entries, &signatures);
        assert!(result.is_ok());
    }

    // T021: Test validate_unique_matches returns Ok with exact signatures (no ambiguity possible)
    // With exact matching, partial signatures don't match anything - they're just unmatched.
    // Ambiguity can only occur if somehow identical symbols appear, which validate handles.
    #[test]
    fn test_validate_unique_matches_exact_signatures() {
        let entries = vec![
            PerfEntry {
                children_pct: 90.0,
                self_pct: 1.0,
                symbol: "DCT4DBlock::DCT4DBlock(Block4D const&, double)".to_string(),
            },
            PerfEntry {
                children_pct: 50.0,
                self_pct: 5.0,
                symbol: "DCT4DBlock::inverse(Block4D&) const".to_string(),
            },
        ];
        // Exact signatures - each matches exactly one symbol
        let signatures = vec![
            "DCT4DBlock::DCT4DBlock(Block4D const&, double)".to_string(),
            "DCT4DBlock::inverse(Block4D&) const".to_string(),
        ];
        let result = validate_unique_matches(&entries, &signatures);
        assert!(result.is_ok());
    }

    // T021: Test that partial signatures don't cause ambiguity - they just don't match
    #[test]
    fn test_validate_unique_matches_partial_signature_no_match() {
        let entries = vec![
            PerfEntry {
                children_pct: 90.0,
                self_pct: 1.0,
                symbol: "DCT4DBlock::DCT4DBlock(Block4D const&, double)".to_string(),
            },
            PerfEntry {
                children_pct: 50.0,
                self_pct: 5.0,
                symbol: "DCT4DBlock::inverse(Block4D&) const".to_string(),
            },
        ];
        // Partial signature - won't match any symbol exactly (0 matches, not ambiguous)
        let signatures = vec!["DCT4DBlock".to_string()];
        let result = validate_unique_matches(&entries, &signatures);
        // With exact matching, partial signature matches 0 symbols, so Ok (not ambiguous)
        assert!(result.is_ok());
    }

    // T031: Test detect_unmatched_targets returns empty vec when all signatures match exactly
    #[test]
    fn test_detect_unmatched_targets_all_match() {
        let entries = vec![
            PerfEntry {
                children_pct: 90.0,
                self_pct: 1.0,
                symbol: "DCT4DBlock::DCT4DBlock(Block4D const&, double)".to_string(),
            },
            PerfEntry {
                children_pct: 50.0,
                self_pct: 5.0,
                symbol: "Other::function()".to_string(),
            },
        ];
        // Use exact signatures that match the symbols
        let signatures = vec!["DCT4DBlock::DCT4DBlock(Block4D const&, double)".to_string()];
        let unmatched = detect_unmatched_targets(&entries, &signatures);
        assert!(unmatched.is_empty());
    }

    // T032: Test detect_unmatched_targets returns list of unmatched signatures
    #[test]
    fn test_detect_unmatched_targets_some_unmatched() {
        let entries = vec![PerfEntry {
            children_pct: 90.0,
            self_pct: 1.0,
            symbol: "DCT4DBlock::DCT4DBlock(Block4D const&, double)".to_string(),
        }];
        let signatures = vec![
            "DCT4DBlock::DCT4DBlock(Block4D const&, double)".to_string(), // matches
            "NonExistent::function()".to_string(),                        // no match
            "Another::missing()".to_string(),                             // no match
        ];
        let unmatched = detect_unmatched_targets(&entries, &signatures);
        assert_eq!(unmatched.len(), 2);
        assert!(unmatched.contains(&"NonExistent::function()".to_string()));
        assert!(unmatched.contains(&"Another::missing()".to_string()));
    }

    // T032: Test detect_unmatched_targets returns all signatures when none match
    #[test]
    fn test_detect_unmatched_targets_none_match() {
        let entries = vec![PerfEntry {
            children_pct: 90.0,
            self_pct: 1.0,
            symbol: "DCT4DBlock::DCT4DBlock(Block4D const&, double)".to_string(),
        }];
        let signatures = vec![
            "NonExistent::function()".to_string(),
            "Another::missing()".to_string(),
        ];
        let unmatched = detect_unmatched_targets(&entries, &signatures);
        assert_eq!(unmatched.len(), 2);
    }
}
