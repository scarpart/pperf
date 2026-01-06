use std::fs;
use std::path::PathBuf;
use std::process;

use clap::{Args, Parser, Subcommand};

use pperf::PperfError;
use pperf::hierarchy::{
    build_hierarchy_entries, compute_averaged_call_relations, compute_call_relations,
    parse_file_call_trees, parse_multi_file_call_trees,
};
use pperf::output::format_hierarchy_table;
use pperf::parser::SortOrder;
use pperf::symbol::should_use_color;

/// Parse count argument, ensuring it's >= 1
fn parse_count(s: &str) -> Result<usize, String> {
    let count: usize = s
        .parse()
        .map_err(|_| format!("'{}' is not a valid number", s))?;
    if count == 0 {
        Err("number must be at least 1".to_string())
    } else {
        Ok(count)
    }
}

/// Perf report analyzer
#[derive(Parser)]
#[command(name = "pperf", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Display top functions by CPU time
    Top(TopArgs),
}

#[derive(Args)]
struct TopArgs {
    /// Sort by Self% instead of Children%
    #[arg(short = 's', long = "self")]
    sort_self: bool,

    /// Number of functions to display
    #[arg(short = 'n', long = "number", default_value = "10", value_parser = parse_count)]
    number: usize,

    /// Filter by function name substrings (repeatable: -t val1 -t val2)
    #[arg(short = 't', long = "targets")]
    targets: Vec<String>,

    /// Display call relationships between targets
    #[arg(short = 'H', long = "hierarchy")]
    hierarchy: bool,

    /// Show calculation path for hierarchy percentages
    #[arg(short = 'D', long = "debug")]
    debug: bool,

    /// Disable colored output
    #[arg(long = "no-color")]
    no_color: bool,

    /// Perf report file(s) to analyze (multiple files are averaged)
    #[arg(required = true)]
    files: Vec<PathBuf>,
}

fn main() {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            e.print().expect("Failed to print error");
            // Use Clap's exit code for help/version (0), otherwise use 3 for arg errors
            let exit_code = if e.use_stderr() { 3 } else { 0 };
            process::exit(exit_code);
        }
    };

    let result = match cli.command {
        Commands::Top(args) => run_top(args),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        let exit_code = match e {
            PperfError::FileNotFound(_) => 1,
            PperfError::InvalidFormat => 2,
            PperfError::InvalidCount => 3,
            PperfError::NoMatches => 4,
            PperfError::HierarchyRequiresTargets => 3,
        };
        process::exit(exit_code);
    }
}

fn run_top(args: TopArgs) -> Result<(), PperfError> {
    // Map Clap args to existing variable names
    let sort_order = if args.sort_self {
        SortOrder::Self_
    } else {
        SortOrder::Children
    };
    let count = args.number;
    let targets = args.targets;
    let hierarchy_flag = args.hierarchy;
    let debug_flag = args.debug;
    let no_color_flag = args.no_color;

    // Validate --hierarchy requires --targets
    if hierarchy_flag && targets.is_empty() {
        return Err(PperfError::HierarchyRequiresTargets);
    }

    let use_color = should_use_color(no_color_flag);
    let files = &args.files;
    let is_multi_file = files.len() > 1;

    // Parse all files and compute averaged entries
    let report_set = pperf::averaging::ReportSet::parse_all(files)?;
    let mut averaged_entries = report_set.average();

    // Apply target filtering if specified
    if !targets.is_empty() {
        averaged_entries = pperf::filter::filter_averaged_entries(&averaged_entries, &targets);
        if averaged_entries.is_empty() {
            return Err(PperfError::NoMatches);
        }
    }

    // Sort entries
    sort_averaged_entries(&mut averaged_entries, sort_order);

    // T048: Wire hierarchy computation when --hierarchy is specified
    if hierarchy_flag {
        // Convert averaged entries back to PerfEntry for hierarchy functions
        let entries: Vec<_> = averaged_entries
            .iter()
            .map(|e| pperf::parser::PerfEntry {
                children_pct: e.children_pct,
                self_pct: e.self_pct,
                symbol: e.symbol.clone(),
            })
            .collect();

        // Read file contents for call tree parsing
        let file_contents: Result<Vec<String>, PperfError> = files
            .iter()
            .map(|path| {
                fs::read_to_string(path)
                    .map_err(|_| PperfError::FileNotFound(path.display().to_string()))
            })
            .collect();
        let file_contents = file_contents?;

        let relations = if is_multi_file {
            // Parse call trees from all files and compute averaged relations
            let all_trees = parse_multi_file_call_trees(&file_contents, &averaged_entries);
            compute_averaged_call_relations(&all_trees, &targets)
        } else {
            // Single file: use existing logic
            let trees = parse_file_call_trees(&file_contents[0], &entries);
            compute_call_relations(&trees, &targets)
        };

        // Build hierarchy entries with adjusted percentages
        let hierarchy_entries = build_hierarchy_entries(&entries, &targets, &relations);

        // Format and output (T005: pass debug_flag to format_hierarchy_table)
        let display_entries: Vec<_> = hierarchy_entries.into_iter().take(count).collect();
        let output = format_hierarchy_table(&display_entries, &relations, use_color, debug_flag);
        print!("{}", output);
    } else {
        let display_entries: Vec<_> = averaged_entries.into_iter().take(count).collect();
        let output = format_averaged_table(&display_entries, use_color, debug_flag, is_multi_file);
        print!("{}", output);
    }

    Ok(())
}

/// Sort averaged entries by the specified order.
fn sort_averaged_entries(entries: &mut [pperf::averaging::AveragedPerfEntry], order: SortOrder) {
    match order {
        SortOrder::Children => {
            entries.sort_by(|a, b| {
                b.children_pct
                    .partial_cmp(&a.children_pct)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }
        SortOrder::Self_ => {
            entries.sort_by(|a, b| {
                let primary = b
                    .self_pct
                    .partial_cmp(&a.self_pct)
                    .unwrap_or(std::cmp::Ordering::Equal);
                if primary == std::cmp::Ordering::Equal {
                    b.children_pct
                        .partial_cmp(&a.children_pct)
                        .unwrap_or(std::cmp::Ordering::Equal)
                } else {
                    primary
                }
            });
        }
    }
}

/// Format table for averaged entries with optional per-report breakdown in debug mode.
fn format_averaged_table(
    entries: &[pperf::averaging::AveragedPerfEntry],
    use_color: bool,
    debug: bool,
    is_multi_file: bool,
) -> String {
    use pperf::output::truncate_symbol;
    use pperf::symbol::format_colored_symbol;

    let mut output = String::new();
    output.push_str("Children%   Self%  Function\n");

    for entry in entries {
        let symbol = truncate_symbol(&entry.symbol, 100);
        let colored_symbol = format_colored_symbol(&symbol, use_color);
        output.push_str(&format!(
            "{:>8.2}  {:>6.2}  {}\n",
            entry.children_pct, entry.self_pct, colored_symbol
        ));

        // Show per-report values in debug mode for multi-file
        if debug && is_multi_file {
            let annotation = format_per_report_values(&entry.per_report_values, use_color);
            if !annotation.is_empty() {
                output.push_str(&format!("                  {}\n", annotation));
            }
        }
    }

    output
}

/// Format per-report values annotation.
/// Format: (values: 73.86%, 73.60%, 70.40%) or (values: 73.86%, -, 70.40%) for missing
fn format_per_report_values(values: &[Option<(f64, f64)>], use_color: bool) -> String {
    use pperf::symbol::{DIM, RESET};

    let formatted: Vec<String> = values
        .iter()
        .map(|v| match v {
            Some((children, _)) => format!("{:.2}%", children),
            None => "-".to_string(),
        })
        .collect();

    let content = format!("(values: {})", formatted.join(", "));

    if use_color {
        format!("{}{}{}", DIM, content, RESET)
    } else {
        content
    }
}
