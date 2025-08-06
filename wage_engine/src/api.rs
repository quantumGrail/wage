//! HTTP API for the Wage Engine.
//!
//! This module exposes a minimal REST API around the payroll engine
//! using the [`axum`](https://crates.io/crates/axum) framework.  The
//! API allows clients to submit a payroll run definition and receive
//! the results in JSON.  The server relies on the same tax law and
//! calculator definitions used by the core engine.

use crate::engine::run_payroll;
use crate::models::{PayRunInput, PayRunResult};
use crate::tax::{load_tax_laws_from_dir, FlatStateCalculator, TaxLaw, TaxCalculator, UsFederalCalculator};
use anyhow::Result;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Application state shared across requests.
pub struct AppState {
    pub tax_laws: RwLock<HashMap<String, TaxLaw>>,
    pub calculators: RwLock<HashMap<String, Arc<dyn TaxCalculator>>>,
}

/// Build the API router and initialise tax laws/calculators from the
/// given directory.  Returns the router and a handle to the state.
pub async fn build_router(tax_law_dir: PathBuf) -> Result<(Router, Arc<AppState>)> {
    // Load tax laws from disk
    let laws = load_tax_laws_from_dir(&tax_law_dir)?;
    let mut law_map = HashMap::new();
    for law in laws.into_iter() {
        law_map.insert(format!("{}-{}", law.region, law.version), law);
    }
    // Build calculators; register at least a federal calculator as a fallback
    let mut calculators: HashMap<String, Arc<dyn TaxCalculator>> = HashMap::new();
    calculators.insert("US-FED".to_string(), Arc::new(UsFederalCalculator));
    // Example: register state calculators for each region found in the tax laws
    let regions: Vec<String> = law_map
        .values()
        .map(|law| law.region.clone())
        .collect();
    for region in regions {
        // Avoid registering the federal calculator twice
        if region == "US-FED" {
            continue;
        }
        calculators.insert(region.clone(), Arc::new(FlatStateCalculator { region }));
    }
    let state = Arc::new(AppState {
        tax_laws: RwLock::new(law_map),
        calculators: RwLock::new(calculators),
    });
    // Construct router
    let router = Router::new()
        .route("/api/calculate", post(calculate_handler))
        .with_state(state.clone());
    Ok((router, state))
}

/// Handler for POST /api/calculate
async fn calculate_handler(
    State(app_state): State<Arc<AppState>>,
    Json(input): Json<PayRunInput>,
) -> impl IntoResponse {
    // Clone tax laws and calculators under read lock for this request
    let tax_laws = app_state.tax_laws.read().await;
    let calculators = app_state.calculators.read().await;
    match run_payroll(input, &*tax_laws, &*calculators) {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => {
            let body = Json(serde_json::json!({"error": err.to_string()}));
            (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
        }
    }
}

/// Launch the API server.  This function builds the router from the
/// given tax law directory and binds to the supplied address.  It
/// blocks until the server terminates (e.g. when interrupted).
pub async fn serve(addr: &str, tax_law_dir: PathBuf) -> Result<()> {
    let (router, _state) = build_router(tax_law_dir).await?;
    println!("Server listening on {}", addr);
    axum::Server::bind(&addr.parse().unwrap())
        .serve(router.into_make_service())
        .await
        .map_err(|e| e.into())
}