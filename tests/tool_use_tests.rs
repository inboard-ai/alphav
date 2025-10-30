//! Tests for the tool_use module
//! 
//! These tests verify that each tool can be called successfully and returns valid JSON.
//! They require a valid ALPHAVANTAGE_API_KEY to be set in the environment.

use alphav::AlphaVantage;
use alphav::tool_use::{list_tools, call_tool, get_tool_details, ToolCallResult};
use serde_json::json;

/// Helper to setup client
#[cfg(feature = "dotenvy")]
fn setup_client() -> AlphaVantage {
    AlphaVantage::new().expect("Failed to create client. Make sure ALPHAVANTAGE_API_KEY is set")
}

#[cfg(not(feature = "dotenvy"))]
fn setup_client() -> AlphaVantage {
    let api_key = std::env::var("ALPHAVANTAGE_API_KEY")
        .expect("ALPHAVANTAGE_API_KEY must be set");
    AlphaVantage::default().with_key(api_key)
}

#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored --test-threads=1
async fn test_list_tools() {
    let tools = list_tools();
    assert_eq!(tools.len(), 10, "Expected 10 tools");
    
    // Verify all expected tools are present
    let tool_ids: Vec<String> = tools.iter().map(|t| t.id.clone()).collect();
    assert!(tool_ids.contains(&"time_series_intraday".to_string()));
    assert!(tool_ids.contains(&"time_series_daily".to_string()));
    assert!(tool_ids.contains(&"time_series_weekly".to_string()));
    assert!(tool_ids.contains(&"time_series_monthly".to_string()));
    assert!(tool_ids.contains(&"company_overview".to_string()));
    assert!(tool_ids.contains(&"earnings".to_string()));
    assert!(tool_ids.contains(&"earnings_estimates".to_string()));
    assert!(tool_ids.contains(&"income_statement".to_string()));
    assert!(tool_ids.contains(&"balance_sheet".to_string()));
    assert!(tool_ids.contains(&"cash_flow".to_string()));
}

#[tokio::test]
#[ignore]
async fn test_get_tool_details() {
    // Test valid tool
    let tool = get_tool_details("time_series_daily");
    assert!(tool.is_some());
    assert_eq!(tool.unwrap().id, "time_series_daily");
    
    // Test invalid tool
    let tool = get_tool_details("invalid_tool");
    assert!(tool.is_none());
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
    
    let result = call_tool(&client, request).await;
    assert!(result.is_ok(), "Failed to call time_series_intraday: {:?}", result.err());
    
    let ToolCallResult::DataFrame { data: response, .. } = result.unwrap() else {
        panic!("Expected DataFrame result");
    };
    
    // Verify metadata structure
    let metadata = response.get("Meta Data")
        .expect("Response should have Meta Data");
    assert_eq!(metadata.get("1. Information").and_then(|v| v.as_str()),
        Some("Intraday (5min) open, high, low, close prices and volume"));
    assert_eq!(metadata.get("2. Symbol").and_then(|v| v.as_str()), Some("AAPL"));
    assert!(metadata.get("3. Last Refreshed").is_some());
    assert_eq!(metadata.get("4. Interval").and_then(|v| v.as_str()), Some("5min"));
    assert_eq!(metadata.get("5. Output Size").and_then(|v| v.as_str()), Some("Compact"));
    
    // Verify time series data
    let time_series = response.get("Time Series (5min)")
        .expect("Response should have Time Series (5min)")
        .as_object()
        .expect("Time series should be an object");
    
    assert!(!time_series.is_empty(), "Time series should contain data");
    
    // Check structure of first data point
    let (timestamp, data) = time_series.iter().next().unwrap();
    // Verify timestamp format (should be like "2025-10-29 16:00:00")
    assert!(timestamp.contains(' ') && timestamp.len() >= 19, 
        "Timestamp should be datetime format: {}", timestamp);
    
    assert!(data.get("1. open").and_then(|v| v.as_str()).is_some());
    assert!(data.get("2. high").and_then(|v| v.as_str()).is_some());
    assert!(data.get("3. low").and_then(|v| v.as_str()).is_some());
    assert!(data.get("4. close").and_then(|v| v.as_str()).is_some());
    assert!(data.get("5. volume").and_then(|v| v.as_str()).is_some());
    
    // Verify values are numeric strings
    let open = data.get("1. open").and_then(|v| v.as_str()).unwrap();
    open.parse::<f64>().expect("Open should be a valid number");
}

#[tokio::test]
#[ignore]
async fn test_time_series_daily() {
    let client = setup_client();
    let request = json!({
        "tool": "time_series_daily",
        "params": {
            "symbol": "MSFT",
            "outputsize": "compact"
        }
    });
    
    let result = call_tool(&client, request).await;
    assert!(result.is_ok(), "Failed to call time_series_daily: {:?}", result.err());
    
    let ToolCallResult::DataFrame { data: response, .. } = result.unwrap() else {
        panic!("Expected DataFrame result");
    };
    
    // Verify metadata
    let metadata = response.get("Meta Data")
        .expect("Response should have Meta Data");
    assert_eq!(metadata.get("2. Symbol").and_then(|v| v.as_str()), Some("MSFT"));
    assert_eq!(metadata.get("4. Output Size").and_then(|v| v.as_str()), Some("Compact"));
    
    // Verify time series data
    let time_series = response.get("Time Series (Daily)")
        .expect("Response should have Time Series (Daily)")
        .as_object()
        .expect("Time series should be an object");
    
    // Compact should have ~100 data points
    assert!(time_series.len() > 50 && time_series.len() <= 100, 
        "Compact output should have ~100 data points, got {}", time_series.len());
    
    // Verify date format and data structure
    for (date, data) in time_series.iter().take(5) {
        // Date should be YYYY-MM-DD format
        assert!(date.len() == 10 && date.chars().nth(4) == Some('-'), 
            "Date should be in YYYY-MM-DD format: {}", date);
        
        // All required fields present and numeric
        let open = data.get("1. open").and_then(|v| v.as_str())
            .expect("Should have open price");
        let high = data.get("2. high").and_then(|v| v.as_str())
            .expect("Should have high price");
        let low = data.get("3. low").and_then(|v| v.as_str())
            .expect("Should have low price");
        let close = data.get("4. close").and_then(|v| v.as_str())
            .expect("Should have close price");
        let volume = data.get("5. volume").and_then(|v| v.as_str())
            .expect("Should have volume");
        
        // Parse to verify they're valid numbers
        let open_val = open.parse::<f64>().expect("Open should be numeric");
        let high_val = high.parse::<f64>().expect("High should be numeric");
        let low_val = low.parse::<f64>().expect("Low should be numeric");
        let close_val = close.parse::<f64>().expect("Close should be numeric");
        volume.parse::<u64>().expect("Volume should be numeric");
        
        // Sanity checks
        assert!(high_val >= low_val, "High should be >= low");
        assert!(open_val >= low_val && open_val <= high_val, "Open should be between low and high");
        assert!(close_val >= low_val && close_val <= high_val, "Close should be between low and high");
    }
}

#[tokio::test]
#[ignore]
async fn test_time_series_weekly() {
    let client = setup_client();
    let request = json!({
        "tool": "time_series_weekly",
        "params": {
            "symbol": "GOOGL"
        }
    });
    
    let result = call_tool(&client, request).await;
    assert!(result.is_ok(), "Failed to call time_series_weekly: {:?}", result.err());
    
    let ToolCallResult::DataFrame { data: response, .. } = result.unwrap() else {
        panic!("Expected DataFrame result");
    };
    assert!(response.get("Meta Data").is_some());
    assert!(response.get("Weekly Time Series").is_some());
}

#[tokio::test]
#[ignore]
async fn test_time_series_monthly() {
    let client = setup_client();
    let request = json!({
        "tool": "time_series_monthly",
        "params": {
            "symbol": "AMZN"
        }
    });
    
    let result = call_tool(&client, request).await;
    assert!(result.is_ok(), "Failed to call time_series_monthly: {:?}", result.err());
    
    let ToolCallResult::DataFrame { data: response, .. } = result.unwrap() else {
        panic!("Expected DataFrame result");
    };
    assert!(response.get("Meta Data").is_some());
    assert!(response.get("Monthly Time Series").is_some());
}

#[tokio::test]
#[ignore]
async fn test_company_overview() {
    let client = setup_client();
    let request = json!({
        "tool": "company_overview",
        "params": {
            "symbol": "AAPL"
        }
    });
    
    let result = call_tool(&client, request).await;
    assert!(result.is_ok(), "Failed to call company_overview: {:?}", result.err());
    
    let ToolCallResult::DataFrame { data: response, .. } = result.unwrap() else {
        panic!("Expected DataFrame result");
    };
    
    // Verify key company information
    assert_eq!(response.get("Symbol").and_then(|v| v.as_str()), Some("AAPL"));
    assert_eq!(response.get("Name").and_then(|v| v.as_str()), Some("Apple Inc"));
    assert_eq!(response.get("Exchange").and_then(|v| v.as_str()), Some("NASDAQ"));
    
    // Verify numeric fields exist and are parseable
    let market_cap = response.get("MarketCapitalization")
        .and_then(|v| v.as_str())
        .expect("Should have MarketCapitalization");
    market_cap.parse::<u64>().expect("Market cap should be numeric");
    
    // Verify key financial metrics exist
    assert!(response.get("PERatio").is_some(), "Should have PE ratio");
    assert!(response.get("DividendYield").is_some(), "Should have dividend yield");
    assert!(response.get("52WeekHigh").is_some(), "Should have 52 week high");
    assert!(response.get("52WeekLow").is_some(), "Should have 52 week low");
    
    // Verify company description exists
    let description = response.get("Description")
        .and_then(|v| v.as_str())
        .expect("Should have company description");
    assert!(description.contains("Apple") || description.contains("iPhone"), 
        "Description should mention Apple or its products");
}

#[tokio::test]
#[ignore]
async fn test_earnings() {
    let client = setup_client();
    let request = json!({
        "tool": "earnings",
        "params": {
            "symbol": "TSLA"
        }
    });
    
    let result = call_tool(&client, request).await;
    assert!(result.is_ok(), "Failed to call earnings: {:?}", result.err());
    
    let ToolCallResult::DataFrame { data: response, .. } = result.unwrap() else {
        panic!("Expected DataFrame result");
    };
    // Earnings typically has quarterlyEarnings and annualEarnings
    assert!(response.get("symbol").is_some() || response.get("quarterlyEarnings").is_some());
}

#[tokio::test]
#[ignore]
async fn test_earnings_estimates() {
    let client = setup_client();
    let request = json!({
        "tool": "earnings_estimates",
        "params": {
            "symbol": "META",
            "horizon": "3month"
        }
    });
    
    let result = call_tool(&client, request).await;
    assert!(result.is_ok(), "Failed to call earnings_estimates: {:?}", result.err());
    
    let ToolCallResult::DataFrame { data: response, .. } = result.unwrap() else {
        panic!("Expected DataFrame result");
    };
    // Check for typical earnings estimates response structure
    assert!(response.is_object(), "Expected object response");
}

#[tokio::test]
#[ignore]
async fn test_income_statement() {
    let client = setup_client();
    let request = json!({
        "tool": "income_statement",
        "params": {
            "symbol": "NVDA"
        }
    });
    
    let result = call_tool(&client, request).await;
    assert!(result.is_ok(), "Failed to call income_statement: {:?}", result.err());
    
    let ToolCallResult::DataFrame { data: response, .. } = result.unwrap() else {
        panic!("Expected DataFrame result");
    };
    
    // Verify symbol
    assert_eq!(response.get("symbol").and_then(|v| v.as_str()), Some("NVDA"));
    
    // Verify annual reports exist and have data
    let annual_reports = response.get("annualReports")
        .and_then(|v| v.as_array())
        .expect("Should have annualReports array");
    assert!(!annual_reports.is_empty(), "Should have annual report data");
    
    // Check first annual report structure
    let first_report = &annual_reports[0];
    assert!(first_report.get("fiscalDateEnding").is_some());
    
    // Verify key income statement items
    let revenue = first_report.get("totalRevenue")
        .and_then(|v| v.as_str())
        .expect("Should have totalRevenue");
    revenue.parse::<i64>().expect("Revenue should be numeric");
    
    assert!(first_report.get("grossProfit").is_some(), "Should have gross profit");
    assert!(first_report.get("operatingIncome").is_some(), "Should have operating income");
    assert!(first_report.get("netIncome").is_some(), "Should have net income");
    
    // Verify quarterly reports also exist
    let quarterly_reports = response.get("quarterlyReports")
        .and_then(|v| v.as_array())
        .expect("Should have quarterlyReports array");
    assert!(quarterly_reports.len() >= 4, "Should have at least 4 quarters of data");
}

#[tokio::test]
#[ignore]
async fn test_balance_sheet() {
    let client = setup_client();
    let request = json!({
        "tool": "balance_sheet",
        "params": {
            "symbol": "JPM"
        }
    });
    
    let result = call_tool(&client, request).await;
    assert!(result.is_ok(), "Failed to call balance_sheet: {:?}", result.err());
    
    let ToolCallResult::DataFrame { data: response, .. } = result.unwrap() else {
        panic!("Expected DataFrame result");
    };
    
    // Verify symbol
    assert_eq!(response.get("symbol").and_then(|v| v.as_str()), Some("JPM"));
    
    // Verify annual reports
    let annual_reports = response.get("annualReports")
        .and_then(|v| v.as_array())
        .expect("Should have annualReports array");
    assert!(!annual_reports.is_empty(), "Should have annual report data");
    
    // Check balance sheet items in first report
    let first_report = &annual_reports[0];
    assert!(first_report.get("fiscalDateEnding").is_some(), "Should have fiscal date");
    
    // Key balance sheet items based on actual API response
    assert!(first_report.get("totalAssets").is_some(), "Should have total assets");
    assert!(first_report.get("totalLiabilities").is_some(), "Should have total liabilities");
    assert!(first_report.get("totalShareholderEquity").is_some(), "Should have shareholder equity");
    assert!(first_report.get("cashAndCashEquivalentsAtCarryingValue").is_some(), "Should have cash");
    assert!(first_report.get("currentNetReceivables").is_some(), "Should have receivables");
    assert!(first_report.get("currentAccountsPayable").is_some(), "Should have payables");
    assert!(first_report.get("longTermDebt").is_some(), "Should have long term debt");
    assert!(first_report.get("commonStock").is_some(), "Should have common stock");
    assert!(first_report.get("retainedEarnings").is_some(), "Should have retained earnings");
}

#[tokio::test]
#[ignore]
async fn test_cash_flow() {
    let client = setup_client();
    let request = json!({
        "tool": "cash_flow",
        "params": {
            "symbol": "NFLX"
        }
    });
    
    let result = call_tool(&client, request).await;
    assert!(result.is_ok(), "Failed to call cash_flow: {:?}", result.err());
    
    let ToolCallResult::DataFrame { data: response, .. } = result.unwrap() else {
        panic!("Expected DataFrame result");
    };
    
    // Verify symbol
    assert_eq!(response.get("symbol").and_then(|v| v.as_str()), Some("NFLX"));
    
    // Verify annual reports
    let annual_reports = response.get("annualReports")
        .and_then(|v| v.as_array())
        .expect("Should have annualReports array");
    assert!(!annual_reports.is_empty(), "Should have annual report data");
    
    // Check cash flow categories in first report
    let first_report = &annual_reports[0];
    assert!(first_report.get("fiscalDateEnding").is_some(), "Should have fiscal date");
    
    // Operating activities
    assert!(first_report.get("operatingCashflow").is_some(), "Should have operating cash flow");
    assert!(first_report.get("netIncome").is_some(), "Should have net income");
    assert!(first_report.get("depreciationDepletionAndAmortization").is_some(), "Should have depreciation");
    
    // Investing activities  
    assert!(first_report.get("capitalExpenditures").is_some(), "Should have capex");
    assert!(first_report.get("cashflowFromInvestment").is_some(), "Should have investing cash flow");
    
    // Financing activities
    assert!(first_report.get("cashflowFromFinancing").is_some(), "Should have financing cash flow");
    assert!(first_report.get("dividendPayout").is_some(), "Should have dividend info");
    
    // Net change in cash (correct field name)
    assert!(first_report.get("changeInCashAndCashEquivalents").is_some(), "Should have change in cash and equivalents");
}

#[tokio::test]
#[ignore]
async fn test_invalid_tool() {
    let client = setup_client();
    let request = json!({
        "tool": "invalid_tool_name",
        "params": {
            "symbol": "AAPL"
        }
    });
    
    let result = call_tool(&client, request).await;
    assert!(result.is_err(), "Expected error for invalid tool");
}

#[tokio::test]
#[ignore]
async fn test_missing_required_params() {
    let client = setup_client();
    
    // Test missing symbol
    let request = json!({
        "tool": "time_series_daily",
        "params": {}
    });
    
    let result = call_tool(&client, request).await;
    assert!(result.is_err(), "Expected error for missing symbol parameter");
    
    // Test missing interval for intraday
    let request = json!({
        "tool": "time_series_intraday",
        "params": {
            "symbol": "AAPL"
        }
    });
    
    let result = call_tool(&client, request).await;
    assert!(result.is_err(), "Expected error for missing interval parameter");
}

#[tokio::test]
#[ignore]
async fn test_invalid_interval() {
    let client = setup_client();
    let request = json!({
        "tool": "time_series_intraday",
        "params": {
            "symbol": "AAPL",
            "interval": "2min"  // Invalid interval
        }
    });
    
    let result = call_tool(&client, request).await;
    assert!(result.is_err(), "Expected error for invalid interval");
}

/// Test that optional parameters work correctly
#[tokio::test]
#[ignore]
async fn test_optional_params() {
    let client = setup_client();
    
    // Test earnings_estimates without horizon (should use default)
    let request = json!({
        "tool": "earnings_estimates",
        "params": {
            "symbol": "AAPL"
        }
    });
    
    let result = call_tool(&client, request).await;
    assert!(result.is_ok(), "Should work without optional horizon parameter");
    
    // Test time_series_daily without outputsize (should use default)
    let request = json!({
        "tool": "time_series_daily",
        "params": {
            "symbol": "MSFT"
        }
    });
    
    let result = call_tool(&client, request).await;
    assert!(result.is_ok(), "Should work without optional outputsize parameter");
}
