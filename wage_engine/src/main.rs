//! Entry point for the Wage Engine binary.
//!
//! Running this binary will start an HTTP server that exposes a
//! minimal API for calculating payroll.  The directory containing
//! tax law JSON files may be specified via the `WAGE_TAX_LAW_DIR`
//! environment variable; if unset the server looks for a `tax_laws`
//! folder relative to the current working directory.

use std::path::PathBuf;

#[tokio::main]
async fn main() {
    // Determine where tax law files are located
    let tax_dir = std::env::var("WAGE_TAX_LAW_DIR").unwrap_or_else(|_| "tax_laws".to_string());
    let tax_dir_path = PathBuf::from(tax_dir);
    // Determine bind address
    let addr = std::env::var("WAGE_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    if let Err(err) = wage_engine::api::serve(&addr, tax_dir_path).await {
        eprintln!("Error running server: {}", err);
    }
}

// Public re-exports so the binary has access to library modules
pub use wage_engine::{api, engine, models, tax};