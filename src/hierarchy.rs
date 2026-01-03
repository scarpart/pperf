//! Call hierarchy parsing and analysis module.
//!
//! This module handles parsing perf report call trees and computing
//! caller-callee relationships between target functions.

use crate::parser::PerfEntry;
use crate::symbol::simplify_symbol;
use std::collections::HashSet;

/// T001: Represents a single line from the perf report call tree section.
#[derive(Debug, Clone, PartialEq)]
pub struct CallTreeLine {
    /// Call stack depth (0 = top-level entry, 1+ = call tree)
    pub depth: usize,
    /// Percentage if present (from `--XX.XX%--` pattern)
    pub relative_pct: Option<f64>,
    /// Function name (simplified via symbol module)
    pub symbol: String,
    /// True if this is a top-level perf entry with absolute %
    pub is_top_level: bool,
}

/// T003: Hierarchical representation of a function and its callees.
#[derive(Debug, Clone, PartialEq)]
pub struct CallTreeNode {
    /// Simplified function name
    pub symbol: String,
    /// Percentage relative to parent (0.0-100.0)
    pub relative_pct: f64,
    /// Direct callees in the call tree
    pub children: Vec<CallTreeNode>,
}

/// T004: Represents a caller→callee relationship between two target functions.
#[derive(Debug, Clone, PartialEq)]
pub struct CallRelation {
    /// Caller target function (simplified name)
    pub caller: String,
    /// Callee target function (simplified name)
    pub callee: String,
    /// Callee's contribution as % of caller's time
    pub relative_pct: f64,
    /// Absolute contribution: caller.children_pct × relative_pct / 100
    pub absolute_pct: f64,
}

/// T005: Target function with computed hierarchy data for output.
#[derive(Debug, Clone, PartialEq)]
pub struct HierarchyEntry {
    /// Simplified function name
    pub symbol: String,
    /// Original Children% from perf report
    pub original_children_pct: f64,
    /// Original Self% from perf report
    pub original_self_pct: f64,
    /// After subtracting callee contributions
    pub adjusted_children_pct: f64,
    /// Targeted callees under this function
    pub callees: Vec<CallRelation>,
    /// True if this function has targeted callees
    pub is_caller: bool,
}

// ============================================================================
// Phase 2: Call Tree Parsing Functions
// ============================================================================

/// T013: Count the depth of a call tree line based on column position.
/// In perf report, each nesting level adds approximately 11 characters of indentation.
/// We find the position of the `--XX.XX%--` or `---` pattern and divide by 11.
pub fn count_depth(line: &str) -> usize {
    // Find the position of the percentage pattern (--XX.XX%--)
    if let Some(pct_end) = line.find("%--") {
        // Search backwards from %-- to find the leading --
        let before = &line[..pct_end];
        if let Some(dash_pos) = before.rfind("--") {
            // Each tree level is approximately 11 characters wide
            return (dash_pos / 11) + 1;
        }
    }

    // Fallback: look for --- pattern (function without percentage)
    if let Some(pos) = line.find("---") {
        return (pos / 11) + 1;
    }

    // Final fallback: count pipes (for lines that don't match above patterns)
    line.chars().filter(|&c| c == '|').count()
}

/// T014: Extract percentage from `--XX.XX%--` pattern.
pub fn extract_percentage(line: &str) -> Option<f64> {
    // Find pattern: --XX.XX%--
    let start_marker = "--";
    let end_marker = "%--";

    // Find the percentage pattern
    if let Some(end_pos) = line.find(end_marker) {
        // Search backwards from end_pos for the start marker
        let search_region = &line[..end_pos];
        if let Some(start_pos) = search_region.rfind(start_marker) {
            let pct_str = &line[start_pos + 2..end_pos];
            return pct_str.trim().parse().ok();
        }
    }
    None
}

/// T015: Extract the function symbol from a call tree line.
pub fn extract_symbol(line: &str) -> Option<String> {
    // If line contains percentage pattern, extract symbol after it
    if let Some(end_pos) = line.find("%--") {
        let after_pct = &line[end_pos + 3..];
        let symbol = after_pct.trim();
        if !symbol.is_empty() {
            return Some(simplify_symbol(symbol));
        }
    }

    // For lines without percentage (function continuations)
    // Look for function name after the tree markers
    let trimmed = line.trim();

    // Skip lines that are just pipes or tree structure
    if trimmed.is_empty() || trimmed.chars().all(|c| c == '|' || c == ' ' || c == '-') {
        return None;
    }

    // Find the actual function name
    // It's typically after the last tree marker sequence
    if let Some(pos) = trimmed.rfind("---") {
        let after = &trimmed[pos + 3..];
        let symbol = after.trim();
        if !symbol.is_empty() {
            return Some(simplify_symbol(symbol));
        }
    }

    // Check if it's a continuation line (just function name after pipes/spaces)
    let content: String = trimmed
        .chars()
        .skip_while(|&c| c == '|' || c == ' ')
        .collect();
    let content = content.trim();
    if !content.is_empty() && !content.starts_with('-') {
        return Some(simplify_symbol(content));
    }

    None
}

/// T016: Parse a single call tree line into a CallTreeLine struct.
pub fn parse_call_tree_line(line: &str) -> Option<CallTreeLine> {
    let trimmed = line.trim_start();

    // Skip empty lines and comments
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }

    // Check if this is a top-level entry (starts with percentage)
    if trimmed.chars().next()?.is_ascii_digit() {
        // This is a top-level perf entry, not a call tree line
        // We handle these separately
        return None;
    }

    // Must be a call tree line (contains | or starts with ---)
    if !trimmed.starts_with('|') && !trimmed.starts_with('-') {
        // Could be a function continuation line
        if line.contains('|') {
            // Parse as call tree line
        } else {
            return None;
        }
    }

    let depth = count_depth(line);
    let relative_pct = extract_percentage(line);
    let symbol = extract_symbol(line)?;

    Some(CallTreeLine {
        depth,
        relative_pct,
        symbol,
        is_top_level: false,
    })
}

/// T017: Build a call tree from a list of CallTreeLine entries.
pub fn build_call_tree(lines: &[CallTreeLine]) -> Vec<CallTreeNode> {
    if lines.is_empty() {
        return Vec::new();
    }

    let mut roots: Vec<CallTreeNode> = Vec::new();
    let mut stack: Vec<(usize, CallTreeNode)> = Vec::new(); // (depth, node)

    for line in lines {
        let node = CallTreeNode {
            symbol: line.symbol.clone(),
            relative_pct: line.relative_pct.unwrap_or(100.0),
            children: Vec::new(),
        };

        if stack.is_empty() {
            // First node becomes root
            stack.push((line.depth, node));
        } else {
            let current_depth = line.depth;
            let (last_depth, _) = stack.last().unwrap();

            if current_depth > *last_depth {
                // Deeper: this is a child of the previous node
                stack.push((current_depth, node));
            } else {
                // Same or shallower: pop until we find the parent level
                while stack.len() > 1 {
                    let (d, _) = stack.last().unwrap();
                    if *d < current_depth {
                        break;
                    }
                    let (_, child) = stack.pop().unwrap();
                    if let Some((_, parent)) = stack.last_mut() {
                        parent.children.push(child);
                    } else {
                        roots.push(child);
                    }
                }

                if current_depth == 0 || stack.is_empty() {
                    // This is a new root
                    if !stack.is_empty() {
                        let (_, child) = stack.pop().unwrap();
                        roots.push(child);
                    }
                    stack.push((current_depth, node));
                } else {
                    stack.push((current_depth, node));
                }
            }
        }
    }

    // Flush remaining stack
    while let Some((_, child)) = stack.pop() {
        if let Some((_, parent)) = stack.last_mut() {
            parent.children.push(child);
        } else {
            roots.push(child);
        }
    }

    roots
}

/// T019: Parse call trees from perf report content.
/// Returns a list of (top-level PerfEntry, associated call tree nodes).
pub fn parse_file_call_trees(
    content: &str,
    _entries: &[PerfEntry],
) -> Vec<(PerfEntry, Vec<CallTreeNode>)> {
    let mut result: Vec<(PerfEntry, Vec<CallTreeNode>)> = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    let mut current_entry: Option<PerfEntry> = None;
    let mut current_tree_lines: Vec<CallTreeLine> = Vec::new();

    for line in &lines {
        let trimmed = line.trim_start();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Check if this is a top-level entry
        if trimmed.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            // Finalize previous entry if any
            if let Some(entry) = current_entry.take() {
                let tree = build_call_tree(&current_tree_lines);
                result.push((entry, tree));
                current_tree_lines.clear();
            }

            // Parse this as a new top-level entry
            if let Some(parsed) = crate::parser::parse_line(line) {
                // Simplify the symbol
                let simplified = PerfEntry {
                    children_pct: parsed.children_pct,
                    self_pct: parsed.self_pct,
                    symbol: simplify_symbol(&parsed.symbol),
                };
                current_entry = Some(simplified);
            }
        } else if let Some(tree_line) = parse_call_tree_line(line) {
            current_tree_lines.push(tree_line);
        }
    }

    // Finalize last entry
    if let Some(entry) = current_entry {
        let tree = build_call_tree(&current_tree_lines);
        result.push((entry, tree));
    }

    result
}

// ============================================================================
// Phase 3: Target Relationship Discovery
// ============================================================================

/// T027: Check if a target exists in the call tree.
pub fn find_target_in_tree(tree: &CallTreeNode, target: &str) -> bool {
    if tree.symbol.contains(target) {
        return true;
    }
    for child in &tree.children {
        if find_target_in_tree(child, target) {
            return true;
        }
    }
    false
}

/// T028-T029: Find target callees under a caller, with recursion detection
/// and percentage multiplication through intermediates.
pub fn find_target_callees(
    node: &CallTreeNode,
    targets: &[String],
    caller: &str,
    caller_children_pct: f64,
    cumulative_pct: f64,
    seen: &mut HashSet<String>,
) -> Vec<CallRelation> {
    let mut relations = Vec::new();

    for child in &node.children {
        let child_pct = child.relative_pct;
        let new_cumulative = cumulative_pct * child_pct / 100.0;

        // Check if this child matches any target
        let is_target = targets.iter().any(|t| child.symbol.contains(t));

        if is_target {
            // Check for recursion - if already seen, traverse into it but don't record
            if seen.contains(&child.symbol) {
                // Still traverse into this node to find deeper targets
                let deeper = find_target_callees(
                    child,
                    targets,
                    caller,
                    caller_children_pct,
                    new_cumulative,
                    seen,
                );
                relations.extend(deeper);
            } else {
                // Record this relationship
                // relative_pct is the percentage shown in the indented output
                // absolute_pct is caller.children_pct × relative_pct / 100 (for adjusted %)
                let relation = CallRelation {
                    caller: caller.to_string(),
                    callee: child.symbol.clone(),
                    relative_pct: child_pct,
                    absolute_pct: caller_children_pct * child_pct / 100.0,
                };
                relations.push(relation);

                // Mark as seen to prevent recording again
                seen.insert(child.symbol.clone());

                // Don't traverse deeper into this target's subtree
                // (they'll have their own top-level entry)
            }
        } else {
            // Not a target, continue traversing
            let deeper = find_target_callees(
                child,
                targets,
                caller,
                caller_children_pct,
                new_cumulative,
                seen,
            );
            relations.extend(deeper);
        }
    }

    relations
}

/// Check if an entry is a "leaf" function where the call tree shows callers, not callees.
/// Leaf functions have Self% approximately equal to Children%, meaning they don't call
/// other functions that consume significant time. For these, perf report shows the
/// call path TO the function, not FROM it.
fn is_leaf_function(entry: &PerfEntry) -> bool {
    // Consider it a leaf if Self% is within 1% of Children%, or if Self% > 50% of Children%
    let diff = (entry.children_pct - entry.self_pct).abs();
    diff < 1.0 || entry.self_pct > entry.children_pct * 0.5
}

/// T030: Compute all call relations between targets.
pub fn compute_call_relations(
    trees: &[(PerfEntry, Vec<CallTreeNode>)],
    targets: &[String],
) -> Vec<CallRelation> {
    let mut all_relations = Vec::new();

    for (entry, tree_roots) in trees {
        // Check if this entry is a target
        let is_target = targets.iter().any(|t| entry.symbol.contains(t));

        if is_target {
            // Skip leaf functions - their call tree shows callers, not callees
            if is_leaf_function(entry) {
                continue;
            }

            // This entry is a caller, look for callees
            for root in tree_roots {
                let mut seen = HashSet::new();
                seen.insert(entry.symbol.clone()); // Prevent self-recursion

                let relations = find_target_callees(
                    root,
                    targets,
                    &entry.symbol,
                    entry.children_pct,
                    100.0, // Start at 100% of caller's time
                    &mut seen,
                );
                all_relations.extend(relations);
            }
        }
    }

    all_relations
}

// ============================================================================
// Phase 4: Percentage Adjustment
// ============================================================================

/// T036: Compute adjusted percentage after subtracting contributions.
pub fn compute_adjusted_percentage(original: f64, contributions: &[f64]) -> f64 {
    let sum: f64 = contributions.iter().sum();
    (original - sum).max(0.0)
}

/// T037: Build hierarchy entries from entries and relations.
pub fn build_hierarchy_entries(
    entries: &[PerfEntry],
    targets: &[String],
    relations: &[CallRelation],
) -> Vec<HierarchyEntry> {
    use crate::symbol::simplify_symbol;

    let mut result = Vec::new();

    // Track which simplified symbols we've already added to avoid duplicates
    // (e.g., rd_optimize_transform appears twice with 71.80% and 71.78%)
    let mut added_symbols: HashSet<String> = HashSet::new();

    // Collect unique callers from relations (these are the "root" callers)
    let callers: HashSet<String> = relations.iter().map(|r| r.caller.clone()).collect();

    for entry in entries {
        // Check if this entry matches any target
        let is_target = targets.iter().any(|t| entry.symbol.contains(t));
        if !is_target {
            continue;
        }

        // Simplify the symbol for deduplication
        let simplified = simplify_symbol(&entry.symbol);

        // Skip if we've already added an entry with this simplified symbol
        if added_symbols.contains(&simplified) {
            continue;
        }

        // Find callees for this entry (use contains for matching simplified symbols)
        // Deduplicate by callee symbol, keeping only unique callees
        let mut callees: Vec<CallRelation> = Vec::new();
        let mut seen_callees: HashSet<String> = HashSet::new();
        for r in relations
            .iter()
            .filter(|r| entry.symbol.contains(&r.caller))
        {
            if !seen_callees.contains(&r.callee) {
                seen_callees.insert(r.callee.clone());
                callees.push(r.clone());
            }
        }

        // Find contributions TO this entry (when it's a callee)
        // Only count contributions once per unique caller
        let mut contribution_callers: HashSet<String> = HashSet::new();
        let contributions: Vec<f64> = relations
            .iter()
            .filter(|r| {
                if entry.symbol.contains(&r.callee) && !contribution_callers.contains(&r.caller) {
                    contribution_callers.insert(r.caller.clone());
                    true
                } else {
                    false
                }
            })
            .map(|r| r.absolute_pct)
            .collect();

        let adjusted = compute_adjusted_percentage(entry.children_pct, &contributions);

        // Determine if this entry is a caller (has callees) or just a callee
        let is_caller = !callees.is_empty();

        // If this is purely a callee (not a caller), check if it's called by another target
        // and only show it as standalone if it has unique standalone time
        let is_callee_of_target = callers.iter().any(|c| entry.symbol.contains(c));

        // Skip entries that are both a callee AND have no callees themselves
        // unless they're also a caller
        if !is_caller && is_callee_of_target && adjusted < 0.01 {
            // This entry's time is fully accounted for by callers, skip it
            continue;
        }

        added_symbols.insert(simplified);

        result.push(HierarchyEntry {
            symbol: entry.symbol.clone(),
            original_children_pct: entry.children_pct,
            original_self_pct: entry.self_pct,
            adjusted_children_pct: adjusted,
            callees,
            is_caller,
        });
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    // T006: Test parse_call_tree_line with percentage
    #[test]
    fn test_parse_call_tree_line_with_percentage() {
        let line = "               |--17.23%--DCT4DBlock::DCT4DBlock";
        let result = parse_call_tree_line(line);
        assert!(result.is_some());
        let tree_line = result.unwrap();
        // Depth is now based on column position: -- at column 16, 16/11+1 = 2
        assert_eq!(tree_line.depth, 2);
        assert!((tree_line.relative_pct.unwrap() - 17.23).abs() < 0.01);
        assert!(tree_line.symbol.contains("DCT4DBlock"));
    }

    // T009: Test count_depth - now based on column position of --XX%-- pattern
    #[test]
    fn test_count_depth() {
        // Depth 1: -- at column ~16 (16/11 + 1 = 2, but actual perf uses ~16)
        assert_eq!(count_depth("               |--17.23%--func"), 2);
        // Depth 2: -- at column ~27
        assert_eq!(count_depth("               |           --5.00%--func"), 3);
        // Depth 3: |-- at column ~37
        assert_eq!(
            count_depth("               |                     |--5.00%--func"),
            4
        );
        // No pattern found: fallback to pipe count
        assert_eq!(count_depth("no pipes here"), 0);
    }

    // T010: Test extract_percentage
    #[test]
    fn test_extract_percentage() {
        assert!((extract_percentage("|--17.23%--func").unwrap() - 17.23).abs() < 0.01);
        assert!((extract_percentage("--49.34%--func").unwrap() - 49.34).abs() < 0.01);
        assert!(extract_percentage("func without percentage").is_none());
    }

    // T011: Test extract_symbol
    #[test]
    fn test_extract_symbol() {
        let result = extract_symbol("|--17.23%--MyFunction");
        assert!(result.is_some());
        assert!(result.unwrap().contains("MyFunction"));
    }

    // T032: Test compute_adjusted_percentage with single contribution
    #[test]
    fn test_compute_adjusted_percentage_single() {
        let adjusted = compute_adjusted_percentage(38.0, &[12.37]);
        assert!((adjusted - 25.63).abs() < 0.01);
    }

    // T033: Test compute_adjusted_percentage with multiple contributions
    #[test]
    fn test_compute_adjusted_percentage_multiple() {
        let adjusted = compute_adjusted_percentage(50.0, &[10.0, 15.0, 5.0]);
        assert!((adjusted - 20.0).abs() < 0.01);
    }

    // T034: Test compute_adjusted_percentage flooring at zero
    #[test]
    fn test_compute_adjusted_percentage_floor() {
        let adjusted = compute_adjusted_percentage(10.0, &[15.0, 20.0]);
        assert_eq!(adjusted, 0.0);
    }
}
