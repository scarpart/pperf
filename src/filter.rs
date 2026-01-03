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

#[cfg(test)]
mod tests {
    use super::*;

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
}
