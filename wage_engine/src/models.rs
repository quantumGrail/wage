//! Data models for the Wage Engine.
//!
//! The `models` module defines a set of serialisable structs and
//! enums representing employees, pay items and pay periods.  These
//! data types derive `Serialize` and `Deserialize` so that they can
//! be easily persisted or transmitted over a network.  They form the
//! basis of the engine’s input and output structures.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents an employee in the payroll system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Employee {
    /// A globally unique identifier for the employee.  This could be a
    /// UUID or any unique string used by your organisation.
    pub id: String,
    /// The employee’s full name.
    pub name: String,
    /// A region code identifying the tax jurisdiction for this
    /// employee.  Examples: `"US-OK"` for Oklahoma, `"US-CA"` for
    /// California, or `"UK"` for the United Kingdom.
    pub home_region: String,
    /// The base pay rate.  For hourly employees this is the hourly
    /// rate; for salaried employees this is the amount per pay
    /// period.
    pub pay_rate: f64,
    /// How the employee is paid (hourly or salaried).  See
    /// [`PayFrequency`] for details.
    pub pay_frequency: PayFrequency,
}

/// Indicates whether an employee is paid hourly or on a salaried basis.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PayFrequency {
    /// The employee is paid an hourly rate.  Hours worked must be
    /// supplied via `PayItem`s.
    Hourly,
    /// The employee receives a fixed salary per pay period.
    Salary,
}

/// Additional earnings or deductions applied to an employee’s pay.
///
/// A `PayItem` might represent overtime, bonuses, reimbursements,
/// benefits, or deductions such as healthcare premiums or 401(k)
/// contributions.  Positive amounts add to gross earnings, while
/// negative amounts subtract.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayItem {
    /// Human‑readable description of the item.
    pub description: String,
    /// Monetary value of this item.  Positive values represent
    /// earnings; negative values represent deductions.
    pub amount: f64,
}

/// Defines the start and end dates of a pay period.  Dates are
/// represented as ISO 8601 strings (`YYYY-MM-DD`) for simplicity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayPeriod {
    /// Inclusive start date of the pay period.
    pub start: String,
    /// Inclusive end date of the pay period.
    pub end: String,
}

/// Input to the payroll engine.
///
/// A `PayRunInput` contains a list of employees, a mapping from
/// employee IDs to their associated pay items, and the pay period to
/// be processed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayRunInput {
    /// The employees to be paid in this run.
    pub employees: Vec<Employee>,
    /// A mapping of employee ID to the list of pay items affecting
    /// that employee.  If an employee has no pay items, they may be
    /// omitted or given an empty vector.
    pub pay_items: HashMap<String, Vec<PayItem>>,
    /// The period over which payment is being calculated.
    pub pay_period: PayPeriod,
}

/// The result of a payroll calculation for a single employee.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeePayResult {
    /// The employee for whom this result applies.
    pub employee: Employee,
    /// Total gross earnings for the period (before taxes).
    pub gross: f64,
    /// Total tax withheld for the period.
    pub taxes: f64,
    /// Net pay after taxes (gross minus taxes).
    pub net: f64,
    /// Additional details.  Implementations may use this field to
    /// include per‑tax breakdowns, employer contributions, etc.  The
    /// structure is intentionally flexible; if you require more
    /// structure consider replacing it with a strongly‑typed type of
    /// your own.
    pub details: serde_json::Value,
}

/// The aggregate result of a payroll run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayRunResult {
    /// The pay period that was processed.
    pub period: PayPeriod,
    /// Individual results for each employee.
    pub results: Vec<EmployeePayResult>,
}