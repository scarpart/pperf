//! Call hierarchy parsing and analysis module.
//!
//! This module handles parsing perf report call trees and computing
//! caller-callee relationships between target functions.

use crate::parser::PerfEntry;
use crate::symbol::simplify_symbol;
use std::collections::HashSet;

/// Match a symbol against targets based on the matching mode.
/// - `exact_mode = true`: targets are exact signatures, simplify and compare with equality
/// - `exact_mode = false`: targets are substrings, use contains matching
fn matches_any_target(symbol: &str, targets: &[String], exact_mode: bool) -> bool {
    if exact_mode {
        // Exact mode: simplify each target and compare with equality
        targets
            .iter()
            .any(|t| symbol == simplify_symbol(t))
    } else {
        // Substring mode: check if symbol contains any target
        targets.iter().any(|t| symbol.contains(t))
    }
}

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

/// Represents one step in the intermediary path between caller and callee.
/// Used to show the calculation breakdown in debug mode.
#[derive(Debug, Clone, PartialEq)]
pub struct IntermediaryStep {
    /// Simplified function name of the intermediary
    pub symbol: String,
    /// Relative percentage at this step in the call chain
    pub percentage: f64,
}

/// Represents one caller's contribution to a standalone entry's adjusted percentage.
/// Used to show the subtraction breakdown in debug mode.
#[derive(Debug, Clone, PartialEq)]
pub struct CallerContribution {
    /// Simplified name of the calling target function
    pub caller: String,
    /// The contribution amount (absolute %) subtracted from original
    pub absolute_pct: f64,
}

/// T004: Represents a caller→callee relationship between two target functions.
#[derive(Debug, Clone, PartialEq)]
pub struct CallRelation {
    /// Caller target function (simplified name)
    pub caller: String,
    /// Callee target function (simplified name)
    pub callee: String,
    /// Callee's contribution as % of caller's time (in context if context_root is set)
    pub relative_pct: f64,
    /// Absolute contribution: root.children_pct × path_product / 100
    pub absolute_pct: f64,
    /// If this relation was found in another caller's tree, store that root caller.
    /// None = this is from the caller's own tree (overall relationship)
    /// Some(root) = this is path-specific, found when traversing root's tree
    pub context_root: Option<String>,
    /// Ordered list of non-target functions traversed between caller and callee.
    /// Empty if this is a direct call (no intermediaries).
    pub intermediary_path: Vec<IntermediaryStep>,
    /// Callee's direct percentage relative to its immediate parent in the call tree.
    /// Used for debug annotations to show the complete calculation path.
    pub callee_direct_pct: f64,
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
    /// Breakdown of contributions FROM callers that were subtracted (for debug mode)
    pub contributions: Vec<CallerContribution>,
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

    // For continuation lines (function names without --- or --XX%-- patterns),
    // use leading whitespace to estimate depth
    let leading_spaces = line.len() - line.trim_start().len();
    if leading_spaces >= 8 {
        // Continuation lines are indented; estimate depth from column position
        return (leading_spaces / 11) + 1;
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

    // Must be a call tree line. Valid call tree lines:
    // 1. Start with | or - (tree markers)
    // 2. Contain | (tree structure)
    // 3. Have significant indentation (>=8 spaces) - continuation lines
    let leading_spaces = line.len() - line.trim_start().len();
    if !trimmed.starts_with('|') && !trimmed.starts_with('-') && !line.contains('|') {
        // Could be a function continuation line (indented function name without tree markers)
        if leading_spaces < 8 {
            return None;
        }
        // Continuation line: parse as call tree line with depth from indentation
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

/// Check if a call tree line is a continuation line (no `---` or `--XX%--` markers).
fn is_continuation_line(line: &str) -> bool {
    !line.contains("---") && !line.contains("%--")
}

/// Post-process call tree lines to fix continuation line depths.
/// In perf reports, continuation lines (function names without markers) that follow
/// a `---` line are immediate callees and should be children, not siblings.
fn fix_continuation_depths(lines: &mut [CallTreeLine]) {
    if lines.is_empty() {
        return;
    }

    // Track whether the previous non-continuation line was a root entry (has ---)
    // and at what depth
    let mut i = 0;
    while i < lines.len() {
        // Skip non-continuation lines (they have proper depth from markers)
        if lines[i].relative_pct.is_some() {
            // This line has a percentage marker - keep original depth
            i += 1;
            continue;
        }

        // This line has no percentage - check if it's a continuation following
        // another line at the same depth
        if i > 0 {
            let prev_depth = lines[i - 1].depth;
            let curr_depth = lines[i].depth;

            // If same depth and previous had no percentage either (both continuations),
            // or previous was a root (--- line), make this line a child
            if curr_depth == prev_depth && lines[i - 1].relative_pct.is_none() {
                // Consecutive continuation lines at same depth: second is child of first
                lines[i].depth = prev_depth + 1;
            }
        }
        i += 1;
    }
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
                // Fix continuation line depths before building tree
                fix_continuation_depths(&mut current_tree_lines);
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
        // Fix continuation line depths before building tree
        fix_continuation_depths(&mut current_tree_lines);
        let tree = build_call_tree(&current_tree_lines);
        result.push((entry, tree));
    }

    result
}

// ============================================================================
// Phase 3: Target Relationship Discovery
// ============================================================================

/// T027: Check if a target exists in the call tree.
/// - `exact_mode = true`: compare simplified target with equality
/// - `exact_mode = false`: use substring matching
pub fn find_target_in_tree(tree: &CallTreeNode, target: &str, exact_mode: bool) -> bool {
    let matches = if exact_mode {
        tree.symbol == simplify_symbol(target)
    } else {
        tree.symbol.contains(target)
    };
    if matches {
        return true;
    }
    for child in &tree.children {
        if find_target_in_tree(child, target, exact_mode) {
            return true;
        }
    }
    false
}

/// T028-T029: Find target callees under a caller, with context tracking.
/// Now traverses INTO target subtrees to find path-specific percentages.
/// T011: Now also tracks intermediary_path for debug annotations.
///
/// Parameters:
/// - node: Current node in the call tree
/// - targets: List of target function patterns
/// - root_caller: The root caller we're traversing from (for context_root)
/// - root_children_pct: Root caller's Children% (for absolute calculations)
/// - target_stack: Stack of (target_symbol, cumulative_pct_to_target) for intermediate targets
/// - cumulative_pct: Cumulative product of percentages from root to current node
/// - seen: Set of targets already recorded (prevents duplicate recording)
/// - inside_root_recursion: True if path only contains root caller recursive calls (no other intermediates)
/// - current_path: Accumulator for non-target intermediary functions traversed
/// - exact_mode: True for exact signature matching (--target-file), false for substring matching (-t)
#[allow(clippy::too_many_arguments)]
pub fn find_target_callees(
    node: &CallTreeNode,
    targets: &[String],
    root_caller: &str,
    root_children_pct: f64,
    target_stack: &mut Vec<(String, f64)>,
    cumulative_pct: f64,
    seen: &mut HashSet<String>,
    inside_root_recursion: bool,
    current_path: &mut Vec<IntermediaryStep>,
    exact_mode: bool,
) -> Vec<CallRelation> {
    let mut relations = Vec::new();
    let root_caller_simplified = simplify_symbol(root_caller);

    for child in &node.children {
        let child_pct = child.relative_pct;

        // Check if this child is a recursive call of the root caller
        let is_root_recursion = child.symbol == root_caller_simplified;

        // Track whether we're inside root-caller recursion
        // If we encounter root caller again, restore to inside=true
        // This handles: rd_optimize -> eval -> rd_optimize -> DCT4DBlock
        // DCT4DBlock should use child_pct because it's direct child of rd_optimize
        let still_inside_root_recursion = if is_root_recursion {
            true // Re-entering root caller: restore to inside
        } else {
            false // Non-root intermediate: break the chain
        };

        // Calculate cumulative percentage
        let new_cumulative = if is_root_recursion {
            // For recursive calls of root caller, reset to child's percentage
            child_pct
        } else {
            cumulative_pct * child_pct / 100.0
        };

        // Check if this child matches any target
        let is_target = matches_any_target(&child.symbol, targets, exact_mode);

        if is_target {
            // Check for recursion - if already seen, skip recording but continue traversing
            if seen.contains(&child.symbol) {
                // Continue traversing to find deeper targets
                // Clear path when entering already-seen target's subtree
                let mut fresh_path = Vec::new();
                let deeper = find_target_callees(
                    child,
                    targets,
                    root_caller,
                    root_children_pct,
                    target_stack,
                    new_cumulative,
                    seen,
                    still_inside_root_recursion,
                    &mut fresh_path,
                    exact_mode,
                );
                relations.extend(deeper);
            } else {
                // Record this relationship
                if target_stack.is_empty() {
                    // Direct callee of root caller
                    // Use child_pct when found through root caller recursion only
                    // (e.g., rd_optimize -> rd_optimize -> DCT4DBlock)
                    // Use cumulative when found through other intermediates
                    // (e.g., DCT4DBlock -> do_4d_transform -> inner_product)
                    let effective_pct = if inside_root_recursion {
                        child_pct // Path through root recursion: use direct %
                    } else {
                        new_cumulative // Path through other intermediates: use cumulative
                    };
                    let relation = CallRelation {
                        caller: root_caller.to_string(),
                        callee: child.symbol.clone(),
                        relative_pct: effective_pct,
                        absolute_pct: root_children_pct * effective_pct / 100.0,
                        context_root: None, // Direct from root, no context
                        intermediary_path: current_path.clone(), // T011: Include accumulated path
                        callee_direct_pct: child_pct, // Callee's % relative to its direct parent
                    };
                    relations.push(relation);
                } else {
                    // Callee under an intermediate target
                    let (immediate_caller, caller_cumulative) = target_stack.last().unwrap();
                    // Calculate relative % from immediate caller to this callee
                    // new_cumulative is from root, caller_cumulative is from root to immediate caller
                    let relative_to_caller = if *caller_cumulative > 0.0 {
                        new_cumulative / caller_cumulative * 100.0
                    } else {
                        0.0
                    };
                    let relation = CallRelation {
                        caller: immediate_caller.clone(),
                        callee: child.symbol.clone(),
                        relative_pct: relative_to_caller,
                        absolute_pct: root_children_pct * new_cumulative / 100.0,
                        context_root: Some(root_caller.to_string()),
                        intermediary_path: current_path.clone(), // T011: Include accumulated path
                        callee_direct_pct: child_pct, // Callee's % relative to its direct parent
                    };
                    relations.push(relation);
                }

                // Mark as seen to prevent recording duplicate relations
                seen.insert(child.symbol.clone());

                // Push this target onto the stack and continue traversing its subtree
                // When entering a target's subtree, reset inside_root_recursion to true
                // (we start fresh tracking for the new caller)
                // T011: Clear path when entering target's subtree (new caller context)
                target_stack.push((child.symbol.clone(), new_cumulative));
                let mut fresh_path = Vec::new();
                let deeper = find_target_callees(
                    child,
                    targets,
                    root_caller,
                    root_children_pct,
                    target_stack,
                    new_cumulative,
                    seen,
                    true, // Reset: entering target's own subtree
                    &mut fresh_path,
                    exact_mode,
                );
                relations.extend(deeper);
                target_stack.pop();
            }
        } else {
            // Not a target, continue traversing
            // T011: Add this non-target function to the intermediary path
            // Skip root caller recursion (self-calls don't count as intermediaries)
            if !is_root_recursion {
                current_path.push(IntermediaryStep {
                    symbol: child.symbol.clone(),
                    percentage: child_pct,
                });
            }

            // Pass still_inside_root_recursion - becomes false if we went through non-root intermediate
            let deeper = find_target_callees(
                child,
                targets,
                root_caller,
                root_children_pct,
                target_stack,
                new_cumulative,
                seen,
                still_inside_root_recursion,
                current_path,
                exact_mode,
            );
            relations.extend(deeper);

            // T011: Pop the intermediary after returning from subtree
            if !is_root_recursion {
                current_path.pop();
            }
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
/// Now returns both direct relations and context-specific nested relations.
/// - `exact_mode = true`: targets are exact signatures, use equality matching after simplification
/// - `exact_mode = false`: targets are substrings, use contains matching
pub fn compute_call_relations(
    trees: &[(PerfEntry, Vec<CallTreeNode>)],
    targets: &[String],
    exact_mode: bool,
) -> Vec<CallRelation> {
    let mut all_relations = Vec::new();

    for (entry, tree_roots) in trees {
        // Check if this entry is a target
        // For exact mode, we need to simplify the entry symbol to compare
        let entry_simplified = simplify_symbol(&entry.symbol);
        let is_target = matches_any_target(&entry_simplified, targets, exact_mode);

        if is_target {
            // Skip leaf functions - their call tree shows callers, not callees
            if is_leaf_function(entry) {
                continue;
            }

            // This entry is a caller, look for callees (including nested ones)
            for root in tree_roots {
                let mut seen = HashSet::new();
                seen.insert(entry.symbol.clone()); // Prevent self-recursion
                let mut target_stack = Vec::new(); // Track intermediate targets
                let mut current_path = Vec::new(); // T011: Track intermediary path

                // The root node is already one level deep in the call tree.
                // Check if it's a non-target intermediary that needs to be tracked.
                let root_is_target = matches_any_target(&root.symbol, targets, exact_mode);

                if !root_is_target {
                    // Root is a non-target intermediary - add to path
                    current_path.push(IntermediaryStep {
                        symbol: root.symbol.clone(),
                        percentage: root.relative_pct,
                    });
                }

                // Start cumulative at root's percentage (not 100%)
                // since root is already a child of the entry
                let relations = find_target_callees(
                    root,
                    targets,
                    &entry.symbol,
                    entry.children_pct,
                    &mut target_stack,
                    root.relative_pct, // Start at root's percentage
                    &mut seen,
                    true, // Start inside root caller's "recursion zone"
                    &mut current_path,
                    exact_mode,
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
/// - `exact_mode = true`: targets are exact signatures, use equality matching after simplification
/// - `exact_mode = false`: targets are substrings, use contains matching
pub fn build_hierarchy_entries(
    entries: &[PerfEntry],
    targets: &[String],
    relations: &[CallRelation],
    exact_mode: bool,
) -> Vec<HierarchyEntry> {
    let mut result = Vec::new();

    // Track which simplified symbols we've already added to avoid duplicates
    // (e.g., rd_optimize_transform appears twice with 71.80% and 71.78%)
    let mut added_symbols: HashSet<String> = HashSet::new();

    // Collect unique callers from relations (these are the "root" callers)
    let callers: HashSet<String> = relations.iter().map(|r| r.caller.clone()).collect();

    for entry in entries {
        // Simplify the symbol for matching and deduplication
        let simplified = simplify_symbol(&entry.symbol);

        // Check if this entry matches any target
        let is_target = matches_any_target(&simplified, targets, exact_mode);
        if !is_target {
            continue;
        }

        // Skip if we've already added an entry with this simplified symbol
        if added_symbols.contains(&simplified) {
            continue;
        }

        // Find OVERALL callees for this entry (context_root = None means from entry's own tree)
        // These are used for standalone display and remainder calculations
        // Deduplicate by callee symbol, keeping only unique callees
        let mut callees: Vec<CallRelation> = Vec::new();
        let mut seen_callees: HashSet<String> = HashSet::new();
        // Match relations where this entry is the caller
        // r.caller is simplified (from parse_file_call_trees), compare with simplified entry symbol
        for r in relations.iter().filter(|r| {
            r.caller == simplified && r.context_root.is_none()
        }) {
            if !seen_callees.contains(&r.callee) {
                seen_callees.insert(r.callee.clone());
                callees.push(r.clone());
            }
        }

        // Find contributions TO this entry (when it's a callee)
        // Group by caller and take MAX absolute_pct per caller
        // (same caller->callee pair may appear multiple times from different contexts)
        let mut contribution_by_caller: std::collections::HashMap<String, f64> =
            std::collections::HashMap::new();
        for r in relations.iter() {
            if simplified == r.callee {
                let entry = contribution_by_caller
                    .entry(r.caller.clone())
                    .or_insert(0.0);
                if r.absolute_pct > *entry {
                    *entry = r.absolute_pct;
                }
            }
        }

        // Build contributions breakdown for debug mode
        let contributions_breakdown: Vec<CallerContribution> = contribution_by_caller
            .iter()
            .map(|(caller, &pct)| CallerContribution {
                caller: caller.clone(),
                absolute_pct: pct,
            })
            .collect();

        let contribution_values: Vec<f64> = contribution_by_caller.values().copied().collect();
        let adjusted = compute_adjusted_percentage(entry.children_pct, &contribution_values);

        // Determine if this entry is a caller (has callees) or just a callee
        let is_caller = !callees.is_empty();

        // If this is purely a callee (not a caller), check if it's called by another target
        // and only show it as standalone if it has unique standalone time
        // callers contains simplified symbols, compare with simplified entry symbol
        let is_callee_of_target = callers.contains(&simplified);

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
            contributions: contributions_breakdown,
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

    // T007: Unit test for IntermediaryStep struct creation
    #[test]
    fn test_intermediary_step_creation() {
        let step = IntermediaryStep {
            symbol: "do_4d_transform".to_string(),
            percentage: 42.0,
        };
        assert_eq!(step.symbol, "do_4d_transform");
        assert!((step.percentage - 42.0).abs() < 0.01);
    }

    // T007: Test IntermediaryStep in CallRelation
    #[test]
    fn test_call_relation_with_intermediary_path() {
        let relation = CallRelation {
            caller: "rd_optimize".to_string(),
            callee: "inner_product".to_string(),
            relative_pct: 7.23,
            absolute_pct: 5.19,
            context_root: None,
            intermediary_path: vec![IntermediaryStep {
                symbol: "do_4d_transform".to_string(),
                percentage: 42.0,
            }],
            callee_direct_pct: 17.2, // inner_product's % relative to do_4d_transform
        };
        assert_eq!(relation.intermediary_path.len(), 1);
        assert_eq!(relation.intermediary_path[0].symbol, "do_4d_transform");
        assert!((relation.callee_direct_pct - 17.2).abs() < 0.01);
    }

    // T007: Test empty intermediary_path (direct call)
    #[test]
    fn test_call_relation_direct_call() {
        let relation = CallRelation {
            caller: "rd_optimize".to_string(),
            callee: "DCT4DBlock".to_string(),
            relative_pct: 17.23,
            absolute_pct: 12.37,
            context_root: None,
            intermediary_path: vec![],
            callee_direct_pct: 17.23, // Same as relative_pct for direct calls
        };
        assert!(relation.intermediary_path.is_empty());
        assert!((relation.callee_direct_pct - 17.23).abs() < 0.01);
    }
}
