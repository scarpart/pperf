use std::env;
use std::path::Path;
use std::process;

use pperf::PperfError;
use pperf::output::format_table;
use pperf::parser::{SortOrder, parse_file, sort_entries};

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
        };
        process::exit(exit_code);
    }
}

fn run_top(args: &[String]) -> Result<(), PperfError> {
    let mut sort_order = SortOrder::Children;
    let mut count: usize = 10;
    let mut file_path: Option<&str> = None;
    let mut targets: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--self" | "-s" => {
                sort_order = SortOrder::Self_;
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

    let file_path = file_path.ok_or_else(|| {
        eprintln!("Usage: pperf top [--self] [-n <count>] [--targets <names>...] <file>");
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

    let display_entries: Vec<_> = entries.into_iter().take(count).collect();
    let output = format_table(&display_entries);
    print!("{}", output);

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
    println!("    --targets, -t <N>... Filter by function name prefixes");
    println!("    --help, -h           Show this help message");
    println!("    --version            Show version information");
}
