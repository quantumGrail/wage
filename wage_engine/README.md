# Wage Engine

**Wage Engine** (Worldwide Automated Gross Earnings) is a modular payroll
calculation engine written in Rust.  It draws inspiration from a set
of design notes (included below for reference) and aims to provide a
foundation for scalable, multi‑core payroll computation with
pluggable tax logic and a simple HTTP interface.  Each state or
territory can define its own tax rules via a common trait and
external JSON files, allowing the engine to handle domestic and
international payroll scenarios.

## Design goals

The design notes from the specification suggest the following
high‑level objectives:

1. **Low‑level multi‑core engine** – Calculate payroll for many
   employees concurrently using multiple CPU cores.  In this
   implementation the [`rayon`](https://crates.io/crates/rayon)
   library is used to parallelise operations across workers.
2. **Pluggable tax modules** – Each state or territory is expressed
   as a trait object implementing the `TaxCalculator` trait.  Tax
   rules are stored in versioned JSON files under the
   `tax_laws/` directory and deserialised at runtime.
3. **Serializable models** – Employees, pay items and pay runs are
   represented as structs deriving [`serde::Serialize`]
   and [`serde::Deserialize`].  This makes it straightforward to
   persist and transmit data via API calls.
4. **External API** – The optional HTTP API (built using
   [`axum`](https://crates.io/crates/axum) and [`tokio`](https://crates.io/crates/tokio))
   exposes endpoints for previewing and calculating payroll.  Other
   applications can integrate with the engine via these endpoints
   without being tied to the internal implementation.
5. **Versioned tax law** – Tax law changes over time.  By storing
   versioned JSON files (e.g. `us_federal_2025.json`) and exposing
   a minimal interface to load them, the engine can answer questions
   such as “how will taxes change next year?” or “what were taxes last
   year?”

Although this repository is structured as a complete Rust crate,
building it requires the Rust toolchain, which is not available in
the execution environment used to create this answer.  To build and
run the engine locally, install Rust from <https://rustup.rs> and run
the following commands:

```sh
# build the library and binary
cd wage_engine
cargo build --release

# run the API server (default binds to localhost:3000)
cargo run --bin wage_engine

# in another terminal, calculate payroll for a set of employees
curl -X POST http://localhost:3000/api/calculate \
  -H 'Content-Type: application/json' \
  -d '{"employees":[{"id":"1","name":"Alice","home_region":"US-OK","pay_rate":50.0,"pay_frequency":"Hourly"}],"pay_items":{},"pay_period":{"start":"2025-08-01","end":"2025-08-15"}}'

```

The API will respond with a JSON payload describing each employee’s
gross, tax and net amounts for the specified pay period.

## Specification notes (from handwritten document)

The design of this engine is guided by a set of handwritten notes
(provided by the user).  They include the following key points:

- **WAGE:** Worldwide Automated Gross Earnings
- **Core architecture:** a low‑level multi‑core engine with inputs for global
  scaling.  Each state/territory is a passable trait.  This is
  reflected in the `engine` module and `tax` traits.
- **Models:** serializable data types for employees and pay items,
  implemented in the `models` module.
- **External API:** an API to facilitate preview and calculation within
  other applications.  The `api` module demonstrates how to expose
  the calculation over HTTP.
- **Tax laws stored as versioned JSON:** stored under `tax_laws/` and
  loaded at runtime.  The `TaxLaw` struct holds the parsed data.
- **Abstract higher:** the notes mention turning payroll into a domain
  of a generic data processor.  This crate does not implement a
  general data processor, but the modular design allows the core
  engine to be embedded within a larger system.

Feel free to extend or modify this project to suit your needs.