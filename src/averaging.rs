//! Multi-report averaging module.
//!
//! This module handles parsing multiple perf report files and computing
//! averaged metrics across all reports.

use std::collections::HashMap;
use std::path::PathBuf;

use crate::PperfError;
use crate::parser::{PerfEntry, parse_file};

/// Represents a function's aggregated profiling data across multiple reports.
#[derive(Debug, Clone, PartialEq)]
pub struct AveragedPerfEntry {
    /// Averaged Children% across reports
    pub children_pct: f64,
    /// Averaged Self% across reports
    pub self_pct: f64,
    /// Full function signature (unique key for matching)
    pub symbol: String,
    /// Individual (children, self) per report; None if missing from that report
    pub per_report_values: Vec<Option<(f64, f64)>>,
    /// Number of reports containing this function
    pub report_count: usize,
}

/// Collection of parsed reports for aggregation.
#[derive(Debug, Clone)]
pub struct ReportSet {
    /// Parsed entries from each file, in file order
    pub reports: Vec<Vec<PerfEntry>>,
    /// Original file paths (for error messages)
    pub file_paths: Vec<PathBuf>,
}

impl ReportSet {
    /// Parse all files and create a ReportSet.
    pub fn parse_all(paths: &[PathBuf]) -> Result<Self, PperfError> {
        let mut reports = Vec::with_capacity(paths.len());
        let file_paths = paths.to_vec();

        for path in paths {
            let entries = parse_file(path)?;
            reports.push(entries);
        }

        Ok(ReportSet {
            reports,
            file_paths,
        })
    }

    /// Compute averaged entries across all reports.
    ///
    /// Algorithm:
    /// 1. Aggregate entries by symbol into HashMap<String, Vec<Option<(f64, f64)>>>
    /// 2. For each symbol, compute average over present values only
    /// 3. Return Vec<AveragedPerfEntry> with all aggregated data
    pub fn average(&self) -> Vec<AveragedPerfEntry> {
        let report_count = self.reports.len();
        if report_count == 0 {
            return Vec::new();
        }

        // symbol -> per-report values (index corresponds to report index)
        let mut symbol_data: HashMap<String, Vec<Option<(f64, f64)>>> = HashMap::new();

        // Collect all symbols and their values from each report
        for (report_idx, entries) in self.reports.iter().enumerate() {
            for entry in entries {
                let per_report = symbol_data
                    .entry(entry.symbol.clone())
                    .or_insert_with(|| vec![None; report_count]);

                // Ensure the vector is long enough (in case of later reports adding new symbols)
                while per_report.len() < report_count {
                    per_report.push(None);
                }

                per_report[report_idx] = Some((entry.children_pct, entry.self_pct));
            }
        }

        // Convert to AveragedPerfEntry
        symbol_data
            .into_iter()
            .map(|(symbol, per_report_values)| {
                // Count present values and sum them
                let mut children_sum = 0.0;
                let mut self_sum = 0.0;
                let mut present_count = 0usize;

                for (children, self_pct) in per_report_values.iter().flatten() {
                    children_sum += children;
                    self_sum += self_pct;
                    present_count += 1;
                }

                // Compute averages
                let children_pct = if present_count > 0 {
                    children_sum / present_count as f64
                } else {
                    0.0
                };
                let self_pct = if present_count > 0 {
                    self_sum / present_count as f64
                } else {
                    0.0
                };

                AveragedPerfEntry {
                    children_pct,
                    self_pct,
                    symbol,
                    per_report_values,
                    report_count: present_count,
                }
            })
            .collect()
    }
}

/// Convert a single PerfEntry to AveragedPerfEntry (for single-file case).
impl From<PerfEntry> for AveragedPerfEntry {
    fn from(entry: PerfEntry) -> Self {
        AveragedPerfEntry {
            children_pct: entry.children_pct,
            self_pct: entry.self_pct,
            symbol: entry.symbol,
            per_report_values: vec![Some((entry.children_pct, entry.self_pct))],
            report_count: 1,
        }
    }
}

/// Convert a slice of PerfEntry to Vec<AveragedPerfEntry> (for single-file backward compat).
pub fn entries_to_averaged(entries: &[PerfEntry]) -> Vec<AveragedPerfEntry> {
    entries
        .iter()
        .cloned()
        .map(AveragedPerfEntry::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    // T006: Unit test: average_entries aggregates by symbol
    #[test]
    fn test_average_entries_aggregates_by_symbol() {
        let entries1 = vec![
            PerfEntry {
                children_pct: 70.0,
                self_pct: 10.0,
                symbol: "funcA".to_string(),
            },
            PerfEntry {
                children_pct: 30.0,
                self_pct: 5.0,
                symbol: "funcB".to_string(),
            },
        ];
        let entries2 = vec![
            PerfEntry {
                children_pct: 80.0,
                self_pct: 12.0,
                symbol: "funcA".to_string(),
            },
            PerfEntry {
                children_pct: 20.0,
                self_pct: 4.0,
                symbol: "funcB".to_string(),
            },
        ];

        let report_set = ReportSet {
            reports: vec![entries1, entries2],
            file_paths: vec![PathBuf::from("rep0.txt"), PathBuf::from("rep1.txt")],
        };

        let averaged = report_set.average();
        assert_eq!(averaged.len(), 2);

        // Find funcA
        let func_a = averaged.iter().find(|e| e.symbol == "funcA").unwrap();
        assert!((func_a.children_pct - 75.0).abs() < 0.01); // (70+80)/2
        assert!((func_a.self_pct - 11.0).abs() < 0.01); // (10+12)/2
        assert_eq!(func_a.report_count, 2);
    }

    // T007: Unit test: arithmetic mean calculation is correct
    #[test]
    fn test_arithmetic_mean_calculation() {
        let entries1 = vec![PerfEntry {
            children_pct: 73.86,
            self_pct: 0.0,
            symbol: "rd_optimize".to_string(),
        }];
        let entries2 = vec![PerfEntry {
            children_pct: 73.60,
            self_pct: 0.0,
            symbol: "rd_optimize".to_string(),
        }];
        let entries3 = vec![PerfEntry {
            children_pct: 70.40,
            self_pct: 0.0,
            symbol: "rd_optimize".to_string(),
        }];

        let report_set = ReportSet {
            reports: vec![entries1, entries2, entries3],
            file_paths: vec![
                PathBuf::from("rep0.txt"),
                PathBuf::from("rep1.txt"),
                PathBuf::from("rep2.txt"),
            ],
        };

        let averaged = report_set.average();
        let entry = averaged.iter().find(|e| e.symbol == "rd_optimize").unwrap();

        // (73.86 + 73.60 + 70.40) / 3 = 72.62
        assert!((entry.children_pct - 72.62).abs() < 0.01);
        assert_eq!(entry.report_count, 3);
        assert_eq!(entry.per_report_values.len(), 3);
    }

    // T008: Unit test: functions with same simplified name but different signatures are distinct
    #[test]
    fn test_different_signatures_are_distinct() {
        let entries1 = vec![
            PerfEntry {
                children_pct: 50.0,
                self_pct: 5.0,
                symbol: "func(int)".to_string(),
            },
            PerfEntry {
                children_pct: 30.0,
                self_pct: 3.0,
                symbol: "func(double)".to_string(),
            },
        ];
        let entries2 = vec![
            PerfEntry {
                children_pct: 60.0,
                self_pct: 6.0,
                symbol: "func(int)".to_string(),
            },
            PerfEntry {
                children_pct: 40.0,
                self_pct: 4.0,
                symbol: "func(double)".to_string(),
            },
        ];

        let report_set = ReportSet {
            reports: vec![entries1, entries2],
            file_paths: vec![PathBuf::from("rep0.txt"), PathBuf::from("rep1.txt")],
        };

        let averaged = report_set.average();
        assert_eq!(averaged.len(), 2);

        let func_int = averaged.iter().find(|e| e.symbol == "func(int)").unwrap();
        let func_double = averaged
            .iter()
            .find(|e| e.symbol == "func(double)")
            .unwrap();

        assert!((func_int.children_pct - 55.0).abs() < 0.01); // (50+60)/2
        assert!((func_double.children_pct - 35.0).abs() < 0.01); // (30+40)/2
    }

    // T009: Unit test: function present in only some reports averages over present count
    #[test]
    fn test_missing_function_averages_over_present_count() {
        let entries1 = vec![
            PerfEntry {
                children_pct: 60.0,
                self_pct: 6.0,
                symbol: "common".to_string(),
            },
            PerfEntry {
                children_pct: 30.0,
                self_pct: 3.0,
                symbol: "only_in_first".to_string(),
            },
        ];
        let entries2 = vec![PerfEntry {
            children_pct: 80.0,
            self_pct: 8.0,
            symbol: "common".to_string(),
        }];
        let entries3 = vec![
            PerfEntry {
                children_pct: 70.0,
                self_pct: 7.0,
                symbol: "common".to_string(),
            },
            PerfEntry {
                children_pct: 20.0,
                self_pct: 2.0,
                symbol: "only_in_third".to_string(),
            },
        ];

        let report_set = ReportSet {
            reports: vec![entries1, entries2, entries3],
            file_paths: vec![
                PathBuf::from("rep0.txt"),
                PathBuf::from("rep1.txt"),
                PathBuf::from("rep2.txt"),
            ],
        };

        let averaged = report_set.average();

        // "common" should average over 3 reports
        let common = averaged.iter().find(|e| e.symbol == "common").unwrap();
        assert!((common.children_pct - 70.0).abs() < 0.01); // (60+80+70)/3
        assert_eq!(common.report_count, 3);

        // "only_in_first" should average over 1 report (not divided by 3)
        let only_first = averaged
            .iter()
            .find(|e| e.symbol == "only_in_first")
            .unwrap();
        assert!((only_first.children_pct - 30.0).abs() < 0.01); // 30/1, NOT 30/3
        assert_eq!(only_first.report_count, 1);

        // Check per_report_values shows None for missing reports
        assert!(only_first.per_report_values[0].is_some());
        assert!(only_first.per_report_values[1].is_none());
        assert!(only_first.per_report_values[2].is_none());
    }

    // Test PerfEntry to AveragedPerfEntry conversion
    #[test]
    fn test_perf_entry_to_averaged_conversion() {
        let entry = PerfEntry {
            children_pct: 42.5,
            self_pct: 3.2,
            symbol: "test_func".to_string(),
        };

        let averaged: AveragedPerfEntry = entry.into();
        assert!((averaged.children_pct - 42.5).abs() < 0.01);
        assert!((averaged.self_pct - 3.2).abs() < 0.01);
        assert_eq!(averaged.symbol, "test_func");
        assert_eq!(averaged.report_count, 1);
        assert_eq!(averaged.per_report_values.len(), 1);
        assert!(averaged.per_report_values[0].is_some());
    }

    // Test entries_to_averaged helper
    #[test]
    fn test_entries_to_averaged() {
        let entries = vec![
            PerfEntry {
                children_pct: 50.0,
                self_pct: 5.0,
                symbol: "funcA".to_string(),
            },
            PerfEntry {
                children_pct: 30.0,
                self_pct: 3.0,
                symbol: "funcB".to_string(),
            },
        ];

        let averaged = entries_to_averaged(&entries);
        assert_eq!(averaged.len(), 2);
        assert_eq!(averaged[0].symbol, "funcA");
        assert_eq!(averaged[1].symbol, "funcB");
        assert_eq!(averaged[0].report_count, 1);
    }
}
