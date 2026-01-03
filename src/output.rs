use crate::hierarchy::HierarchyEntry;
use crate::parser::PerfEntry;
use crate::symbol::format_colored_symbol;

/// T021: Format table with optional color support
pub fn format_table(entries: &[PerfEntry], use_color: bool) -> String {
    let mut output = String::new();
    output.push_str("Children%   Self%  Function\n");

    for entry in entries {
        let symbol = truncate_symbol(&entry.symbol, 100);
        // T022: Apply colors to each entry's symbol
        let colored_symbol = format_colored_symbol(&symbol, use_color);
        output.push_str(&format!(
            "{:>8.2}  {:>6.2}  {}\n",
            entry.children_pct, entry.self_pct, colored_symbol
        ));
    }

    output
}

pub fn truncate_symbol(symbol: &str, max_len: usize) -> String {
    if symbol.len() <= max_len {
        symbol.to_string()
    } else {
        format!("{}...", &symbol[..max_len - 3])
    }
}

/// T047: Format hierarchy table with indented callees.
/// Callers show their callees indented with relative percentages.
/// Standalone entries show adjusted absolute percentages.
pub fn format_hierarchy_table(entries: &[HierarchyEntry], use_color: bool) -> String {
    let mut output = String::new();
    output.push_str("Children%   Self%  Function\n");

    for entry in entries {
        // First, display the caller with original/adjusted percentages
        let symbol = truncate_symbol(&entry.symbol, 100);
        let colored_symbol = format_colored_symbol(&symbol, use_color);

        // If this entry has callees, show original percentage
        // Otherwise show adjusted percentage
        let children_pct = if entry.is_caller {
            entry.original_children_pct
        } else {
            entry.adjusted_children_pct
        };

        output.push_str(&format!(
            "{:>8.2}  {:>6.2}  {}\n",
            children_pct, entry.original_self_pct, colored_symbol
        ));

        // Then display indented callees with relative percentages (4-space indent)
        for callee in &entry.callees {
            let callee_symbol = truncate_symbol(&callee.callee, 96);
            let colored_callee = format_colored_symbol(&callee_symbol, use_color);
            output.push_str(&format!(
                "{:>8.2}  {:>6.2}      {}\n",
                callee.relative_pct, 0.0, colored_callee
            ));
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use crate::parser::PerfEntry;

    #[test]
    fn test_format_table_aligned_output() {
        let entries = vec![
            PerfEntry {
                children_pct: 90.74,
                self_pct: 0.00,
                symbol: "parallel_for_with_progress".to_string(),
            },
            PerfEntry {
                children_pct: 71.80,
                self_pct: 11.94,
                symbol: "get_mSubbandLF_significance".to_string(),
            },
            PerfEntry {
                children_pct: 7.45,
                self_pct: 7.45,
                symbol: "std::inner_product".to_string(),
            },
        ];

        let output = super::format_table(&entries, false);

        let lines: Vec<&str> = output.lines().collect();
        assert!(!lines.is_empty(), "Output should not be empty");
        let header = lines[0];
        assert!(
            header.contains("Children%"),
            "Header should contain 'Children%'"
        );
        assert!(header.contains("Self%"), "Header should contain 'Self%'");
        assert!(
            header.contains("Function"),
            "Header should contain 'Function'"
        );

        assert!(lines.len() >= 4, "Should have header + 3 data rows");

        let first_data_row = lines[1];
        assert!(
            first_data_row.contains("90.74"),
            "First row should contain children_pct 90.74"
        );
        assert!(
            first_data_row.contains("0.00"),
            "First row should contain self_pct 0.00"
        );

        let second_data_row = lines[2];
        assert!(
            second_data_row.contains("71.80"),
            "Second row should contain children_pct 71.80"
        );
        assert!(
            second_data_row.contains("11.94"),
            "Second row should contain self_pct 11.94"
        );

        assert!(
            output.contains("parallel_for_with_progress"),
            "Output should contain first function name"
        );
        assert!(
            output.contains("get_mSubbandLF_significance"),
            "Output should contain second function name"
        );
        assert!(
            output.contains("std::inner_product"),
            "Output should contain third function name"
        );
    }

    #[test]
    fn test_truncate_symbol_short() {
        let short = "short_name";
        assert_eq!(super::truncate_symbol(short, 100), "short_name");
    }

    #[test]
    fn test_truncate_symbol_long() {
        let long = "a".repeat(150);
        let truncated = super::truncate_symbol(&long, 100);
        assert_eq!(truncated.len(), 100);
        assert!(truncated.ends_with("..."));
    }
}
