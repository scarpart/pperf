use crate::hierarchy::{CallRelation, HierarchyEntry};
use crate::parser::PerfEntry;
use crate::symbol::{format_colored_symbol, simplify_symbol};
use std::collections::{HashMap, HashSet};

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

/// Format hierarchy table with multi-level nested callees.
/// Supports recursive nesting: A → B → C displayed with increasing indentation.
/// Tracks "consumed" relationships to avoid repetition.
pub fn format_hierarchy_table(entries: &[HierarchyEntry], use_color: bool) -> String {
    let mut output = String::new();
    output.push_str("Children%   Self%  Function\n");

    // Build a map of caller symbol → callees for recursive lookup
    let mut callee_map: HashMap<String, Vec<&CallRelation>> = HashMap::new();
    for entry in entries {
        if !entry.callees.is_empty() {
            callee_map.insert(entry.symbol.clone(), entry.callees.iter().collect());
        }
    }

    // Build a map of callee simplified symbol → entry symbol (for recursive lookup)
    // This helps us find the entry for a callee to check if it has its own callees
    let mut callee_to_entry: HashMap<String, String> = HashMap::new();
    for entry in entries {
        for callee in &entry.callees {
            // Map the callee's simplified name to the entry that has it as a callee
            // We need to find entries whose symbol contains this callee
            for e in entries {
                if e.symbol.contains(&callee.callee) {
                    callee_to_entry.insert(callee.callee.clone(), e.symbol.clone());
                    break;
                }
            }
        }
    }

    // Track consumed caller→callee pairs (displayed under a parent)
    let mut consumed: HashSet<(String, String)> = HashSet::new();

    // First pass: display root callers with their nested callees
    for entry in entries {
        if !entry.is_caller {
            continue; // Skip non-callers in first pass
        }

        // Display root caller
        let symbol = truncate_symbol(&entry.symbol, 100);
        let colored_symbol = format_colored_symbol(&symbol, use_color);
        output.push_str(&format!(
            "{:>8.2}  {:>6.2}  {}\n",
            entry.original_children_pct, entry.original_self_pct, colored_symbol
        ));

        // Track visited callees to prevent infinite recursion (using simplified symbols)
        let mut visited: HashSet<String> = HashSet::new();
        visited.insert(simplify_symbol(&entry.symbol));

        // Recursively display callees with multi-level indentation
        display_callees_recursive(
            &entry.symbol,
            &callee_map,
            &callee_to_entry,
            &mut consumed,
            &mut visited,
            &mut output,
            1, // Start at indent level 1
            use_color,
        );
    }

    // Second pass: display standalone entries (not callers, or callers with remaining callees)
    for entry in entries {
        if entry.is_caller {
            // Check if all its callees were consumed
            let all_consumed = entry
                .callees
                .iter()
                .all(|c| consumed.contains(&(entry.symbol.clone(), c.callee.clone())));
            if all_consumed {
                continue; // Skip, all relationships already shown
            }
            // Show as standalone with any unconsumed callees
            let symbol = truncate_symbol(&entry.symbol, 100);
            let colored_symbol = format_colored_symbol(&symbol, use_color);
            output.push_str(&format!(
                "{:>8.2}  {:>6.2}  {}\n",
                entry.adjusted_children_pct, entry.original_self_pct, colored_symbol
            ));

            // Track visited callees to prevent infinite recursion (using simplified symbols)
            let mut visited: HashSet<String> = HashSet::new();
            visited.insert(simplify_symbol(&entry.symbol));

            // Show only unconsumed callees
            for callee in &entry.callees {
                if !consumed.contains(&(entry.symbol.clone(), callee.callee.clone())) {
                    display_callees_recursive(
                        &entry.symbol,
                        &callee_map,
                        &callee_to_entry,
                        &mut consumed,
                        &mut visited,
                        &mut output,
                        1,
                        use_color,
                    );
                    break; // Only need to call once, it handles all
                }
            }
        } else {
            // Pure standalone (not a caller)
            let symbol = truncate_symbol(&entry.symbol, 100);
            let colored_symbol = format_colored_symbol(&symbol, use_color);
            output.push_str(&format!(
                "{:>8.2}  {:>6.2}  {}\n",
                entry.adjusted_children_pct, entry.original_self_pct, colored_symbol
            ));
        }
    }

    output
}

/// Recursively display callees with increasing indentation levels.
/// Each level adds 4 spaces of indentation.
/// Uses visited set to prevent infinite recursion for recursive functions.
#[allow(clippy::too_many_arguments)]
fn display_callees_recursive(
    caller: &str,
    callee_map: &HashMap<String, Vec<&CallRelation>>,
    callee_to_entry: &HashMap<String, String>,
    consumed: &mut HashSet<(String, String)>,
    visited: &mut HashSet<String>,
    output: &mut String,
    indent_level: usize,
    use_color: bool,
) {
    // Find callees for this caller
    let Some(callees) = callee_map.get(caller) else {
        return;
    };

    for callee_rel in callees {
        // Skip if already consumed
        if consumed.contains(&(caller.to_string(), callee_rel.callee.clone())) {
            continue;
        }

        // Mark as consumed
        consumed.insert((caller.to_string(), callee_rel.callee.clone()));

        // Calculate indentation (4 spaces per level)
        let indent = "    ".repeat(indent_level);
        let max_symbol_len = 100 - (indent_level * 4);
        let callee_symbol = truncate_symbol(&callee_rel.callee, max_symbol_len);
        let colored_callee = format_colored_symbol(&callee_symbol, use_color);

        output.push_str(&format!(
            "{:>8.2}  {:>6.2}  {}{}\n",
            callee_rel.relative_pct, 0.0, indent, colored_callee
        ));

        // Check if this callee is also a caller (has its own callees)
        // Use the callee_to_entry map to find the corresponding entry
        if let Some(entry_symbol) = callee_to_entry.get(&callee_rel.callee) {
            // Use simplified symbol to detect recursion (handles lambda variants)
            let simplified = simplify_symbol(entry_symbol);

            // Skip if already visited (prevents infinite recursion)
            if visited.contains(&simplified) {
                continue;
            }

            // Mark as visited before recursing
            visited.insert(simplified);

            // Only recurse if this entry has callees in the callee_map
            if callee_map.contains_key(entry_symbol) {
                display_callees_recursive(
                    entry_symbol,
                    callee_map,
                    callee_to_entry,
                    consumed,
                    visited,
                    output,
                    indent_level + 1,
                    use_color,
                );
            }
        }
    }
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
