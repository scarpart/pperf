use std::env;
use std::fs;
use std::path::Path;
use std::process;

use pperf::PperfError;
use pperf::hierarchy::{build_hierarchy_entries, compute_call_relations, parse_file_call_trees};
use pperf::output::{format_hierarchy_table, format_table};
use pperf::parser::{SortOrder, parse_file, sort_entries};
use pperf::symbol::should_use_color;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: pperf <subcommand> [options] <file>");
        eprintln!("Subcommands: top");
        process::exit(3);
    }

    let result = match args[1].as_str() {
        "top" => run_top(&args[2..]),
        "--help" | "-h" => {
            print_help();
            Ok(())
        }
        "--version" => {
            println!("pperf 0.1.0");
            Ok(())
        }
        _ => {
            eprintln!("Unknown subcommand: {}", args[1]);
            process::exit(3);
        }
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

fn run_top(args: &[String]) -> Result<(), PperfError> {
    let mut sort_order = SortOrder::Children;
    let mut count: usize = 10;
    let mut file_path: Option<&str> = None;
    let mut targets: Vec<String> = Vec::new();
    let mut no_color_flag = false;
    // T044: Add --hierarchy flag parsing
    let mut hierarchy_flag = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--self" | "-s" => {
                sort_order = SortOrder::Self_;
            }
            "--no-color" => {
                no_color_flag = true;
            }
            // T044: Parse --hierarchy / -H flag
            "--hierarchy" | "-H" => {
                hierarchy_flag = true;
            }
            "-n" | "--number" => {
                i += 1;
                if i >= args.len() {
                    return Err(PperfError::InvalidCount);
                }
                count = args[i].parse().map_err(|_| PperfError::InvalidCount)?;
                if count == 0 {
                    return Err(PperfError::InvalidCount);
                }
            }
            "-t" | "--targets" => {
                i += 1;
                while i < args.len() && !args[i].starts_with('-') {
                    if Path::new(&args[i]).exists() {
                        file_path = Some(&args[i]);
                        break;
                    }
                    targets.push(args[i].clone());
                    i += 1;
                }
                continue;
            }
            arg if arg.starts_with('-') => {
                eprintln!("Unknown option: {}", arg);
                return Err(PperfError::InvalidCount);
            }
            _ => {
                file_path = Some(&args[i]);
            }
        }
        i += 1;
    }

    // T045: Validate --hierarchy requires --targets
    if hierarchy_flag && targets.is_empty() {
        return Err(PperfError::HierarchyRequiresTargets);
    }

    let file_path = file_path.ok_or_else(|| {
        eprintln!(
            "Usage: pperf top [--self] [-n <count>] [--targets <names>...] [--hierarchy] <file>"
        );
        PperfError::InvalidCount
    })?;

    let path = Path::new(file_path);
    let mut entries = parse_file(path)?;

    if !targets.is_empty() {
        entries = pperf::filter::filter_entries(&entries, &targets);
        if entries.is_empty() {
            return Err(PperfError::NoMatches);
        }
    }

    sort_entries(&mut entries, sort_order);

    let use_color = should_use_color(no_color_flag);

    // T048: Wire hierarchy computation when --hierarchy is specified
    if hierarchy_flag {
        // Read file content for call tree parsing
        let content = fs::read_to_string(path)
            .map_err(|_| PperfError::FileNotFound(path.display().to_string()))?;

        // Parse call trees from content
        let trees = parse_file_call_trees(&content, &entries);

        // Compute relationships between targets
        let relations = compute_call_relations(&trees, &targets);

        // Build hierarchy entries with adjusted percentages
        let hierarchy_entries = build_hierarchy_entries(&entries, &targets, &relations);

        // Format and output
        let display_entries: Vec<_> = hierarchy_entries.into_iter().take(count).collect();
        let output = format_hierarchy_table(&display_entries, &relations, use_color);
        print!("{}", output);
    } else {
        let display_entries: Vec<_> = entries.into_iter().take(count).collect();
        let output = format_table(&display_entries, use_color);
        print!("{}", output);
    }

    Ok(())
}

fn print_help() {
    println!("pperf - Perf report analyzer");
    println!();
    println!("USAGE:");
    println!("    pperf <SUBCOMMAND> [OPTIONS] <FILE>");
    println!();
    println!("SUBCOMMANDS:");
    println!("    top     Display top functions by CPU time");
    println!();
    println!("OPTIONS:");
    println!("    --self, -s           Sort by Self% instead of Children%");
    println!("    -n, --number <N>     Number of functions to display (default: 10)");
    println!("    --targets, -t <N>... Filter by function name substrings");
    // T049: Document --hierarchy flag in help text
    println!("    --hierarchy, -H      Display call relationships between targets");
    println!("    --no-color           Disable colored output");
    println!("    --help, -h           Show this help message");
    println!("    --version            Show version information");
}
