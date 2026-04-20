//! Tests for the tool_use module
//!
//! These tests verify that each tool can be called successfully and returns a
//! `ToolResult::DataFrame` whose schema and data match the transformed shape
//! produced by the module. They require a valid `ALPHAVANTAGE_API_KEY` and are
//! marked `#[ignore]` to avoid burning API quota.

use alphav::AlphaVantage;
use alphav::tool_use::{ToolResult, call_tool, get_tool_details, list_tools};
use emporium_core::tool::DataFrame;
use serde_json::{Value, json};

#[cfg(feature = "dotenvy")]
fn setup_client() -> AlphaVantage {
    AlphaVantage::new().expect("Failed to create client. Make sure ALPHAVANTAGE_API_KEY is set")
}

#[cfg(not(feature = "dotenvy"))]
fn setup_client() -> AlphaVantage {
    let api_key = std::env::var("ALPHAVANTAGE_API_KEY").expect("ALPHAVANTAGE_API_KEY must be set");
    AlphaVantage::default().with_key(api_key)
}

/// Pattern-unwrap a successful `ToolResult` into the inner `DataFrame` (panicking
/// with the `Text` body if the tool returned text instead).
fn expect_df(result: ToolResult) -> DataFrame {
    match result {
        ToolResult::DataFrame(df) => df,
        ToolResult::Text(t) => panic!("expected DataFrame, got Text: {}", t.content),
        other => panic!("expected DataFrame, got unknown ToolResult variant: {other:?}"),
    }
}

fn expect_rows(df: &DataFrame) -> &[Value] {
    df.data
        .as_array()
        .expect("DataFrame.data should be a JSON array of rows")
}

#[tokio::test]
#[ignore]
async fn test_list_tools() {
    let tools = list_tools();
    assert_eq!(tools.len(), 10, "Expected 10 tools");

    let tool_ids: Vec<&str> = tools.iter().map(|t| t.id.as_str()).collect();
    for expected in [
        "time_series_intraday",
        "time_series_daily",
        "time_series_weekly",
        "time_series_monthly",
        "company_overview",
        "earnings",
        "earnings_estimates",
        "income_statement",
        "balance_sheet",
        "cash_flow",
    ] {
        assert!(tool_ids.contains(&expected), "missing tool: {expected}");
    }
}

#[tokio::test]
#[ignore]
async fn test_get_tool_details() {
    let tool = get_tool_details("time_series_daily").expect("time_series_daily should exist");
    assert_eq!(tool.id, "time_series_daily");

    assert!(get_tool_details("invalid_tool").is_none());
}

#[tokio::test]
#[ignore]
async fn test_time_series_intraday() {
    let client = setup_client();
    let request = json!({
        "tool": "time_series_intraday",
        "params": {
            "symbol": "AAPL",
            "interval": "5min",
            "outputsize": "compact"
        }
    });

    let df = expect_df(call_tool(&client, request).await.expect("call_tool should succeed"));

    // Metadata preserves the raw Alpha Vantage "Meta Data" block.
    let metadata = df.metadata.as_ref().expect("intraday response should carry metadata");
    assert_eq!(metadata.get("2. Symbol").and_then(|v| v.as_str()), Some("AAPL"));
    assert_eq!(metadata.get("4. Interval").and_then(|v| v.as_str()), Some("5min"));

    // Schema should describe the six transformed columns.
    let col_names: Vec<&str> = df.schema.iter().map(|c| c.name.as_str()).collect();
    for expected in ["timestamp", "open", "high", "low", "close", "volume"] {
        assert!(col_names.contains(&expected), "schema missing column {expected}");
    }

    let rows = expect_rows(&df);
    assert!(!rows.is_empty(), "intraday should return at least one row");

    let first = &rows[0];
    let ts = first.get("timestamp").and_then(|v| v.as_str()).expect("timestamp");
    assert!(ts.len() >= 19 && ts.contains(' '), "unexpected timestamp format: {ts}");
    let open = first.get("open").and_then(|v| v.as_str()).expect("open");
    open.parse::<f64>().expect("open should be numeric");
}

#[tokio::test]
#[ignore]
async fn test_time_series_daily() {
    let client = setup_client();
    let request = json!({
        "tool": "time_series_daily",
        "params": { "symbol": "MSFT", "outputsize": "compact" }
    });

    let df = expect_df(call_tool(&client, request).await.expect("call_tool should succeed"));

    let metadata = df.metadata.as_ref().expect("daily response should carry metadata");
    assert_eq!(metadata.get("2. Symbol").and_then(|v| v.as_str()), Some("MSFT"));

    let rows = expect_rows(&df);
    assert!(
        rows.len() > 50 && rows.len() <= 100,
        "compact daily should have ~100 rows, got {}",
        rows.len()
    );

    for row in rows.iter().take(5) {
        let date = row.get("date").and_then(|v| v.as_str()).expect("date");
        assert!(
            date.len() == 10 && date.chars().nth(4) == Some('-'),
            "date should be YYYY-MM-DD: {date}"
        );
        let open: f64 = row.get("open").and_then(|v| v.as_str()).unwrap().parse().unwrap();
        let high: f64 = row.get("high").and_then(|v| v.as_str()).unwrap().parse().unwrap();
        let low: f64 = row.get("low").and_then(|v| v.as_str()).unwrap().parse().unwrap();
        let close: f64 = row.get("close").and_then(|v| v.as_str()).unwrap().parse().unwrap();
        row.get("volume")
            .and_then(|v| v.as_str())
            .unwrap()
            .parse::<u64>()
            .expect("volume numeric");

        assert!(high >= low, "high should be >= low");
        assert!(open >= low && open <= high, "open within [low, high]");
        assert!(close >= low && close <= high, "close within [low, high]");
    }
}

#[tokio::test]
#[ignore]
async fn test_time_series_weekly() {
    let client = setup_client();
    let request = json!({
        "tool": "time_series_weekly",
        "params": { "symbol": "GOOGL" }
    });

    let df = expect_df(call_tool(&client, request).await.expect("call_tool should succeed"));
    assert!(df.metadata.is_some(), "weekly response should carry metadata");
    assert!(!expect_rows(&df).is_empty(), "weekly should return at least one row");
    assert!(
        df.schema
            .iter()
            .any(|c| c.name == "week_ending"),
        "weekly schema should include week_ending column"
    );
}

#[tokio::test]
#[ignore]
async fn test_time_series_monthly() {
    let client = setup_client();
    let request = json!({
        "tool": "time_series_monthly",
        "params": { "symbol": "AMZN" }
    });

    let df = expect_df(call_tool(&client, request).await.expect("call_tool should succeed"));
    assert!(df.metadata.is_some());
    assert!(!expect_rows(&df).is_empty());
    assert!(df.schema.iter().any(|c| c.name == "month"));
}

#[tokio::test]
#[ignore]
async fn test_company_overview() {
    let client = setup_client();
    let request = json!({
        "tool": "company_overview",
        "params": { "symbol": "AAPL" }
    });

    let df = expect_df(call_tool(&client, request).await.expect("call_tool should succeed"));
    let rows = expect_rows(&df);
    assert_eq!(rows.len(), 1, "overview should be a single-row table");

    let row = &rows[0];
    assert_eq!(row.get("Symbol").and_then(|v| v.as_str()), Some("AAPL"));
    assert!(
        row.get("Name")
            .and_then(|v| v.as_str())
            .map(|s| s.contains("Apple"))
            .unwrap_or(false),
        "Name should mention Apple"
    );
    assert_eq!(row.get("Exchange").and_then(|v| v.as_str()), Some("NASDAQ"));

    let market_cap = row
        .get("MarketCapitalization")
        .and_then(|v| v.as_str())
        .expect("MarketCapitalization");
    market_cap.parse::<u64>().expect("market cap numeric");

    assert!(row.get("PERatio").is_some());
    assert!(row.get("DividendYield").is_some());
    assert!(row.get("52WeekHigh").is_some());
    assert!(row.get("52WeekLow").is_some());
    assert!(
        row.get("Description")
            .and_then(|v| v.as_str())
            .map(|d| d.contains("Apple") || d.contains("iPhone"))
            .unwrap_or(false)
    );
}

#[tokio::test]
#[ignore]
async fn test_earnings() {
    let client = setup_client();
    let request = json!({
        "tool": "earnings",
        "params": { "symbol": "TSLA" }
    });

    let df = expect_df(call_tool(&client, request).await.expect("call_tool should succeed"));
    assert_eq!(
        df.metadata
            .as_ref()
            .and_then(|m| m.get("symbol"))
            .and_then(|v| v.as_str()),
        Some("TSLA")
    );
    let rows = expect_rows(&df);
    assert!(!rows.is_empty(), "earnings should return rows");
    assert!(rows[0].get("period_type").is_some());
    assert!(rows[0].get("fiscal_date_ending").is_some());
}

#[tokio::test]
#[ignore]
async fn test_earnings_estimates() {
    let client = setup_client();
    let request = json!({
        "tool": "earnings_estimates",
        "params": { "symbol": "META", "horizon": "3month" }
    });

    let df = expect_df(call_tool(&client, request).await.expect("call_tool should succeed"));
    assert!(df.schema.iter().any(|c| c.name == "date"));
    assert!(df.schema.iter().any(|c| c.name == "horizon"));
    assert!(df.schema.iter().any(|c| c.name == "eps_estimate_average"));
}

#[tokio::test]
#[ignore]
async fn test_income_statement() {
    let client = setup_client();
    let request = json!({
        "tool": "income_statement",
        "params": { "symbol": "NVDA" }
    });

    let df = expect_df(call_tool(&client, request).await.expect("call_tool should succeed"));
    assert_eq!(
        df.metadata
            .as_ref()
            .and_then(|m| m.get("symbol"))
            .and_then(|v| v.as_str()),
        Some("NVDA")
    );

    let rows = expect_rows(&df);
    assert!(!rows.is_empty());
    let first = &rows[0];
    assert!(first.get("fiscal_date_ending").is_some());
    let revenue = first
        .get("total_revenue")
        .and_then(|v| v.as_str())
        .expect("total_revenue");
    revenue.parse::<i64>().expect("revenue numeric");
    assert!(first.get("gross_profit").is_some());
    assert!(first.get("operating_income").is_some());
    assert!(first.get("net_income").is_some());

    // Should include annual AND quarterly rows (at least ~4 quarterly expected).
    let quarterly_count = rows
        .iter()
        .filter(|r| r.get("period_type").and_then(|v| v.as_str()) == Some("quarterly"))
        .count();
    assert!(
        quarterly_count >= 4,
        "expected at least 4 quarterly rows, got {quarterly_count}"
    );
}

#[tokio::test]
#[ignore]
async fn test_balance_sheet() {
    let client = setup_client();
    let request = json!({
        "tool": "balance_sheet",
        "params": { "symbol": "JPM" }
    });

    let df = expect_df(call_tool(&client, request).await.expect("call_tool should succeed"));
    assert_eq!(
        df.metadata
            .as_ref()
            .and_then(|m| m.get("symbol"))
            .and_then(|v| v.as_str()),
        Some("JPM")
    );
    let rows = expect_rows(&df);
    assert!(!rows.is_empty());
    let first = &rows[0];
    assert!(first.get("fiscal_date_ending").is_some());
    assert!(first.get("total_assets").is_some());
    assert!(first.get("total_liabilities").is_some());
    assert!(first.get("total_shareholder_equity").is_some());
    assert!(first.get("cash_and_cash_equivalents").is_some());
    assert!(first.get("common_stock").is_some());
    assert!(first.get("retained_earnings").is_some());
}

#[tokio::test]
#[ignore]
async fn test_cash_flow() {
    let client = setup_client();
    let request = json!({
        "tool": "cash_flow",
        "params": { "symbol": "NFLX" }
    });

    let df = expect_df(call_tool(&client, request).await.expect("call_tool should succeed"));
    assert_eq!(
        df.metadata
            .as_ref()
            .and_then(|m| m.get("symbol"))
            .and_then(|v| v.as_str()),
        Some("NFLX")
    );
    let rows = expect_rows(&df);
    assert!(!rows.is_empty());
    let first = &rows[0];
    assert!(first.get("fiscal_date_ending").is_some());
    assert!(first.get("operating_cashflow").is_some());
    assert!(first.get("capital_expenditures").is_some());
    assert!(first.get("cashflow_from_investment").is_some());
    assert!(first.get("cashflow_from_financing").is_some());
}

#[tokio::test]
#[ignore]
async fn test_invalid_tool() {
    let client = setup_client();
    let request = json!({ "tool": "invalid_tool_name", "params": { "symbol": "AAPL" } });
    assert!(call_tool(&client, request).await.is_err());
}

#[tokio::test]
#[ignore]
async fn test_missing_required_params() {
    let client = setup_client();

    let request = json!({ "tool": "time_series_daily", "params": {} });
    assert!(
        call_tool(&client, request).await.is_err(),
        "missing symbol should error"
    );

    let request = json!({ "tool": "time_series_intraday", "params": { "symbol": "AAPL" } });
    assert!(
        call_tool(&client, request).await.is_err(),
        "missing interval should error"
    );
}

#[tokio::test]
#[ignore]
async fn test_invalid_interval() {
    let client = setup_client();
    let request = json!({
        "tool": "time_series_intraday",
        "params": { "symbol": "AAPL", "interval": "2min" }
    });
    assert!(call_tool(&client, request).await.is_err());
}

#[tokio::test]
#[ignore]
async fn test_optional_params() {
    let client = setup_client();

    let request = json!({ "tool": "earnings_estimates", "params": { "symbol": "AAPL" } });
    assert!(call_tool(&client, request).await.is_ok());

    let request = json!({ "tool": "time_series_daily", "params": { "symbol": "MSFT" } });
    assert!(call_tool(&client, request).await.is_ok());
}
