use std::fs;
use std::path::Path;

use crate::PperfError;

#[derive(Debug, Clone, PartialEq)]
pub struct PerfEntry {
    pub children_pct: f64,
    pub self_pct: f64,
    pub symbol: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SortOrder {
    Children,
    Self_,
}

pub fn parse_line(line: &str) -> Option<PerfEntry> {
    let trimmed = line.trim_start();

    if trimmed.starts_with('#') || trimmed.is_empty() {
        return None;
    }

    if trimmed.starts_with('|') || trimmed.starts_with('-') {
        return None;
    }

    if !trimmed.chars().next()?.is_ascii_digit() {
        return None;
    }

    let pct_end = trimmed.find('%')?;
    let children_str = &trimmed[..pct_end];
    let children_pct: f64 = children_str.trim().parse().ok()?;

    let rest = &trimmed[pct_end + 1..].trim_start();
    let pct_end2 = rest.find('%')?;
    let self_str = &rest[..pct_end2];
    let self_pct: f64 = self_str.trim().parse().ok()?;

    let after_self = &rest[pct_end2 + 1..].trim_start();

    let symbol = if let Some(marker_pos) = after_self.find("[.] ") {
        after_self[marker_pos + 4..].to_string()
    } else if let Some(marker_pos) = after_self.find("[k] ") {
        after_self[marker_pos + 4..].to_string()
    } else {
        let parts: Vec<&str> = after_self.split_whitespace().collect();
        if parts.len() >= 2 {
            parts[parts.len() - 1].to_string()
        } else {
            return None;
        }
    };

    Some(PerfEntry {
        children_pct,
        self_pct,
        symbol,
    })
}

pub fn parse_file(path: &Path) -> Result<Vec<PerfEntry>, PperfError> {
    let content = fs::read_to_string(path)
        .map_err(|_| PperfError::FileNotFound(path.display().to_string()))?;

    let entries: Vec<PerfEntry> = content.lines().filter_map(parse_line).collect();

    if entries.is_empty() {
        return Err(PperfError::InvalidFormat);
    }

    Ok(entries)
}

pub fn sort_entries(entries: &mut [PerfEntry], order: SortOrder) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perf_entry_creation() {
        let entry = PerfEntry {
            children_pct: 90.74,
            self_pct: 0.00,
            symbol: "test_function".to_string(),
        };
        assert_eq!(entry.children_pct, 90.74);
        assert_eq!(entry.self_pct, 0.00);
        assert_eq!(entry.symbol, "test_function");
    }

    #[test]
    fn test_sort_order_variants() {
        let children = SortOrder::Children;
        let self_order = SortOrder::Self_;
        assert!(matches!(children, SortOrder::Children));
        assert!(matches!(self_order, SortOrder::Self_));
    }

    #[test]
    fn test_parse_file_real_data() {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/perf-report.txt");

        let entries = parse_file(&path).expect("Failed to parse perf report file");

        assert!(
            entries.len() > 1,
            "Expected multiple entries, got {}",
            entries.len()
        );

        let first = &entries[0];
        assert!(
            (first.children_pct - 90.74).abs() < 0.01,
            "Expected first entry children_pct ~90.74, got {}",
            first.children_pct
        );

        for (i, entry) in entries.iter().enumerate() {
            assert!(!entry.symbol.is_empty(), "Entry {} has empty symbol", i);
        }
    }

    #[test]
    fn test_parse_line_valid_data() {
        let line = "    90.74%     0.00%  jpl-encoder-bin  jpl-encoder-bin      [.] parallel_for_with_progress";
        let result = parse_line(line);
        assert!(result.is_some(), "Expected Some for valid data line");
        let entry = result.unwrap();
        assert_eq!(entry.children_pct, 90.74);
        assert_eq!(entry.self_pct, 0.00);
        assert_eq!(entry.symbol, "parallel_for_with_progress");
    }

    #[test]
    fn test_parse_line_skip_comments() {
        let comment_line = "# Overhead  Command          Shared Object        Symbol";
        let result = parse_line(comment_line);
        assert!(result.is_none(), "Expected None for comment line");

        let another_comment = "#   Children      Self  Command   Shared Object       Symbol";
        let result2 = parse_line(another_comment);
        assert!(result2.is_none(), "Expected None for header comment line");
    }

    #[test]
    fn test_parse_line_skip_call_tree() {
        let pipe_line = "            |          ";
        let result1 = parse_line(pipe_line);
        assert!(
            result1.is_none(),
            "Expected None for pipe-indented call tree line"
        );

        let dashes_line = "            ---parallel_for_with_progress";
        let result2 = parse_line(dashes_line);
        assert!(result2.is_none(), "Expected None for dashes call tree line");

        let deep_indent_line = "                                     run_for_block_4d";
        let result3 = parse_line(deep_indent_line);
        assert!(
            result3.is_none(),
            "Expected None for deeply indented call tree line"
        );
    }

    #[test]
    fn test_sort_entries_by_self() {
        let mut entries = vec![
            PerfEntry {
                children_pct: 90.0,
                self_pct: 1.0,
                symbol: "a".to_string(),
            },
            PerfEntry {
                children_pct: 50.0,
                self_pct: 10.0,
                symbol: "b".to_string(),
            },
            PerfEntry {
                children_pct: 30.0,
                self_pct: 5.0,
                symbol: "c".to_string(),
            },
        ];
        sort_entries(&mut entries, SortOrder::Self_);
        assert_eq!(entries[0].self_pct, 10.0);
        assert_eq!(entries[1].self_pct, 5.0);
        assert_eq!(entries[2].self_pct, 1.0);
    }

    #[test]
    fn test_sort_entries_by_self_tiebreaker() {
        let mut entries = vec![
            PerfEntry {
                children_pct: 30.0,
                self_pct: 5.0,
                symbol: "a".to_string(),
            },
            PerfEntry {
                children_pct: 90.0,
                self_pct: 5.0,
                symbol: "b".to_string(),
            },
            PerfEntry {
                children_pct: 50.0,
                self_pct: 5.0,
                symbol: "c".to_string(),
            },
        ];
        sort_entries(&mut entries, SortOrder::Self_);
        assert_eq!(entries[0].children_pct, 90.0);
        assert_eq!(entries[1].children_pct, 50.0);
        assert_eq!(entries[2].children_pct, 30.0);
    }
}
