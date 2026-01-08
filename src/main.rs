use std::fs;
use std::path::PathBuf;
use std::process;

use clap::{Args, Parser, Subcommand};

use pperf::PperfError;
use pperf::hierarchy::{build_hierarchy_entries, compute_call_relations, parse_file_call_trees};
use pperf::output::{format_hierarchy_table, format_table};
use pperf::parser::{SortOrder, parse_file, sort_entries};
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
    #[arg(short = 't', long = "targets", conflicts_with = "target_file")]
    targets: Vec<String>,

    /// File containing exact function signatures (one per line)
    #[arg(long = "target-file", conflicts_with = "targets")]
    target_file: Option<PathBuf>,

    /// Display call relationships between targets
    #[arg(short = 'H', long = "hierarchy")]
    hierarchy: bool,

    /// Show calculation path for hierarchy percentages
    #[arg(short = 'D', long = "debug")]
    debug: bool,

    /// Disable colored output
    #[arg(long = "no-color")]
    no_color: bool,

    /// Perf report file to analyze
    file: PathBuf,
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
            PperfError::TargetFileNotFound(_) => 1,
            PperfError::InvalidFormat => 2,
            PperfError::InvalidCount => 3,
            PperfError::HierarchyRequiresTargets => 3,
            PperfError::NoMatches => 4,
            PperfError::AmbiguousTarget { .. } => 5,
            PperfError::UnmatchedTargets(_) => 6,
            PperfError::EmptyTargetFile => 6,
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
    let target_file = args.target_file;
    let hierarchy_flag = args.hierarchy;
    let debug_flag = args.debug;
    let no_color_flag = args.no_color;

    // Parse target file if provided
    let exact_signatures = if let Some(ref tf_path) = target_file {
        Some(pperf::filter::parse_target_file(tf_path)?)
    } else {
        None
    };

    // Validate --hierarchy requires targets (either -t or --target-file)
    let has_targets = !targets.is_empty() || exact_signatures.is_some();
    if hierarchy_flag && !has_targets {
        return Err(PperfError::HierarchyRequiresTargets);
    }

    let path = &args.file;
    let mut entries = parse_file(path)?;

    // Apply filtering based on mode
    if let Some(ref signatures) = exact_signatures {
        // Check for signatures that don't match any entries
        let unmatched = pperf::filter::detect_unmatched_targets(&entries, signatures);
        if !unmatched.is_empty() {
            return Err(PperfError::UnmatchedTargets(unmatched));
        }

        // Validate that each signature matches exactly one unique symbol
        pperf::filter::validate_unique_matches(&entries, signatures)?;

        // Exact matching mode (--target-file)
        entries = pperf::filter::filter_entries_exact(&entries, signatures);
        if entries.is_empty() {
            return Err(PperfError::NoMatches);
        }
    } else if !targets.is_empty() {
        // Substring matching mode (-t)
        entries = pperf::filter::filter_entries(&entries, &targets);
        if entries.is_empty() {
            return Err(PperfError::NoMatches);
        }
    }

    sort_entries(&mut entries, sort_order);

    let use_color = should_use_color(no_color_flag);

    // Wire hierarchy computation when --hierarchy is specified
    if hierarchy_flag {
        // Determine which targets to use for hierarchy
        let hierarchy_targets: Vec<String> = if let Some(ref signatures) = exact_signatures {
            signatures.clone()
        } else {
            targets.clone()
        };

        // Read file content for call tree parsing
        let content = fs::read_to_string(path)
            .map_err(|_| PperfError::FileNotFound(path.display().to_string()))?;

        // Parse call trees from content
        let trees = parse_file_call_trees(&content, &entries);

        // Compute relationships between targets
        let relations = compute_call_relations(&trees, &hierarchy_targets);

        // Build hierarchy entries with adjusted percentages
        let hierarchy_entries = build_hierarchy_entries(&entries, &hierarchy_targets, &relations);

        // Format and output
        let display_entries: Vec<_> = hierarchy_entries.into_iter().take(count).collect();
        let output = format_hierarchy_table(&display_entries, &relations, use_color, debug_flag);
        print!("{}", output);
    } else {
        let display_entries: Vec<_> = entries.into_iter().take(count).collect();
        let output = format_table(&display_entries, use_color);
        print!("{}", output);
    }

    Ok(())
}
