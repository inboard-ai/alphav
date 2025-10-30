//! Rust client library for Alpha Vantage API
//!
//! # Quick Start
//!
//! ```no_run
//! use alphav::AlphaVantage;
//! use alphav::rest;
//! use alphav::request::common::Interval;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = AlphaVantage::default().with_key("your_api_key");
//!     let json = rest::time_series::intraday(&client, "AAPL", Interval::FiveMin).get().await?;
//!     println!("{}", json);
//!     Ok(())
//! }
//! ```
//!
//! # Endpoint API
//!
//! Each endpoint returns a specific request builder type. Call `.get()` to execute:
//!
//! ```no_run
//! use alphav::AlphaVantage;
//! use alphav::rest::time_series;
//! use alphav::request::common::{Interval, OutputSize};
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let client = AlphaVantage::default().with_key("your_api_key");
//!
//! // Raw JSON response
//! let json = time_series::intraday(&client, "AAPL", Interval::FiveMin).get().await?;
//!
//! // With options
//! let json = time_series::daily(&client, "AAPL")
//!     .outputsize(OutputSize::Full)
//!     .get()
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Features
//!
//! - **`hyper`** (default) - Uses [`hyper`](https://docs.rs/hyper) as the HTTP client (lightweight and fast).
//!
//! - **`reqwest`** - Alternative HTTP client using [`reqwest`](https://docs.rs/reqwest) (more features).
//!   To use reqwest instead: `default-features = false, features = ["reqwest", "decoder"]`.
//!
//! - **`decoder`** (default) - Enables typed response decoding using the [`decoder`](https://docs.rs/decoder) crate.
//!
//! - **`dotenvy`** - Enables loading API keys from environment variables via [`dotenvy`](https://docs.rs/dotenvy).
//!   Adds `AlphaVantage::new()` which loads `ALPHAVANTAGE_API_KEY` from `.env` or environment.
//!   Without this feature, use `AlphaVantage::default().with_key("your_key")` instead.
//!
//! - **`table`** - Enables Polars DataFrame output via [`polars`](https://docs.rs/polars).

#![warn(missing_docs)]

mod client;
pub mod error;
pub mod request;
pub mod response;
pub mod rest;

pub mod execute;
pub mod processor;
pub mod tool_use;

pub use error::{Error, Result};
pub use request::Request;
pub use response::Response;

/// The main Alpha Vantage API client with the default HTTP client.
///
/// - When `hyper` feature is enabled (default): uses `HyperClient`
/// - When `reqwest` feature is enabled: uses `reqwest::Client`
/// - Otherwise: use `client::AlphaVantage<YourClient>` directly
#[cfg(feature = "reqwest")]
pub type AlphaVantage = client::AlphaVantage<reqwest::Client>;

/// The main Alpha Vantage API client with the default HTTP client.
///
/// - When `hyper` feature is enabled (default): uses `HyperClient`
/// - When `reqwest` feature is enabled: uses `reqwest::Client`
/// - Otherwise: use `client::AlphaVantage<YourClient>` directly
#[cfg(all(feature = "hyper", not(feature = "reqwest")))]
pub type AlphaVantage = client::AlphaVantage<request::HyperClient>;

// When neither reqwest nor hyper is enabled, re-export the generic AlphaVantage
#[cfg(not(any(feature = "reqwest", feature = "hyper")))]
pub use client::AlphaVantage;

#[cfg(any(feature = "reqwest", feature = "hyper"))]
static STATIC_INSTANCE: std::sync::LazyLock<arc_swap::ArcSwap<AlphaVantage>> =
    std::sync::LazyLock::new(|| arc_swap::ArcSwap::from_pointee(AlphaVantage::default()));

/// Initialize a static Alpha Vantage instance.
#[cfg(any(feature = "reqwest", feature = "hyper"))]
pub fn initialize(client: AlphaVantage) -> std::sync::Arc<AlphaVantage> {
    STATIC_INSTANCE.swap(std::sync::Arc::from(client))
}

/// Get the static Alpha Vantage instance.
#[cfg(any(feature = "reqwest", feature = "hyper"))]
pub fn instance() -> std::sync::Arc<AlphaVantage> {
    STATIC_INSTANCE.load().clone()
}
