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
/// Uses context-specific relations for accurate path percentages.
/// Calculates remainder contributions for standalone entries.
pub fn format_hierarchy_table(
    entries: &[HierarchyEntry],
    all_relations: &[CallRelation],
    use_color: bool,
) -> String {
    let mut output = String::new();
    output.push_str("Children%   Self%  Function\n");

    // Build context-specific callee map: (root_caller, caller) → callees
    // For root caller A's tree, when B→C has context_root = Some(A), store under (A, B)
    let mut context_callee_map: HashMap<(String, String), Vec<&CallRelation>> = HashMap::new();
    for r in all_relations {
        if let Some(ref root) = r.context_root {
            context_callee_map
                .entry((root.clone(), r.caller.clone()))
                .or_default()
                .push(r);
        }
    }

    // Build direct callee map for root callers (context_root = None)
    let mut direct_callee_map: HashMap<String, Vec<&CallRelation>> = HashMap::new();
    for r in all_relations {
        if r.context_root.is_none() {
            direct_callee_map
                .entry(r.caller.clone())
                .or_default()
                .push(r);
        }
    }

    // Build entry lookup by simplified symbol
    let mut entry_by_simplified: HashMap<String, &HierarchyEntry> = HashMap::new();
    for entry in entries {
        let simplified = simplify_symbol(&entry.symbol);
        entry_by_simplified.insert(simplified, entry);
    }

    // Collect all callees from overall relations (to identify root vs intermediate callers)
    let all_callees: HashSet<String> = entries
        .iter()
        .flat_map(|e| e.callees.iter().map(|c| c.callee.clone()))
        .collect();

    // Track consumed absolute contributions per callee
    // Key: callee simplified symbol, Value: total absolute % consumed
    let mut consumed_absolute: HashMap<String, f64> = HashMap::new();

    // First pass: display ROOT callers only
    for entry in entries {
        if !entry.is_caller {
            continue;
        }

        let simplified = simplify_symbol(&entry.symbol);
        if all_callees.contains(&simplified) {
            continue; // Not a root caller
        }

        // Display root caller with original percentage
        let symbol = truncate_symbol(&entry.symbol, 100);
        let colored_symbol = format_colored_symbol(&symbol, use_color);
        output.push_str(&format!(
            "{:>8.2}  {:>6.2}  {}\n",
            entry.original_children_pct, entry.original_self_pct, colored_symbol
        ));

        // Display direct callees of this root, using context-specific relations for deeper levels
        let mut visited: HashSet<String> = HashSet::new();
        visited.insert(simplified.clone());

        display_callees_with_context(
            &simplified,  // Use simplified for lookup
            &simplified,
            &direct_callee_map,
            &context_callee_map,
            &entry_by_simplified,
            &mut consumed_absolute,
            &mut visited,
            &mut output,
            1,
            use_color,
        );
    }

    // Second pass: display standalone entries with remainder callees
    for entry in entries {
        let simplified = simplify_symbol(&entry.symbol);
        let is_root_caller = entry.is_caller && !all_callees.contains(&simplified);
        if is_root_caller {
            continue; // Already shown
        }

        // Show entry with adjusted percentage
        let symbol = truncate_symbol(&entry.symbol, 100);
        let colored_symbol = format_colored_symbol(&symbol, use_color);
        output.push_str(&format!(
            "{:>8.2}  {:>6.2}  {}\n",
            entry.adjusted_children_pct, entry.original_self_pct, colored_symbol
        ));

        // If this entry has callees, show remainder callees (overall - consumed)
        if entry.is_caller {
            for callee in &entry.callees {
                let callee_simplified = simplify_symbol(&callee.callee);
                let consumed = consumed_absolute.get(&callee_simplified).copied().unwrap_or(0.0);
                let overall_absolute = callee.absolute_pct;
                let remainder = overall_absolute - consumed;

                if remainder > 0.01 {
                    // Calculate relative % to this entry's standalone time
                    // remainder is absolute, entry.adjusted_children_pct is the standalone base
                    let relative_to_standalone = if entry.adjusted_children_pct > 0.0 {
                        remainder / entry.adjusted_children_pct * 100.0
                    } else {
                        0.0
                    };

                    // Display the remainder
                    let indent = "    ";
                    let callee_symbol = truncate_symbol(&callee.callee, 96);
                    let colored_callee = format_colored_symbol(&callee_symbol, use_color);
                    output.push_str(&format!(
                        "{:>8.2}  {:>6.2}  {}{}\n",
                        relative_to_standalone, 0.0, indent, colored_callee
                    ));
                }
            }
        }
    }

    output
}

/// Display callees recursively using context-specific relations.
#[allow(clippy::too_many_arguments)]
fn display_callees_with_context(
    caller_simplified: &str,
    root_caller_simplified: &str,
    direct_callee_map: &HashMap<String, Vec<&CallRelation>>,
    context_callee_map: &HashMap<(String, String), Vec<&CallRelation>>,
    _entry_by_simplified: &HashMap<String, &HierarchyEntry>,
    consumed_absolute: &mut HashMap<String, f64>,
    visited: &mut HashSet<String>,
    output: &mut String,
    indent_level: usize,
    use_color: bool,
) {
    // Get direct callees for this caller (using simplified name since relations use simplified symbols)
    let callees = match direct_callee_map.get(caller_simplified) {
        Some(c) => c,
        None => return,
    };

    for callee_rel in callees {
        let callee_simplified = simplify_symbol(&callee_rel.callee);

        // Skip if already visited (recursion prevention)
        if visited.contains(&callee_simplified) {
            continue;
        }
        visited.insert(callee_simplified.clone());

        // Display this callee
        let indent = "    ".repeat(indent_level);
        let callee_symbol = truncate_symbol(&callee_rel.callee, 100 - indent_level * 4);
        let colored_callee = format_colored_symbol(&callee_symbol, use_color);
        output.push_str(&format!(
            "{:>8.2}  {:>6.2}  {}{}\n",
            callee_rel.relative_pct, 0.0, indent, colored_callee
        ));

        // Track consumed absolute contribution
        *consumed_absolute.entry(callee_simplified.clone()).or_default() += callee_rel.absolute_pct;

        // Check if this callee has context-specific nested callees
        // Look for relations with context_root = root_caller and caller = this callee
        let context_key = (root_caller_simplified.to_string(), callee_rel.callee.clone());
        if let Some(nested) = context_callee_map.get(&context_key) {
            for nested_rel in nested {
                let nested_simplified = simplify_symbol(&nested_rel.callee);
                if visited.contains(&nested_simplified) {
                    continue;
                }
                visited.insert(nested_simplified.clone());

                // Display nested callee with context-specific percentage
                let nested_indent = "    ".repeat(indent_level + 1);
                let nested_symbol = truncate_symbol(&nested_rel.callee, 100 - (indent_level + 1) * 4);
                let colored_nested = format_colored_symbol(&nested_symbol, use_color);
                output.push_str(&format!(
                    "{:>8.2}  {:>6.2}  {}{}\n",
                    nested_rel.relative_pct, 0.0, nested_indent, colored_nested
                ));

                // Track consumed absolute contribution
                *consumed_absolute.entry(nested_simplified.clone()).or_default() +=
                    nested_rel.absolute_pct;

                // Continue recursively if this nested callee has its own nested callees
                let deeper_key = (root_caller_simplified.to_string(), nested_rel.callee.clone());
                if context_callee_map.contains_key(&deeper_key) {
                    display_nested_context(
                        &nested_rel.callee,
                        root_caller_simplified,
                        context_callee_map,
                        consumed_absolute,
                        visited,
                        output,
                        indent_level + 2,
                        use_color,
                    );
                }
            }
        }
    }
}

/// Display nested callees from context-specific map.
#[allow(clippy::too_many_arguments)]
fn display_nested_context(
    caller: &str,
    root_caller_simplified: &str,
    context_callee_map: &HashMap<(String, String), Vec<&CallRelation>>,
    consumed_absolute: &mut HashMap<String, f64>,
    visited: &mut HashSet<String>,
    output: &mut String,
    indent_level: usize,
    use_color: bool,
) {
    let context_key = (root_caller_simplified.to_string(), caller.to_string());
    let callees = match context_callee_map.get(&context_key) {
        Some(c) => c,
        None => return,
    };

    for callee_rel in callees {
        let callee_simplified = simplify_symbol(&callee_rel.callee);
        if visited.contains(&callee_simplified) {
            continue;
        }
        visited.insert(callee_simplified.clone());

        let indent = "    ".repeat(indent_level);
        let callee_symbol = truncate_symbol(&callee_rel.callee, 100 - indent_level * 4);
        let colored_callee = format_colored_symbol(&callee_symbol, use_color);
        output.push_str(&format!(
            "{:>8.2}  {:>6.2}  {}{}\n",
            callee_rel.relative_pct, 0.0, indent, colored_callee
        ));

        *consumed_absolute.entry(callee_simplified).or_default() += callee_rel.absolute_pct;

        // Continue recursively
        let deeper_key = (root_caller_simplified.to_string(), callee_rel.callee.clone());
        if context_callee_map.contains_key(&deeper_key) {
            display_nested_context(
                &callee_rel.callee,
                root_caller_simplified,
                context_callee_map,
                consumed_absolute,
                visited,
                output,
                indent_level + 1,
                use_color,
            );
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
