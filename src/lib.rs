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
    /// T046: --hierarchy requires --targets
    HierarchyRequiresTargets,
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
}
