//! DataFrame conversion tests for all Alpha Vantage endpoints
use alphav::AlphaVantage;
use alphav::tool_use::{ToolCallResult, call_tool};
use emporium_core::ToolResult;
use serde_json::json;
use std::env;

fn setup() -> Result<AlphaVantage, Box<dyn std::error::Error>> {
    let api_key = env::var("ALPHAVANTAGE_API_KEY").map_err(|_| "ALPHAVANTAGE_API_KEY environment variable not set")?;
    Ok(AlphaVantage::default().with_key(api_key))
}

async fn test_endpoint_dataframe(
    client: &AlphaVantage,
    tool_name: &str,
    params: serde_json::Value,
    min_rows: usize,
    min_columns: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let request = json!({
        "tool": tool_name,
        "params": params
    });

    let result = call_tool(client, request).await?;

    match result {
        ToolCallResult::DataFrame { data, schema, metadata } => {
            // Verify schema is not empty
            assert!(!schema.is_empty(), "Schema should not be empty for {}", tool_name);
            assert!(
                schema.len() >= min_columns,
                "Schema should have at least {} columns for {}, got {}",
                min_columns,
                tool_name,
                schema.len()
            );

            // Convert to emporium DataFrame and verify
            let emp = emporium_core::ToolResult::columnar(data.clone(), schema.clone(), metadata.clone());
            match emp {
                ToolResult::DataFrame(proto) => {
                    let df = proto
                        .to_dataframe()
                        .map_err(|e| format!("Failed to convert {} to DataFrame: {}", tool_name, e))?;

                    assert!(
                        df.height() >= min_rows,
                        "DataFrame should have at least {} rows for {}, got {}",
                        min_rows,
                        tool_name,
                        df.height()
                    );
                    assert!(
                        df.width() >= min_columns,
                        "DataFrame should have at least {} columns for {}, got {}",
                        min_columns,
                        tool_name,
                        df.width()
                    );

                    println!("✅ {}: {} rows × {} columns", tool_name, df.height(), df.width());
                }
                ToolResult::Text(_) => {
                    return Err(format!("Expected DataFrame for {}, got Text", tool_name).into());
                }
            }
        }
        ToolCallResult::Text(_) => {
            return Err(format!("Expected DataFrame for {}, got Text", tool_name).into());
        }
    }

    Ok(())
}

#[tokio::test]
#[ignore]
async fn test_time_series_intraday_dataframe() {
    let client = setup().expect("Failed to initialize client");

    test_endpoint_dataframe(
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

    test_endpoint_dataframe(
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

    test_endpoint_dataframe(
        &client,
        "time_series_weekly",
        json!({
            "symbol": "AAPL"
        }),
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

    test_endpoint_dataframe(
        &client,
        "time_series_monthly",
        json!({
            "symbol": "AAPL"
        }),
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

    test_endpoint_dataframe(
        &client,
        "company_overview",
        json!({
            "symbol": "AAPL"
        }),
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

    test_endpoint_dataframe(
        &client,
        "earnings",
        json!({
            "symbol": "AAPL"
        }),
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

    test_endpoint_dataframe(
        &client,
        "earnings_estimates",
        json!({
            "symbol": "AAPL"
        }),
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

    test_endpoint_dataframe(
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

    test_endpoint_dataframe(
        &client,
        "income_statement",
        json!({
            "symbol": "AAPL"
        }),
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

    test_endpoint_dataframe(
        &client,
        "balance_sheet",
        json!({
            "symbol": "AAPL"
        }),
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

    test_endpoint_dataframe(
        &client,
        "cash_flow",
        json!({
            "symbol": "AAPL"
        }),
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
        test_endpoint_dataframe(&client, tool_name, params, min_rows, min_columns)
            .await
            .expect(&format!("{} should return valid DataFrame", tool_name));
    }

    println!("🎉 All {} endpoints returned valid DataFrames!", 10);
}
