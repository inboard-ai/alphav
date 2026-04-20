//! DataFrame conversion tests for all Alpha Vantage endpoints
use alphav::AlphaVantage;
use alphav::tool_use::{ToolResult, call_tool};
use serde_json::json;
use std::env;

fn setup() -> Result<AlphaVantage, Box<dyn std::error::Error>> {
    let api_key = env::var("ALPHAVANTAGE_API_KEY").map_err(|_| "ALPHAVANTAGE_API_KEY environment variable not set")?;
    Ok(AlphaVantage::default().with_key(api_key))
}

async fn check_endpoint(
    client: &AlphaVantage,
    tool_name: &str,
    params: serde_json::Value,
    min_rows: usize,
    min_columns: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = json!({ "tool": tool_name, "params": params });

    let result = call_tool(client, request).await?;

    let df = match result {
        ToolResult::DataFrame(df) => df,
        ToolResult::Text(t) => return Err(format!("Expected DataFrame for {tool_name}, got Text: {}", t.content).into()),
    };

    assert!(!df.schema.is_empty(), "Schema should not be empty for {tool_name}");
    assert!(
        df.schema.len() >= min_columns,
        "Schema should have at least {min_columns} columns for {tool_name}, got {}",
        df.schema.len()
    );

    let polars = df
        .to_dataframe()
        .map_err(|e| format!("Failed to convert {tool_name} to DataFrame: {e}"))?;

    assert!(
        polars.height() >= min_rows,
        "DataFrame should have at least {min_rows} rows for {tool_name}, got {}",
        polars.height()
    );
    assert!(
        polars.width() >= min_columns,
        "DataFrame should have at least {min_columns} columns for {tool_name}, got {}",
        polars.width()
    );

    println!(
        "✅ {tool_name}: {} rows × {} columns",
        polars.height(),
        polars.width()
    );
    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_time_series_intraday_dataframe() {
    let client = setup().expect("Failed to initialize client");

    check_endpoint(
        &client,
        "time_series_intraday",
        json!({
            "symbol": "AAPL",
            "interval": "5min",
            "outputsize": "compact"
        }),
        50, // At least 50 rows for compact intraday
        6,  // timestamp, open, high, low, close, volume
    )
    .await
    .expect("time_series_intraday should return valid DataFrame");
}

#[tokio::test]
#[ignore]
async fn test_time_series_daily_dataframe() {
    let client = setup().expect("Failed to initialize client");

    check_endpoint(
        &client,
        "time_series_daily",
        json!({
            "symbol": "AAPL",
            "outputsize": "compact"
        }),
        90, // At least 90 rows for compact daily (~100 trading days)
        6,  // date, open, high, low, close, volume
    )
    .await
    .expect("time_series_daily should return valid DataFrame");
}

#[tokio::test]
#[ignore]
async fn test_time_series_weekly_dataframe() {
    let client = setup().expect("Failed to initialize client");

    check_endpoint(
        &client,
        "time_series_weekly",
        json!({ "symbol": "AAPL" }),
        100, // At least 100 weeks of data
        6,   // week_ending, open, high, low, close, volume
    )
    .await
    .expect("time_series_weekly should return valid DataFrame");
}

#[tokio::test]
#[ignore]
async fn test_time_series_monthly_dataframe() {
    let client = setup().expect("Failed to initialize client");

    check_endpoint(
        &client,
        "time_series_monthly",
        json!({ "symbol": "AAPL" }),
        50, // At least 50 months of data
        6,  // month, open, high, low, close, volume
    )
    .await
    .expect("time_series_monthly should return valid DataFrame");
}

#[tokio::test]
#[ignore]
async fn test_company_overview_dataframe() {
    let client = setup().expect("Failed to initialize client");

    check_endpoint(
        &client,
        "company_overview",
        json!({ "symbol": "AAPL" }),
        1,  // Single company record
        20, // At least 20 company overview fields
    )
    .await
    .expect("company_overview should return valid DataFrame");
}

#[tokio::test]
#[ignore]
async fn test_earnings_dataframe() {
    let client = setup().expect("Failed to initialize client");

    check_endpoint(
        &client,
        "earnings",
        json!({ "symbol": "AAPL" }),
        10, // At least 10 earnings records (annual + quarterly)
        5,  // period_type, fiscal_date_ending, reported_eps, etc.
    )
    .await
    .expect("earnings should return valid DataFrame");
}

#[tokio::test]
#[ignore]
async fn test_earnings_estimates_dataframe() {
    let client = setup().expect("Failed to initialize client");

    check_endpoint(
        &client,
        "earnings_estimates",
        json!({ "symbol": "AAPL" }),
        5, // At least 5 estimates
        8, // date, horizon, eps estimates, revenue estimates, etc.
    )
    .await
    .expect("earnings_estimates should return valid DataFrame");
}

#[tokio::test]
#[ignore]
async fn test_earnings_estimates_with_horizon_dataframe() {
    let client = setup().expect("Failed to initialize client");

    check_endpoint(
        &client,
        "earnings_estimates",
        json!({
            "symbol": "AAPL",
            "horizon": "3month"
        }),
        1, // At least 1 estimate with horizon filter
        8, // date, horizon, eps estimates, revenue estimates, etc.
    )
    .await
    .expect("earnings_estimates with horizon should return valid DataFrame");
}

#[tokio::test]
#[ignore]
async fn test_income_statement_dataframe() {
    let client = setup().expect("Failed to initialize client");

    check_endpoint(
        &client,
        "income_statement",
        json!({ "symbol": "AAPL" }),
        20, // At least 20 income statement records (annual + quarterly)
        10, // period_type, fiscal_date_ending, revenue, costs, etc.
    )
    .await
    .expect("income_statement should return valid DataFrame");
}

#[tokio::test]
#[ignore]
async fn test_balance_sheet_dataframe() {
    let client = setup().expect("Failed to initialize client");

    check_endpoint(
        &client,
        "balance_sheet",
        json!({ "symbol": "AAPL" }),
        20, // At least 20 balance sheet records (annual + quarterly)
        9,  // period_type, fiscal_date_ending, assets, liabilities, equity, etc.
    )
    .await
    .expect("balance_sheet should return valid DataFrame");
}

#[tokio::test]
#[ignore]
async fn test_cash_flow_dataframe() {
    let client = setup().expect("Failed to initialize client");

    check_endpoint(
        &client,
        "cash_flow",
        json!({ "symbol": "AAPL" }),
        20, // At least 20 cash flow records (annual + quarterly)
        12, // period_type, fiscal_date_ending, operating/investing/financing flows, etc.
    )
    .await
    .expect("cash_flow should return valid DataFrame");
}

#[tokio::test]
#[ignore]
async fn test_all_endpoints_return_valid_dataframes() {
    let client = setup().expect("Failed to initialize client");

    let test_cases = vec![
        (
            "time_series_intraday",
            json!({"symbol": "AAPL", "interval": "5min", "outputsize": "compact"}),
            50,
            6,
        ),
        (
            "time_series_daily",
            json!({"symbol": "AAPL", "outputsize": "compact"}),
            90,
            6,
        ),
        ("time_series_weekly", json!({"symbol": "AAPL"}), 100, 6),
        ("time_series_monthly", json!({"symbol": "AAPL"}), 50, 6),
        ("company_overview", json!({"symbol": "AAPL"}), 1, 20),
        ("earnings", json!({"symbol": "AAPL"}), 10, 5),
        ("earnings_estimates", json!({"symbol": "AAPL"}), 5, 8),
        ("income_statement", json!({"symbol": "AAPL"}), 20, 10),
        ("balance_sheet", json!({"symbol": "AAPL"}), 20, 9),
        ("cash_flow", json!({"symbol": "AAPL"}), 20, 12),
    ];

    for (tool_name, params, min_rows, min_columns) in test_cases {
        check_endpoint(&client, tool_name, params, min_rows, min_columns)
            .await
            .unwrap_or_else(|e| panic!("{tool_name} should return valid DataFrame: {e}"));
    }

    println!("🎉 All 10 endpoints returned valid DataFrames!");
}
