//! Payroll computation engine.
//!
//! The `engine` module is responsible for turning a [`PayRunInput`]
//! into a [`PayRunResult`].  It uses the [`rayon`] crate to
//! parallelise per‑employee calculations across multiple CPU cores.
//! Tax calculations are delegated to implementations of the
//! [`TaxCalculator`] trait, allowing each region to define its own
//! logic.

use crate::models::{EmployeePayResult, PayItem, PayRunInput, PayRunResult};
use crate::tax::{TaxCalculator, TaxLaw};
use anyhow::{anyhow, Result};
use rayon::prelude::*;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;

/// Runs a payroll for a given input and tax calculators.
///
/// `tax_laws` is a map from region codes to tax law definitions.
/// `calculators` maps region codes to the appropriate tax calculator.
pub fn run_payroll(
    input: PayRunInput,
    tax_laws: &HashMap<String, TaxLaw>,
    calculators: &HashMap<String, Arc<dyn TaxCalculator>>,
) -> Result<PayRunResult> {
    // Clone the inputs required inside the parallel closure
    let period = input.pay_period.clone();
    let pay_items = input.pay_items.clone();

    // Compute each employee's pay result in parallel
    let results: Vec<EmployeePayResult> = input
        .employees
        .into_par_iter()
        .map(|employee| {
            // Determine gross pay (base pay + pay items)
            let base_gross = match employee.pay_frequency {
                crate::models::PayFrequency::Hourly => {
                    // For hourly employees the `pay_rate` represents their hourly rate.
                    // We expect to find a pay item with description "hours" specifying
                    // the number of hours worked; otherwise assume zero.
                    let hours = pay_items
                        .get(&employee.id)
                        .and_then(|items| {
                            items.iter().find(|i| i.description.to_lowercase() == "hours")
                        })
                        .map(|item| item.amount)
                        .unwrap_or(0.0);
                    employee.pay_rate * hours
                }
                crate::models::PayFrequency::Salary => {
                    // For salaried employees the `pay_rate` already represents the
                    // salary per pay period.
                    employee.pay_rate
                }
            };
            let extra: f64 = pay_items
                .get(&employee.id)
                .map(|items| {
                    items
                        .iter()
                        .filter(|i| i.description.to_lowercase() != "hours")
                        .map(|i| i.amount)
                        .sum::<f64>()
                })
                .unwrap_or(0.0);
            let gross = base_gross + extra;
            // Determine tax law for employee's home region, defaulting to zero tax
            let law = tax_laws.get(&employee.home_region).or_else(|| tax_laws.get("US-FED"));
            let calculator = calculators.get(&employee.home_region).or_else(|| calculators.get("US-FED"));
            let taxes = if let (Some(l), Some(calc)) = (law, calculator) {
                calc.calculate(&employee, gross, l)
            } else {
                0.0
            };
            let net = gross - taxes;
            // Build details JSON; for demonstration we include just the tax rate if available
            let details = if let Some(l) = law {
                json!({"tax_region": l.region, "tax_version": l.version})
            } else {
                json!({})
            };
            EmployeePayResult {
                employee,
                gross,
                taxes,
                net,
                details,
            }
        })
        .collect();
    Ok(PayRunResult { period, results })
}