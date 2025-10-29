//! Integration tests for the Alpha Vantage API client
//!
//! These tests make real API calls and should be run sparingly to avoid
//! exhausting API quota. Run with:
//!
//! ```sh
//! cargo test --test integration_tests -- --ignored --test-threads=1
//! ```
//!
//! Ensure ALPHAVANTAGE_API_KEY is set in your environment or .env file.

use alphav::request::common::{Interval, OutputSize};
use alphav::rest;
use alphav::{AlphaVantage, Result};

/// Helper to initialize the client from environment
fn setup() -> Result<AlphaVantage> {
    dotenvy::dotenv().ok();
    std::env::var("ALPHAVANTAGE_API_KEY")
        .map(|key| AlphaVantage::default().with_key(key))
        .map_err(|_| {
            alphav::Error::Custom(
                "ALPHAVANTAGE_API_KEY not found. Set it in .env or environment.".to_string(),
            )
        })
}

#[tokio::test]
#[ignore]
async fn test_time_series_intraday() {
    let client = setup().expect("Failed to initialize client");

    let result = rest::time_series::intraday(&client, "IBM", Interval::FiveMin)
        .outputsize(OutputSize::Compact)
        .get()
        .await;

    assert!(
        result.is_ok(),
        "Failed to fetch intraday data: {:?}",
        result.err()
    );

    let json = result.unwrap();
    assert!(!json.is_empty(), "Response should not be empty");

    // Verify response contains expected structure
    assert!(
        json.contains("Meta Data") || json.contains("Time Series"),
        "Response should contain time series data"
    );
}

#[tokio::test]
#[ignore]
async fn test_time_series_daily() {
    let client = setup().expect("Failed to initialize client");

    let result = rest::time_series::daily(&client, "IBM")
        .outputsize(OutputSize::Compact)
        .get()
        .await;

    assert!(
        result.is_ok(),
        "Failed to fetch daily data: {:?}",
        result.err()
    );

    let json = result.unwrap();
    assert!(!json.is_empty(), "Response should not be empty");
}

#[tokio::test]
#[ignore]
async fn test_earnings_estimates() {
    let client = setup().expect("Failed to initialize client");

    let result = rest::fundamentals::earnings_estimates(&client, "IBM")
        .horizon("3month")
        .get()
        .await;

    assert!(
        result.is_ok(),
        "Failed to fetch earnings estimates: {:?}",
        result.err()
    );

    let json = result.unwrap();
    assert!(!json.is_empty(), "Response should not be empty");

    // Verify response contains expected structure
    assert!(
        json.contains("symbol") || json.contains("estimates") || json.contains("Symbol"),
        "Response should contain earnings estimate data, contains:\n{}",
        json
    );
}

#[tokio::test]
#[ignore]
async fn test_company_overview() {
    let client = setup().expect("Failed to initialize client");

    let result = rest::fundamentals::company_overview(&client, "IBM")
        .get()
        .await;

    assert!(
        result.is_ok(),
        "Failed to fetch company overview: {:?}",
        result.err()
    );

    let json = result.unwrap();
    assert!(!json.is_empty(), "Response should not be empty");
    assert!(
        json.contains("Symbol") || json.contains("symbol"),
        "Response should contain company overview data"
    );
}

#[tokio::test]
#[ignore]
async fn test_earnings() {
    let client = setup().expect("Failed to initialize client");

    let result = rest::fundamentals::earnings(&client, "IBM").get().await;

    assert!(
        result.is_ok(),
        "Failed to fetch earnings: {:?}",
        result.err()
    );

    let json = result.unwrap();
    assert!(!json.is_empty(), "Response should not be empty");
}

#[tokio::test]
#[ignore]
async fn test_missing_api_key() {
    // Test that missing API key produces appropriate error
    let client = AlphaVantage::default();

    let result = rest::time_series::daily(&client, "IBM").get().await;

    assert!(result.is_err(), "Request without API key should fail");

    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(
            error_msg.contains("API key"),
            "Error should mention API key: {}",
            error_msg
        );
    }
}

#[tokio::test]
async fn test_client_initialization() {
    // Test that we can create a client (doesn't make API calls)
    let client = AlphaVantage::default().with_key("test_key");
    assert_eq!(client.api_key(), Some("test_key"));
}

#[tokio::test]
async fn test_builder_pattern() {
    // Test that builder methods work correctly (doesn't make API calls)
    let client = AlphaVantage::default().with_key("test_key");

    // Test that we can chain builder methods
    let _request = rest::time_series::daily(&client, "TEST")
        .outputsize(OutputSize::Compact)
        .datatype("json");

    // If this compiles and runs, the builder pattern works
    assert!(true);
}
