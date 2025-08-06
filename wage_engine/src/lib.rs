//! Wage Engine library crate.
//!
//! This crate exposes the core payroll computation engine and API
//! components as reusable modules.  External applications may
//! depend on the `wage_engine` crate and call into `engine::run_payroll`
//! directly or embed the API via `api::build_router`.

pub mod models;
pub mod tax;
pub mod engine;
pub mod api;