pub mod filter;
pub mod hierarchy;
pub mod output;
pub mod parser;
pub mod symbol;

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum PperfError {
    FileNotFound(String),
    InvalidFormat,
    InvalidCount,
    NoMatches,
    /// --hierarchy requires --targets
    HierarchyRequiresTargets,
    /// Target file specified but not found or unreadable
    TargetFileNotFound(String),
    /// Target file contains no valid signatures
    EmptyTargetFile,
    /// A signature matches multiple distinct functions
    AmbiguousTarget {
        signature: String,
        matches: Vec<String>,
    },
    /// One or more signatures match no entries
    UnmatchedTargets(Vec<String>),
}

impl fmt::Display for PperfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PperfError::FileNotFound(path) => write!(f, "File not found: {}", path),
            PperfError::InvalidFormat => write!(f, "Invalid perf report format"),
            PperfError::InvalidCount => {
                write!(f, "Invalid value for -n: expected positive integer")
            }
            PperfError::NoMatches => write!(f, "No matching functions found"),
            PperfError::HierarchyRequiresTargets => {
                write!(f, "--hierarchy requires --targets to be specified")
            }
            PperfError::TargetFileNotFound(path) => {
                write!(f, "Target file not found: {}", path)
            }
            PperfError::EmptyTargetFile => {
                write!(f, "Target file contains no valid signatures")
            }
            PperfError::AmbiguousTarget { signature, matches } => {
                writeln!(f, "Ambiguous target signature '{}'", signature)?;
                writeln!(f, "Matches:")?;
                for m in matches {
                    writeln!(f, "  - {}", m)?;
                }
                write!(f, "Use the complete function signature.")
            }
            PperfError::UnmatchedTargets(signatures) => {
                writeln!(f, "No matches found for target signatures:")?;
                for sig in signatures {
                    writeln!(f, "  - {}", sig)?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for PperfError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_file_not_found() {
        let err = PperfError::FileNotFound("test.txt".to_string());
        assert_eq!(format!("{}", err), "File not found: test.txt");
    }

    #[test]
    fn test_error_invalid_format() {
        let err = PperfError::InvalidFormat;
        assert_eq!(format!("{}", err), "Invalid perf report format");
    }

    #[test]
    fn test_error_invalid_count() {
        let err = PperfError::InvalidCount;
        assert_eq!(
            format!("{}", err),
            "Invalid value for -n: expected positive integer"
        );
    }

    #[test]
    fn test_error_no_matches() {
        let err = PperfError::NoMatches;
        assert_eq!(format!("{}", err), "No matching functions found");
    }

    #[test]
    fn test_error_hierarchy_requires_targets() {
        let err = PperfError::HierarchyRequiresTargets;
        assert_eq!(
            format!("{}", err),
            "--hierarchy requires --targets to be specified"
        );
    }

    #[test]
    fn test_error_target_file_not_found() {
        let err = PperfError::TargetFileNotFound("targets.txt".to_string());
        assert_eq!(format!("{}", err), "Target file not found: targets.txt");
    }

    #[test]
    fn test_error_empty_target_file() {
        let err = PperfError::EmptyTargetFile;
        assert_eq!(
            format!("{}", err),
            "Target file contains no valid signatures"
        );
    }

    #[test]
    fn test_error_ambiguous_target() {
        let err = PperfError::AmbiguousTarget {
            signature: "DCT4DBlock".to_string(),
            matches: vec![
                "DCT4DBlock::DCT4DBlock(Block4D const&, double)".to_string(),
                "DCT4DBlock::inverse(Block4D&) const".to_string(),
            ],
        };
        let msg = format!("{}", err);
        assert!(msg.contains("Ambiguous target signature 'DCT4DBlock'"));
        assert!(msg.contains("DCT4DBlock::DCT4DBlock(Block4D const&, double)"));
        assert!(msg.contains("DCT4DBlock::inverse(Block4D&) const"));
        assert!(msg.contains("Use the complete function signature."));
    }

    #[test]
    fn test_error_unmatched_targets() {
        let err = PperfError::UnmatchedTargets(vec![
            "NonExistent::function()".to_string(),
            "Another::missing()".to_string(),
        ]);
        let msg = format!("{}", err);
        assert!(msg.contains("No matches found for target signatures:"));
        assert!(msg.contains("NonExistent::function()"));
        assert!(msg.contains("Another::missing()"));
    }
}
