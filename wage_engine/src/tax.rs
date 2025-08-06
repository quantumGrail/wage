//! Tax calculation traits and structures.
//!
//! The `tax` module defines abstractions for calculating taxes in
//! different regions.  It provides the `TaxCalculator` trait, which
//! individual jurisdictions implement, and helper types for loading
//! tax law from versioned JSON files.

use crate::models::{Employee, PayFrequency};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents the tax law for a particular region at a specific
/// version.  Tax laws are expected to be stored externally as JSON
/// files.  The engine does not impose a rigid schema on the
/// contents; instead it provides the raw `serde_json::Value` via the
/// `rules` field.  Implementors of [`TaxCalculator`] are responsible
/// for interpreting the rules according to their needs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxLaw {
    /// A region code such as `"US-OK"` or `"CA-ON"`.
    pub region: String,
    /// Version string, e.g. `"2025"` or `"2024-Q1"`.  Versions
    /// correspond to named JSON files stored under `tax_laws/`.
    pub version: String,
    /// Arbitrary JSON data containing tax rules for this region.
    pub rules: Value,
}

/// A tax calculator determines how much tax to withhold from a gross
/// amount for a given employee.  Each jurisdiction (state, province,
/// country) should provide its own implementation.
///
/// Tax calculators must be thread‑safe (`Send + Sync`) because the
/// engine may invoke them concurrently across multiple threads.
pub trait TaxCalculator: Send + Sync {
    /// Returns the canonical region code (e.g. `"US-OK"`).
    fn region_code(&self) -> &str;
    /// Calculates the tax for the provided `employee` given their gross
    /// earnings for this pay period.  Implementations may use data
    /// from the employee record (e.g. exemptions) and the associated
    /// [`TaxLaw`] to compute the amount.  The returned value should
    /// represent the total tax withheld.
    fn calculate(&self, employee: &Employee, gross: f64, law: &TaxLaw) -> f64;
}

/// Load all tax law definitions from a directory.
///
/// This helper scans a directory and attempts to parse any `.json`
/// files as [`TaxLaw`] objects.  The returned vector contains one
/// entry per file.  Duplicate region/version combinations are not
/// checked; if you need deduplication you should perform it on the
/// caller side.
pub fn load_tax_laws_from_dir(path: &std::path::Path) -> Result<Vec<TaxLaw>> {
    let mut laws = Vec::new();
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            if entry.file_type()?.is_file() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "json" {
                        let data = std::fs::read_to_string(entry.path())?;
                        match serde_json::from_str::<TaxLaw>(&data) {
                            Ok(law) => laws.push(law),
                            Err(err) => {
                                eprintln!("Failed to parse tax law {:?}: {}", entry.path(), err);
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(laws)
}

/// A very simple example tax calculator for US federal taxes.  It
/// interprets the `rules` field of the [`TaxLaw`] as containing a
/// key `"rate"` specifying the flat tax rate to apply.  In reality,
/// federal taxes use graduated brackets and personal deductions, so
/// this implementation should not be used for production systems.
pub struct UsFederalCalculator;

impl TaxCalculator for UsFederalCalculator {
    fn region_code(&self) -> &str {
        "US-FED"
    }

    fn calculate(&self, _employee: &Employee, gross: f64, law: &TaxLaw) -> f64 {
        // Interpret the tax law's rules as an object with a `rate` field
        // representing the federal tax rate (e.g. 0.12 for 12%).  If
        // parsing fails, default to 0.  See `tax_laws/us_federal_2025.json`
        // for an example.
        let rate = law
            .rules
            .get("rate")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        gross * rate
    }
}

/// A simple state tax calculator that applies a flat rate from the
/// provided tax law.  Like `UsFederalCalculator`, this is only a
/// demonstration; real state taxes may involve brackets, credits and
/// numerous special cases.
pub struct FlatStateCalculator {
    pub region: String,
}

impl TaxCalculator for FlatStateCalculator {
    fn region_code(&self) -> &str {
        &self.region
    }

    fn calculate(&self, _employee: &Employee, gross: f64, law: &TaxLaw) -> f64 {
        let rate = law
            .rules
            .get("rate")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        gross * rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Employee, PayFrequency};
    use serde_json::json;

    #[test]
    fn test_us_federal_calculator() {
        let calc = UsFederalCalculator;
        let law = TaxLaw {
            region: "US-FED".into(),
            version: "2025".into(),
            rules: json!({"rate": 0.1}),
        };
        let employee = Employee {
            id: "1".into(),
            name: "Test".into(),
            home_region: "US-OK".into(),
            pay_rate: 100.0,
            pay_frequency: PayFrequency::Salary,
        };
        let tax = calc.calculate(&employee, 1000.0, &law);
        assert_eq!(tax, 100.0);
    }
}