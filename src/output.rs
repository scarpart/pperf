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
/// T006: Added debug parameter to show calculation path annotations.
pub fn format_hierarchy_table(
    entries: &[HierarchyEntry],
    all_relations: &[CallRelation],
    use_color: bool,
    debug: bool,
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
            &simplified, // Use simplified for lookup
            &simplified,
            &direct_callee_map,
            &context_callee_map,
            &entry_by_simplified,
            &mut consumed_absolute,
            &mut visited,
            &mut output,
            1,
            use_color,
            debug,
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

        // Output standalone debug annotation showing the subtraction breakdown
        let standalone_annotation = format_standalone_debug_annotation(
            entry.original_children_pct,
            &entry.contributions,
            entry.adjusted_children_pct,
            use_color,
            debug,
        );
        if !standalone_annotation.is_empty() {
            output.push_str(&format!("                  {}\n", standalone_annotation));
        }

        // If this entry has callees, show remainder callees (overall - consumed)
        if entry.is_caller {
            for callee in &entry.callees {
                let callee_simplified = simplify_symbol(&callee.callee);
                let consumed = consumed_absolute
                    .get(&callee_simplified)
                    .copied()
                    .unwrap_or(0.0);
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
/// T013: Now outputs debug annotations when debug is true.
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
    debug: bool,
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

        // T013: Output debug annotation on separate line below
        let annotation = format_debug_annotation(
            &callee_rel.intermediary_path,
            callee_rel.relative_pct,
            use_color,
            debug,
        );
        if !annotation.is_empty() {
            output.push_str(&format!("                  {}{}\n", indent, annotation));
        }

        // Track consumed absolute contribution
        *consumed_absolute
            .entry(callee_simplified.clone())
            .or_default() += callee_rel.absolute_pct;

        // Check if this callee has context-specific nested callees
        // Look for relations with context_root = root_caller and caller = this callee
        let context_key = (
            root_caller_simplified.to_string(),
            callee_rel.callee.clone(),
        );
        if let Some(nested) = context_callee_map.get(&context_key) {
            for nested_rel in nested {
                let nested_simplified = simplify_symbol(&nested_rel.callee);
                if visited.contains(&nested_simplified) {
                    continue;
                }
                visited.insert(nested_simplified.clone());

                // Display nested callee with context-specific percentage
                let nested_indent = "    ".repeat(indent_level + 1);
                let nested_symbol =
                    truncate_symbol(&nested_rel.callee, 100 - (indent_level + 1) * 4);
                let colored_nested = format_colored_symbol(&nested_symbol, use_color);
                output.push_str(&format!(
                    "{:>8.2}  {:>6.2}  {}{}\n",
                    nested_rel.relative_pct, 0.0, nested_indent, colored_nested
                ));

                // T013: Output debug annotation for nested callee
                let nested_annotation = format_debug_annotation(
                    &nested_rel.intermediary_path,
                    nested_rel.relative_pct,
                    use_color,
                    debug,
                );
                if !nested_annotation.is_empty() {
                    output.push_str(&format!(
                        "                  {}{}\n",
                        nested_indent, nested_annotation
                    ));
                }

                // Track consumed absolute contribution
                *consumed_absolute
                    .entry(nested_simplified.clone())
                    .or_default() += nested_rel.absolute_pct;

                // Continue recursively if this nested callee has its own nested callees
                let deeper_key = (
                    root_caller_simplified.to_string(),
                    nested_rel.callee.clone(),
                );
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
                        debug,
                    );
                }
            }
        }
    }
}

/// Display nested callees from context-specific map.
/// T013: Now outputs debug annotations when debug is true.
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
    debug: bool,
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

        // T013: Output debug annotation
        let annotation = format_debug_annotation(
            &callee_rel.intermediary_path,
            callee_rel.relative_pct,
            use_color,
            debug,
        );
        if !annotation.is_empty() {
            output.push_str(&format!("                  {}{}\n", indent, annotation));
        }

        *consumed_absolute.entry(callee_simplified).or_default() += callee_rel.absolute_pct;

        // Continue recursively
        let deeper_key = (
            root_caller_simplified.to_string(),
            callee_rel.callee.clone(),
        );
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
                debug,
            );
        }
    }
}

/// T012: Format debug annotation for calculation path.
/// Returns empty string if debug is false.
/// For direct calls (empty path): "(direct: X%)"
/// For indirect calls: "(via A 42.00% × B 50.00% = 21.00%)"
pub fn format_debug_annotation(
    intermediary_path: &[crate::hierarchy::IntermediaryStep],
    final_pct: f64,
    use_color: bool,
    debug: bool,
) -> String {
    // Return empty if debug mode is not enabled
    if !debug {
        return String::new();
    }

    use crate::symbol::{DIM, RESET};

    let content = if intermediary_path.is_empty() {
        // T017: Direct call - no intermediaries
        format!("(direct: {:.2}%)", final_pct)
    } else {
        // Indirect call - show multiplication chain
        let steps: Vec<String> = intermediary_path
            .iter()
            .map(|step| format!("{} {:.2}%", step.symbol, step.percentage))
            .collect();
        let chain = steps.join(" × ");
        format!("(via {} = {:.2}%)", chain, final_pct)
    };

    // T014: Apply DIM color when use_color is true
    if use_color {
        format!("{}{}{}", DIM, content, RESET)
    } else {
        content
    }
}

/// Format debug annotation for standalone entries.
/// Returns empty string if debug is false or no contributions to show.
/// Format: "(standalone: X.XX% - Y.YY% (CallerA) - Z.ZZ% (CallerB) = W.WW%)"
pub fn format_standalone_debug_annotation(
    original_pct: f64,
    contributions: &[crate::hierarchy::CallerContribution],
    adjusted_pct: f64,
    use_color: bool,
    debug: bool,
) -> String {
    // Return empty if debug mode is not enabled
    if !debug {
        return String::new();
    }

    // Skip annotation if no contributions (original == adjusted)
    if contributions.is_empty() {
        return String::new();
    }

    use crate::symbol::{DIM, RESET};

    // Build subtraction chain: "- X.XX% (CallerA) - Y.YY% (CallerB)"
    let subtractions: Vec<String> = contributions
        .iter()
        .map(|c| format!("{:.2}% ({})", c.absolute_pct, c.caller))
        .collect();
    let chain = subtractions.join(" - ");

    let content = format!(
        "(standalone: {:.2}% - {} = {:.2}%)",
        original_pct, chain, adjusted_pct
    );

    // Apply DIM color when use_color is true
    if use_color {
        format!("{}{}{}", DIM, content, RESET)
    } else {
        content
    }
}

#[cfg(test)]
mod tests {
    use crate::hierarchy::IntermediaryStep;
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

    // T008: Unit test for format_debug_annotation with single intermediary
    #[test]
    fn test_format_debug_annotation_single_intermediary() {
        let path = vec![IntermediaryStep {
            symbol: "do_4d_transform".to_string(),
            percentage: 42.0,
        }];

        // With debug enabled, no color
        let annotation = super::format_debug_annotation(&path, 42.0, false, true);
        assert!(
            annotation.contains("via"),
            "Should contain 'via' for indirect call"
        );
        assert!(
            annotation.contains("do_4d_transform"),
            "Should contain intermediary name"
        );
        assert!(annotation.contains("42.00%"), "Should contain percentage");

        // With debug disabled, should return empty
        let empty = super::format_debug_annotation(&path, 42.0, false, false);
        assert!(empty.is_empty(), "Should be empty when debug is false");
    }

    // T009: Unit test for format_debug_annotation with multiple intermediaries
    #[test]
    fn test_format_debug_annotation_multiple_intermediaries() {
        let path = vec![
            IntermediaryStep {
                symbol: "do_4d_transform".to_string(),
                percentage: 50.0,
            },
            IntermediaryStep {
                symbol: "compute_dct".to_string(),
                percentage: 80.0,
            },
        ];

        // Final percentage = 50% × 80% = 40%
        let annotation = super::format_debug_annotation(&path, 40.0, false, true);
        assert!(
            annotation.contains("via"),
            "Should contain 'via' for indirect call"
        );
        assert!(
            annotation.contains("do_4d_transform"),
            "Should contain first intermediary"
        );
        assert!(
            annotation.contains("compute_dct"),
            "Should contain second intermediary"
        );
        assert!(
            annotation.contains("×"),
            "Should contain multiplication symbol"
        );
        assert!(
            annotation.contains("50.00%"),
            "Should contain first percentage"
        );
        assert!(
            annotation.contains("80.00%"),
            "Should contain second percentage"
        );
        assert!(annotation.contains("= 40.00%"), "Should show final result");
    }

    // T015: Unit test for format_debug_annotation with empty path (direct call)
    #[test]
    fn test_format_debug_annotation_direct_call() {
        let path: Vec<IntermediaryStep> = vec![];

        // Direct call should show "(direct: X%)"
        let annotation = super::format_debug_annotation(&path, 25.0, false, true);
        assert!(
            annotation.contains("direct"),
            "Should contain 'direct' for direct call"
        );
        assert!(annotation.contains("25.00%"), "Should contain percentage");
        assert!(
            !annotation.contains("via"),
            "Should NOT contain 'via' for direct call"
        );

        // With debug disabled, should return empty
        let empty = super::format_debug_annotation(&path, 25.0, false, false);
        assert!(empty.is_empty(), "Should be empty when debug is false");
    }

    // Unit test for format_standalone_debug_annotation with single caller
    #[test]
    fn test_format_standalone_debug_annotation_single_caller() {
        use crate::hierarchy::CallerContribution;

        let contributions = vec![CallerContribution {
            caller: "rd_optimize_transform".to_string(),
            absolute_pct: 12.37,
        }];

        // original 38.00% - 12.37% = 25.63%
        let annotation =
            super::format_standalone_debug_annotation(38.00, &contributions, 25.63, false, true);
        assert!(
            annotation.contains("standalone"),
            "Should contain 'standalone'"
        );
        assert!(
            annotation.contains("38.00%"),
            "Should contain original percentage"
        );
        assert!(
            annotation.contains("12.37%"),
            "Should contain contribution amount"
        );
        assert!(
            annotation.contains("rd_optimize_transform"),
            "Should contain caller name"
        );
        assert!(
            annotation.contains("25.63%"),
            "Should contain final adjusted percentage"
        );
    }

    // Unit test for format_standalone_debug_annotation with multiple callers
    #[test]
    fn test_format_standalone_debug_annotation_multiple_callers() {
        use crate::hierarchy::CallerContribution;

        let contributions = vec![
            CallerContribution {
                caller: "CallerA".to_string(),
                absolute_pct: 20.0,
            },
            CallerContribution {
                caller: "CallerB".to_string(),
                absolute_pct: 15.0,
            },
        ];

        // original 50.00% - 20.00% - 15.00% = 15.00%
        let annotation =
            super::format_standalone_debug_annotation(50.00, &contributions, 15.00, false, true);
        assert!(
            annotation.contains("standalone"),
            "Should contain 'standalone'"
        );
        assert!(annotation.contains("50.00%"), "Should contain original");
        assert!(
            annotation.contains("20.00%"),
            "Should contain first contribution"
        );
        assert!(
            annotation.contains("15.00%"),
            "Should contain second contribution/result"
        );
        assert!(
            annotation.contains("CallerA"),
            "Should contain first caller"
        );
        assert!(
            annotation.contains("CallerB"),
            "Should contain second caller"
        );
    }

    // Unit test for format_standalone_debug_annotation with empty contributions
    #[test]
    fn test_format_standalone_debug_annotation_no_contributions() {
        use crate::hierarchy::CallerContribution;

        let contributions: Vec<CallerContribution> = vec![];

        // No contributions - should return empty
        let annotation =
            super::format_standalone_debug_annotation(38.00, &contributions, 38.00, false, true);
        assert!(
            annotation.is_empty(),
            "Should be empty when no contributions"
        );
    }

    // Unit test for format_standalone_debug_annotation with debug disabled
    #[test]
    fn test_format_standalone_debug_annotation_debug_disabled() {
        use crate::hierarchy::CallerContribution;

        let contributions = vec![CallerContribution {
            caller: "SomeCaller".to_string(),
            absolute_pct: 10.0,
        }];

        // Debug disabled - should return empty
        let annotation =
            super::format_standalone_debug_annotation(50.00, &contributions, 40.00, false, false);
        assert!(annotation.is_empty(), "Should be empty when debug is false");
    }
}
