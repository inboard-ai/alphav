//! LLM Tool Use Interface for Alpha Vantage API
//!
//! This module provides a progressive discovery interface for the Alpha Vantage API.

use serde_json::{Value, json};

use crate::client::AlphaVantage;
use crate::error::{Error, Result};
use crate::request::Request;
use crate::request::common::{Interval, OutputSize};
use crate::rest;
use crate::rest::fundamentals;

// Always use emporium-core types
pub use emporium_core::{ColumnDef, Schema, ToolInfo};

/// Result from executing a tool - can be text or structured data
#[derive(Debug, Clone)]
pub enum ToolCallResult {
    /// Plain text result
    Text(String),
    /// Structured tabular data with schema
    DataFrame {
        /// The actual JSON data
        data: Value,
        /// Column definitions describing the data structure
        schema: Schema,
        /// Optional metadata from the API response
        metadata: Option<Value>,
    },
}

/// Get details for a specific tool
pub fn get_tool_details(tool_id: &str) -> Option<ToolInfo> {
    list_tools().into_iter().find(|t| t.id == tool_id)
}

/// List all available tools
pub fn list_tools() -> Vec<ToolInfo> {
    vec![
        // Time Series Endpoints
        ToolInfo {
            id: "time_series_intraday".to_string(),
            name: "Intraday Time Series".to_string(),
            description: "Get intraday time series data with specified intervals".to_string(),
            schema: json!({
                "type": "object",
                "properties": {
                    "symbol": {
                        "type": "string",
                        "description": "Stock symbol (e.g., 'AAPL')"
                    },
                    "interval": {
                        "type": "string",
                        "enum": ["1min", "5min", "15min", "30min", "60min"],
                        "description": "Time interval between data points"
                    },
                    "outputsize": {
                        "type": "string",
                        "enum": ["compact", "full"],
                        "default": "compact",
                        "description": "Compact returns last 100 data points, full returns all"
                    }
                },
                "required": ["symbol", "interval"]
            }),
        },
        ToolInfo {
            id: "time_series_daily".to_string(),
            name: "Daily Time Series".to_string(),
            description: "Get daily time series data (open, high, low, close, volume)".to_string(),
            schema: json!({
                "type": "object",
                "properties": {
                    "symbol": {
                        "type": "string",
                        "description": "Stock symbol"
                    },
                    "outputsize": {
                        "type": "string",
                        "enum": ["compact", "full"],
                        "default": "compact"
                    }
                },
                "required": ["symbol"]
            }),
        },
        ToolInfo {
            id: "time_series_weekly".to_string(),
            name: "Weekly Time Series".to_string(),
            description: "Get weekly time series data".to_string(),
            schema: json!({
                "type": "object",
                "properties": {
                    "symbol": {
                        "type": "string",
                        "description": "Stock symbol"
                    }
                },
                "required": ["symbol"]
            }),
        },
        ToolInfo {
            id: "time_series_monthly".to_string(),
            name: "Monthly Time Series".to_string(),
            description: "Get monthly time series data".to_string(),
            schema: json!({
                "type": "object",
                "properties": {
                    "symbol": {
                        "type": "string",
                        "description": "Stock symbol"
                    }
                },
                "required": ["symbol"]
            }),
        },
        // Fundamental Data Endpoints
        ToolInfo {
            id: "company_overview".to_string(),
            name: "Company Overview".to_string(),
            description: "Get comprehensive company information including financials, ratios, and key metrics"
                .to_string(),
            schema: json!({
                "type": "object",
                "properties": {
                    "symbol": {
                        "type": "string",
                        "description": "Stock symbol"
                    }
                },
                "required": ["symbol"]
            }),
        },
        ToolInfo {
            id: "earnings".to_string(),
            name: "Earnings".to_string(),
            description: "Get quarterly and annual earnings data".to_string(),
            schema: json!({
                "type": "object",
                "properties": {
                    "symbol": {
                        "type": "string",
                        "description": "Stock symbol"
                    }
                },
                "required": ["symbol"]
            }),
        },
        ToolInfo {
            id: "earnings_estimates".to_string(),
            name: "Earnings Estimates".to_string(),
            description: "Get analysts' earnings estimates and consensus data".to_string(),
            schema: json!({
                "type": "object",
                "properties": {
                    "symbol": {
                        "type": "string",
                        "description": "Stock symbol"
                    },
                    "horizon": {
                        "type": "string",
                        "enum": ["3month", "6month", "12month", "all"],
                        "default": "all",
                        "description": "Time horizon for estimates"
                    }
                },
                "required": ["symbol"]
            }),
        },
        ToolInfo {
            id: "income_statement".to_string(),
            name: "Income Statement".to_string(),
            description: "Get annual and quarterly income statements".to_string(),
            schema: json!({
                "type": "object",
                "properties": {
                    "symbol": {
                        "type": "string",
                        "description": "Stock symbol"
                    }
                },
                "required": ["symbol"]
            }),
        },
        ToolInfo {
            id: "balance_sheet".to_string(),
            name: "Balance Sheet".to_string(),
            description: "Get annual and quarterly balance sheet data".to_string(),
            schema: json!({
                "type": "object",
                "properties": {
                    "symbol": {
                        "type": "string",
                        "description": "Stock symbol"
                    }
                },
                "required": ["symbol"]
            }),
        },
        ToolInfo {
            id: "cash_flow".to_string(),
            name: "Cash Flow Statement".to_string(),
            description: "Get annual and quarterly cash flow data".to_string(),
            schema: json!({
                "type": "object",
                "properties": {
                    "symbol": {
                        "type": "string",
                        "description": "Stock symbol"
                    }
                },
                "required": ["symbol"]
            }),
        },
    ]
}

/// Transform time_series_intraday response (1min, 5min, 15min, 30min, 60min intervals)
fn transform_intraday_response(response: Value, interval: &str) -> Result<(Value, Option<Value>, Schema)> {
    let metadata = response.get("Meta Data").cloned();

    // The exact key for intraday data
    let time_series_key = format!("Time Series ({})", interval);
    let time_series_obj = response
        .get(&time_series_key)
        .and_then(|v| v.as_object())
        .ok_or_else(|| Error::Custom(format!("No '{}' data found in response", time_series_key)))?;

    let mut data_array: Vec<Value> = Vec::new();
    for (timestamp, values) in time_series_obj {
        if let Some(values_obj) = values.as_object() {
            let mut row = serde_json::Map::new();
            row.insert("timestamp".to_string(), json!(timestamp));
            row.insert(
                "open".to_string(),
                values_obj.get("1. open").cloned().unwrap_or(json!(null)),
            );
            row.insert(
                "high".to_string(),
                values_obj.get("2. high").cloned().unwrap_or(json!(null)),
            );
            row.insert(
                "low".to_string(),
                values_obj.get("3. low").cloned().unwrap_or(json!(null)),
            );
            row.insert(
                "close".to_string(),
                values_obj.get("4. close").cloned().unwrap_or(json!(null)),
            );
            row.insert(
                "volume".to_string(),
                values_obj.get("5. volume").cloned().unwrap_or(json!(null)),
            );
            data_array.push(Value::Object(row));
        }
    }

    data_array.sort_by(|a, b| {
        let ts_a = a.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
        let ts_b = b.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
        ts_b.cmp(ts_a)
    });

    let schema = vec![
        ColumnDef {
            name: "timestamp".to_string(),
            alias: "Timestamp".to_string(),
            dtype: "string".to_string(),
        },
        ColumnDef {
            name: "open".to_string(),
            alias: "Open".to_string(),
            dtype: "number".to_string(),
        },
        ColumnDef {
            name: "high".to_string(),
            alias: "High".to_string(),
            dtype: "number".to_string(),
        },
        ColumnDef {
            name: "low".to_string(),
            alias: "Low".to_string(),
            dtype: "number".to_string(),
        },
        ColumnDef {
            name: "close".to_string(),
            alias: "Close".to_string(),
            dtype: "number".to_string(),
        },
        ColumnDef {
            name: "volume".to_string(),
            alias: "Volume".to_string(),
            dtype: "number".to_string(),
        },
    ];

    Ok((json!(data_array), metadata, schema))
}

/// Transform time_series_daily response
fn transform_daily_response(response: Value) -> Result<(Value, Option<Value>, Schema)> {
    let metadata = response.get("Meta Data").cloned();

    let time_series_obj = response
        .get("Time Series (Daily)")
        .and_then(|v| v.as_object())
        .ok_or_else(|| Error::Custom("No 'Time Series (Daily)' data found in response".to_string()))?;

    let mut data_array: Vec<Value> = Vec::new();
    for (date, values) in time_series_obj {
        if let Some(values_obj) = values.as_object() {
            let mut row = serde_json::Map::new();
            row.insert("date".to_string(), json!(date));
            row.insert(
                "open".to_string(),
                values_obj.get("1. open").cloned().unwrap_or(json!(null)),
            );
            row.insert(
                "high".to_string(),
                values_obj.get("2. high").cloned().unwrap_or(json!(null)),
            );
            row.insert(
                "low".to_string(),
                values_obj.get("3. low").cloned().unwrap_or(json!(null)),
            );
            row.insert(
                "close".to_string(),
                values_obj.get("4. close").cloned().unwrap_or(json!(null)),
            );
            row.insert(
                "volume".to_string(),
                values_obj.get("5. volume").cloned().unwrap_or(json!(null)),
            );
            data_array.push(Value::Object(row));
        }
    }

    data_array.sort_by(|a, b| {
        let date_a = a.get("date").and_then(|v| v.as_str()).unwrap_or("");
        let date_b = b.get("date").and_then(|v| v.as_str()).unwrap_or("");
        date_b.cmp(date_a)
    });

    let schema = vec![
        ColumnDef {
            name: "date".to_string(),
            alias: "Date".to_string(),
            dtype: "string".to_string(),
        },
        ColumnDef {
            name: "open".to_string(),
            alias: "Open".to_string(),
            dtype: "number".to_string(),
        },
        ColumnDef {
            name: "high".to_string(),
            alias: "High".to_string(),
            dtype: "number".to_string(),
        },
        ColumnDef {
            name: "low".to_string(),
            alias: "Low".to_string(),
            dtype: "number".to_string(),
        },
        ColumnDef {
            name: "close".to_string(),
            alias: "Close".to_string(),
            dtype: "number".to_string(),
        },
        ColumnDef {
            name: "volume".to_string(),
            alias: "Volume".to_string(),
            dtype: "number".to_string(),
        },
    ];

    Ok((json!(data_array), metadata, schema))
}

/// Transform time_series_weekly response
fn transform_weekly_response(response: Value) -> Result<(Value, Option<Value>, Schema)> {
    let metadata = response.get("Meta Data").cloned();

    let time_series_obj = response
        .get("Weekly Time Series")
        .and_then(|v| v.as_object())
        .ok_or_else(|| Error::Custom("No 'Weekly Time Series' data found in response".to_string()))?;

    let mut data_array: Vec<Value> = Vec::new();
    for (date, values) in time_series_obj {
        if let Some(values_obj) = values.as_object() {
            let mut row = serde_json::Map::new();
            row.insert("week_ending".to_string(), json!(date));
            row.insert(
                "open".to_string(),
                values_obj.get("1. open").cloned().unwrap_or(json!(null)),
            );
            row.insert(
                "high".to_string(),
                values_obj.get("2. high").cloned().unwrap_or(json!(null)),
            );
            row.insert(
                "low".to_string(),
                values_obj.get("3. low").cloned().unwrap_or(json!(null)),
            );
            row.insert(
                "close".to_string(),
                values_obj.get("4. close").cloned().unwrap_or(json!(null)),
            );
            row.insert(
                "volume".to_string(),
                values_obj.get("5. volume").cloned().unwrap_or(json!(null)),
            );
            data_array.push(Value::Object(row));
        }
    }

    data_array.sort_by(|a, b| {
        let date_a = a.get("week_ending").and_then(|v| v.as_str()).unwrap_or("");
        let date_b = b.get("week_ending").and_then(|v| v.as_str()).unwrap_or("");
        date_b.cmp(date_a)
    });

    let schema = vec![
        ColumnDef {
            name: "week_ending".to_string(),
            alias: "Week Ending".to_string(),
            dtype: "string".to_string(),
        },
        ColumnDef {
            name: "open".to_string(),
            alias: "Open".to_string(),
            dtype: "number".to_string(),
        },
        ColumnDef {
            name: "high".to_string(),
            alias: "High".to_string(),
            dtype: "number".to_string(),
        },
        ColumnDef {
            name: "low".to_string(),
            alias: "Low".to_string(),
            dtype: "number".to_string(),
        },
        ColumnDef {
            name: "close".to_string(),
            alias: "Close".to_string(),
            dtype: "number".to_string(),
        },
        ColumnDef {
            name: "volume".to_string(),
            alias: "Volume".to_string(),
            dtype: "number".to_string(),
        },
    ];

    Ok((json!(data_array), metadata, schema))
}

/// Transform time_series_monthly response
fn transform_monthly_response(response: Value) -> Result<(Value, Option<Value>, Schema)> {
    let metadata = response.get("Meta Data").cloned();

    let time_series_obj = response
        .get("Monthly Time Series")
        .and_then(|v| v.as_object())
        .ok_or_else(|| Error::Custom("No 'Monthly Time Series' data found in response".to_string()))?;

    let mut data_array: Vec<Value> = Vec::new();
    for (date, values) in time_series_obj {
        if let Some(values_obj) = values.as_object() {
            let mut row = serde_json::Map::new();
            row.insert("month".to_string(), json!(date));
            row.insert(
                "open".to_string(),
                values_obj.get("1. open").cloned().unwrap_or(json!(null)),
            );
            row.insert(
                "high".to_string(),
                values_obj.get("2. high").cloned().unwrap_or(json!(null)),
            );
            row.insert(
                "low".to_string(),
                values_obj.get("3. low").cloned().unwrap_or(json!(null)),
            );
            row.insert(
                "close".to_string(),
                values_obj.get("4. close").cloned().unwrap_or(json!(null)),
            );
            row.insert(
                "volume".to_string(),
                values_obj.get("5. volume").cloned().unwrap_or(json!(null)),
            );
            data_array.push(Value::Object(row));
        }
    }

    data_array.sort_by(|a, b| {
        let date_a = a.get("month").and_then(|v| v.as_str()).unwrap_or("");
        let date_b = b.get("month").and_then(|v| v.as_str()).unwrap_or("");
        date_b.cmp(date_a)
    });

    let schema = vec![
        ColumnDef {
            name: "month".to_string(),
            alias: "Month".to_string(),
            dtype: "string".to_string(),
        },
        ColumnDef {
            name: "open".to_string(),
            alias: "Open".to_string(),
            dtype: "number".to_string(),
        },
        ColumnDef {
            name: "high".to_string(),
            alias: "High".to_string(),
            dtype: "number".to_string(),
        },
        ColumnDef {
            name: "low".to_string(),
            alias: "Low".to_string(),
            dtype: "number".to_string(),
        },
        ColumnDef {
            name: "close".to_string(),
            alias: "Close".to_string(),
            dtype: "number".to_string(),
        },
        ColumnDef {
            name: "volume".to_string(),
            alias: "Volume".to_string(),
            dtype: "number".to_string(),
        },
    ];

    Ok((json!(data_array), metadata, schema))
}

/// Transform company_overview response
fn transform_company_overview_response(response: Value) -> Result<(Value, Option<Value>, Schema)> {
    // Company overview is a single object, so we convert it to a single-row array
    let data_array = vec![response.clone()];
    
    let schema = vec![
        ColumnDef { name: "Symbol".to_string(), alias: "Symbol".to_string(), dtype: "string".to_string() },
        ColumnDef { name: "Name".to_string(), alias: "Company Name".to_string(), dtype: "string".to_string() },
        ColumnDef { name: "AssetType".to_string(), alias: "Asset Type".to_string(), dtype: "string".to_string() },
        ColumnDef { name: "Exchange".to_string(), alias: "Exchange".to_string(), dtype: "string".to_string() },
        ColumnDef { name: "Currency".to_string(), alias: "Currency".to_string(), dtype: "string".to_string() },
        ColumnDef { name: "Country".to_string(), alias: "Country".to_string(), dtype: "string".to_string() },
        ColumnDef { name: "Sector".to_string(), alias: "Sector".to_string(), dtype: "string".to_string() },
        ColumnDef { name: "Industry".to_string(), alias: "Industry".to_string(), dtype: "string".to_string() },
        ColumnDef { name: "MarketCapitalization".to_string(), alias: "Market Cap".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "EBITDA".to_string(), alias: "EBITDA".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "PERatio".to_string(), alias: "P/E Ratio".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "PEGRatio".to_string(), alias: "PEG Ratio".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "BookValue".to_string(), alias: "Book Value".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "DividendPerShare".to_string(), alias: "Dividend Per Share".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "DividendYield".to_string(), alias: "Dividend Yield".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "EPS".to_string(), alias: "EPS".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "RevenuePerShareTTM".to_string(), alias: "Revenue Per Share TTM".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "ProfitMargin".to_string(), alias: "Profit Margin".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "OperatingMarginTTM".to_string(), alias: "Operating Margin TTM".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "ReturnOnAssetsTTM".to_string(), alias: "Return on Assets TTM".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "ReturnOnEquityTTM".to_string(), alias: "Return on Equity TTM".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "RevenueTTM".to_string(), alias: "Revenue TTM".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "GrossProfitTTM".to_string(), alias: "Gross Profit TTM".to_string(), dtype: "number".to_string() },
    ];
    
    Ok((json!(data_array), None, schema))
}

/// Transform earnings response
fn transform_earnings_response(response: Value) -> Result<(Value, Option<Value>, Schema)> {
    let symbol = response.get("symbol").cloned();
    let metadata = symbol.map(|s| json!({"symbol": s}));
    
    let mut data_array: Vec<Value> = Vec::new();
    
    // Add annual earnings
    if let Some(annual) = response.get("annualEarnings").and_then(|v| v.as_array()) {
        for earning in annual {
            let mut row = serde_json::Map::new();
            row.insert("period_type".to_string(), json!("annual"));
            row.insert("fiscal_date_ending".to_string(), earning.get("fiscalDateEnding").cloned().unwrap_or(json!(null)));
            row.insert("reported_eps".to_string(), earning.get("reportedEPS").cloned().unwrap_or(json!(null)));
            data_array.push(Value::Object(row));
        }
    }
    
    // Add quarterly earnings
    if let Some(quarterly) = response.get("quarterlyEarnings").and_then(|v| v.as_array()) {
        for earning in quarterly {
            let mut row = serde_json::Map::new();
            row.insert("period_type".to_string(), json!("quarterly"));
            row.insert("fiscal_date_ending".to_string(), earning.get("fiscalDateEnding").cloned().unwrap_or(json!(null)));
            row.insert("reported_eps".to_string(), earning.get("reportedEPS").cloned().unwrap_or(json!(null)));
            row.insert("reported_date".to_string(), earning.get("reportedDate").cloned().unwrap_or(json!(null)));
            row.insert("estimated_eps".to_string(), earning.get("estimatedEPS").cloned().unwrap_or(json!(null)));
            row.insert("surprise".to_string(), earning.get("surprise").cloned().unwrap_or(json!(null)));
            row.insert("surprise_percentage".to_string(), earning.get("surprisePercentage").cloned().unwrap_or(json!(null)));
            data_array.push(Value::Object(row));
        }
    }
    
    // Sort by fiscal date ending (most recent first)
    data_array.sort_by(|a, b| {
        let date_a = a.get("fiscal_date_ending").and_then(|v| v.as_str()).unwrap_or("");
        let date_b = b.get("fiscal_date_ending").and_then(|v| v.as_str()).unwrap_or("");
        date_b.cmp(date_a)
    });
    
    let schema = vec![
        ColumnDef { name: "period_type".to_string(), alias: "Period Type".to_string(), dtype: "string".to_string() },
        ColumnDef { name: "fiscal_date_ending".to_string(), alias: "Fiscal Date Ending".to_string(), dtype: "string".to_string() },
        ColumnDef { name: "reported_eps".to_string(), alias: "Reported EPS".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "reported_date".to_string(), alias: "Reported Date".to_string(), dtype: "string".to_string() },
        ColumnDef { name: "estimated_eps".to_string(), alias: "Estimated EPS".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "surprise".to_string(), alias: "Surprise".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "surprise_percentage".to_string(), alias: "Surprise %".to_string(), dtype: "number".to_string() },
    ];
    
    Ok((json!(data_array), metadata, schema))
}

/// Transform earnings_estimates response
fn transform_earnings_estimates_response(response: Value) -> Result<(Value, Option<Value>, Schema)> {
    let symbol = response.get("symbol").cloned();
    let metadata = symbol.map(|s| json!({"symbol": s}));
    
    let mut data_array: Vec<Value> = Vec::new();
    
    // The earnings estimates response has an "estimates" array
    if let Some(estimates_array) = response.get("estimates").and_then(|v| v.as_array()) {
        for estimate in estimates_array {
            let mut row = serde_json::Map::new();
            row.insert("date".to_string(), estimate.get("date").cloned().unwrap_or(json!(null)));
            row.insert("horizon".to_string(), estimate.get("horizon").cloned().unwrap_or(json!(null)));
            row.insert("eps_estimate_average".to_string(), estimate.get("eps_estimate_average").cloned().unwrap_or(json!(null)));
            row.insert("eps_estimate_high".to_string(), estimate.get("eps_estimate_high").cloned().unwrap_or(json!(null)));
            row.insert("eps_estimate_low".to_string(), estimate.get("eps_estimate_low").cloned().unwrap_or(json!(null)));
            row.insert("eps_estimate_analyst_count".to_string(), estimate.get("eps_estimate_analyst_count").cloned().unwrap_or(json!(null)));
            row.insert("revenue_estimate_average".to_string(), estimate.get("revenue_estimate_average").cloned().unwrap_or(json!(null)));
            row.insert("revenue_estimate_high".to_string(), estimate.get("revenue_estimate_high").cloned().unwrap_or(json!(null)));
            row.insert("revenue_estimate_low".to_string(), estimate.get("revenue_estimate_low").cloned().unwrap_or(json!(null)));
            row.insert("revenue_estimate_analyst_count".to_string(), estimate.get("revenue_estimate_analyst_count").cloned().unwrap_or(json!(null)));
            data_array.push(Value::Object(row));
        }
    }
    
    // Sort by date (most recent first)
    data_array.sort_by(|a, b| {
        let date_a = a.get("date").and_then(|v| v.as_str()).unwrap_or("");
        let date_b = b.get("date").and_then(|v| v.as_str()).unwrap_or("");
        date_b.cmp(date_a)
    });
    
    let schema = vec![
        ColumnDef { name: "date".to_string(), alias: "Date".to_string(), dtype: "string".to_string() },
        ColumnDef { name: "horizon".to_string(), alias: "Horizon".to_string(), dtype: "string".to_string() },
        ColumnDef { name: "eps_estimate_average".to_string(), alias: "EPS Estimate (Avg)".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "eps_estimate_high".to_string(), alias: "EPS Estimate (High)".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "eps_estimate_low".to_string(), alias: "EPS Estimate (Low)".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "eps_estimate_analyst_count".to_string(), alias: "EPS Analyst Count".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "revenue_estimate_average".to_string(), alias: "Revenue Estimate (Avg)".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "revenue_estimate_high".to_string(), alias: "Revenue Estimate (High)".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "revenue_estimate_low".to_string(), alias: "Revenue Estimate (Low)".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "revenue_estimate_analyst_count".to_string(), alias: "Revenue Analyst Count".to_string(), dtype: "number".to_string() },
    ];
    
    Ok((json!(data_array), metadata, schema))
}

/// Transform income_statement response
fn transform_income_statement_response(response: Value) -> Result<(Value, Option<Value>, Schema)> {
    let symbol = response.get("symbol").cloned();
    let metadata = symbol.map(|s| json!({"symbol": s}));
    
    let mut data_array: Vec<Value> = Vec::new();
    
    // Add annual reports
    if let Some(annual) = response.get("annualReports").and_then(|v| v.as_array()) {
        for report in annual {
            let mut row = serde_json::Map::new();
            row.insert("period_type".to_string(), json!("annual"));
            row.insert("fiscal_date_ending".to_string(), report.get("fiscalDateEnding").cloned().unwrap_or(json!(null)));
            row.insert("total_revenue".to_string(), report.get("totalRevenue").cloned().unwrap_or(json!(null)));
            row.insert("cost_of_revenue".to_string(), report.get("costOfRevenue").cloned().unwrap_or(json!(null)));
            row.insert("gross_profit".to_string(), report.get("grossProfit").cloned().unwrap_or(json!(null)));
            row.insert("operating_income".to_string(), report.get("operatingIncome").cloned().unwrap_or(json!(null)));
            row.insert("net_income".to_string(), report.get("netIncome").cloned().unwrap_or(json!(null)));
            row.insert("ebitda".to_string(), report.get("ebitda").cloned().unwrap_or(json!(null)));
            row.insert("ebit".to_string(), report.get("ebit").cloned().unwrap_or(json!(null)));
            row.insert("income_before_tax".to_string(), report.get("incomeBeforeTax").cloned().unwrap_or(json!(null)));
            row.insert("income_tax_expense".to_string(), report.get("incomeTaxExpense").cloned().unwrap_or(json!(null)));
            data_array.push(Value::Object(row));
        }
    }
    
    // Add quarterly reports
    if let Some(quarterly) = response.get("quarterlyReports").and_then(|v| v.as_array()) {
        for report in quarterly {
            let mut row = serde_json::Map::new();
            row.insert("period_type".to_string(), json!("quarterly"));
            row.insert("fiscal_date_ending".to_string(), report.get("fiscalDateEnding").cloned().unwrap_or(json!(null)));
            row.insert("total_revenue".to_string(), report.get("totalRevenue").cloned().unwrap_or(json!(null)));
            row.insert("cost_of_revenue".to_string(), report.get("costOfRevenue").cloned().unwrap_or(json!(null)));
            row.insert("gross_profit".to_string(), report.get("grossProfit").cloned().unwrap_or(json!(null)));
            row.insert("operating_income".to_string(), report.get("operatingIncome").cloned().unwrap_or(json!(null)));
            row.insert("net_income".to_string(), report.get("netIncome").cloned().unwrap_or(json!(null)));
            row.insert("ebitda".to_string(), report.get("ebitda").cloned().unwrap_or(json!(null)));
            row.insert("ebit".to_string(), report.get("ebit").cloned().unwrap_or(json!(null)));
            row.insert("income_before_tax".to_string(), report.get("incomeBeforeTax").cloned().unwrap_or(json!(null)));
            row.insert("income_tax_expense".to_string(), report.get("incomeTaxExpense").cloned().unwrap_or(json!(null)));
            data_array.push(Value::Object(row));
        }
    }
    
    // Sort by fiscal date ending (most recent first)
    data_array.sort_by(|a, b| {
        let date_a = a.get("fiscal_date_ending").and_then(|v| v.as_str()).unwrap_or("");
        let date_b = b.get("fiscal_date_ending").and_then(|v| v.as_str()).unwrap_or("");
        date_b.cmp(date_a)
    });
    
    let schema = vec![
        ColumnDef { name: "period_type".to_string(), alias: "Period Type".to_string(), dtype: "string".to_string() },
        ColumnDef { name: "fiscal_date_ending".to_string(), alias: "Fiscal Date Ending".to_string(), dtype: "string".to_string() },
        ColumnDef { name: "total_revenue".to_string(), alias: "Total Revenue".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "cost_of_revenue".to_string(), alias: "Cost of Revenue".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "gross_profit".to_string(), alias: "Gross Profit".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "operating_income".to_string(), alias: "Operating Income".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "net_income".to_string(), alias: "Net Income".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "ebitda".to_string(), alias: "EBITDA".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "ebit".to_string(), alias: "EBIT".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "income_before_tax".to_string(), alias: "Income Before Tax".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "income_tax_expense".to_string(), alias: "Income Tax Expense".to_string(), dtype: "number".to_string() },
    ];
    
    Ok((json!(data_array), metadata, schema))
}

/// Transform balance_sheet response
fn transform_balance_sheet_response(response: Value) -> Result<(Value, Option<Value>, Schema)> {
    let symbol = response.get("symbol").cloned();
    let metadata = symbol.map(|s| json!({"symbol": s}));
    
    let mut data_array: Vec<Value> = Vec::new();
    
    // Add annual reports
    if let Some(annual) = response.get("annualReports").and_then(|v| v.as_array()) {
        for report in annual {
            let mut row = serde_json::Map::new();
            row.insert("period_type".to_string(), json!("annual"));
            row.insert("fiscal_date_ending".to_string(), report.get("fiscalDateEnding").cloned().unwrap_or(json!(null)));
            row.insert("total_assets".to_string(), report.get("totalAssets").cloned().unwrap_or(json!(null)));
            row.insert("total_current_assets".to_string(), report.get("totalCurrentAssets").cloned().unwrap_or(json!(null)));
            row.insert("cash_and_cash_equivalents".to_string(), report.get("cashAndCashEquivalentsAtCarryingValue").cloned().unwrap_or(json!(null)));
            row.insert("total_liabilities".to_string(), report.get("totalLiabilities").cloned().unwrap_or(json!(null)));
            row.insert("total_current_liabilities".to_string(), report.get("totalCurrentLiabilities").cloned().unwrap_or(json!(null)));
            row.insert("total_shareholder_equity".to_string(), report.get("totalShareholderEquity").cloned().unwrap_or(json!(null)));
            row.insert("retained_earnings".to_string(), report.get("retainedEarnings").cloned().unwrap_or(json!(null)));
            row.insert("common_stock".to_string(), report.get("commonStock").cloned().unwrap_or(json!(null)));
            data_array.push(Value::Object(row));
        }
    }
    
    // Add quarterly reports
    if let Some(quarterly) = response.get("quarterlyReports").and_then(|v| v.as_array()) {
        for report in quarterly {
            let mut row = serde_json::Map::new();
            row.insert("period_type".to_string(), json!("quarterly"));
            row.insert("fiscal_date_ending".to_string(), report.get("fiscalDateEnding").cloned().unwrap_or(json!(null)));
            row.insert("total_assets".to_string(), report.get("totalAssets").cloned().unwrap_or(json!(null)));
            row.insert("total_current_assets".to_string(), report.get("totalCurrentAssets").cloned().unwrap_or(json!(null)));
            row.insert("cash_and_cash_equivalents".to_string(), report.get("cashAndCashEquivalentsAtCarryingValue").cloned().unwrap_or(json!(null)));
            row.insert("total_liabilities".to_string(), report.get("totalLiabilities").cloned().unwrap_or(json!(null)));
            row.insert("total_current_liabilities".to_string(), report.get("totalCurrentLiabilities").cloned().unwrap_or(json!(null)));
            row.insert("total_shareholder_equity".to_string(), report.get("totalShareholderEquity").cloned().unwrap_or(json!(null)));
            row.insert("retained_earnings".to_string(), report.get("retainedEarnings").cloned().unwrap_or(json!(null)));
            row.insert("common_stock".to_string(), report.get("commonStock").cloned().unwrap_or(json!(null)));
            data_array.push(Value::Object(row));
        }
    }
    
    // Sort by fiscal date ending (most recent first)
    data_array.sort_by(|a, b| {
        let date_a = a.get("fiscal_date_ending").and_then(|v| v.as_str()).unwrap_or("");
        let date_b = b.get("fiscal_date_ending").and_then(|v| v.as_str()).unwrap_or("");
        date_b.cmp(date_a)
    });
    
    let schema = vec![
        ColumnDef { name: "period_type".to_string(), alias: "Period Type".to_string(), dtype: "string".to_string() },
        ColumnDef { name: "fiscal_date_ending".to_string(), alias: "Fiscal Date Ending".to_string(), dtype: "string".to_string() },
        ColumnDef { name: "total_assets".to_string(), alias: "Total Assets".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "total_current_assets".to_string(), alias: "Total Current Assets".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "cash_and_cash_equivalents".to_string(), alias: "Cash & Cash Equivalents".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "total_liabilities".to_string(), alias: "Total Liabilities".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "total_current_liabilities".to_string(), alias: "Total Current Liabilities".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "total_shareholder_equity".to_string(), alias: "Total Shareholder Equity".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "retained_earnings".to_string(), alias: "Retained Earnings".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "common_stock".to_string(), alias: "Common Stock".to_string(), dtype: "number".to_string() },
    ];
    
    Ok((json!(data_array), metadata, schema))
}

/// Transform cash_flow response
fn transform_cash_flow_response(response: Value) -> Result<(Value, Option<Value>, Schema)> {
    let symbol = response.get("symbol").cloned();
    let metadata = symbol.map(|s| json!({"symbol": s}));
    
    let mut data_array: Vec<Value> = Vec::new();
    
    // Add annual reports
    if let Some(annual) = response.get("annualReports").and_then(|v| v.as_array()) {
        for report in annual {
            let mut row = serde_json::Map::new();
            row.insert("period_type".to_string(), json!("annual"));
            row.insert("fiscal_date_ending".to_string(), report.get("fiscalDateEnding").cloned().unwrap_or(json!(null)));
            row.insert("operating_cashflow".to_string(), report.get("operatingCashflow").cloned().unwrap_or(json!(null)));
            row.insert("payments_for_operating_activities".to_string(), report.get("paymentsForOperatingActivities").cloned().unwrap_or(json!(null)));
            row.insert("proceeds_from_operating_activities".to_string(), report.get("proceedsFromOperatingActivities").cloned().unwrap_or(json!(null)));
            row.insert("change_in_operating_liabilities".to_string(), report.get("changeInOperatingLiabilities").cloned().unwrap_or(json!(null)));
            row.insert("change_in_operating_assets".to_string(), report.get("changeInOperatingAssets").cloned().unwrap_or(json!(null)));
            row.insert("depreciation_depletion_and_amortization".to_string(), report.get("depreciationDepletionAndAmortization").cloned().unwrap_or(json!(null)));
            row.insert("capital_expenditures".to_string(), report.get("capitalExpenditures").cloned().unwrap_or(json!(null)));
            row.insert("change_in_receivables".to_string(), report.get("changeInReceivables").cloned().unwrap_or(json!(null)));
            row.insert("change_in_inventory".to_string(), report.get("changeInInventory").cloned().unwrap_or(json!(null)));
            row.insert("profit_loss".to_string(), report.get("profitLoss").cloned().unwrap_or(json!(null)));
            row.insert("cashflow_from_investment".to_string(), report.get("cashflowFromInvestment").cloned().unwrap_or(json!(null)));
            row.insert("cashflow_from_financing".to_string(), report.get("cashflowFromFinancing").cloned().unwrap_or(json!(null)));
            data_array.push(Value::Object(row));
        }
    }
    
    // Add quarterly reports
    if let Some(quarterly) = response.get("quarterlyReports").and_then(|v| v.as_array()) {
        for report in quarterly {
            let mut row = serde_json::Map::new();
            row.insert("period_type".to_string(), json!("quarterly"));
            row.insert("fiscal_date_ending".to_string(), report.get("fiscalDateEnding").cloned().unwrap_or(json!(null)));
            row.insert("operating_cashflow".to_string(), report.get("operatingCashflow").cloned().unwrap_or(json!(null)));
            row.insert("payments_for_operating_activities".to_string(), report.get("paymentsForOperatingActivities").cloned().unwrap_or(json!(null)));
            row.insert("proceeds_from_operating_activities".to_string(), report.get("proceedsFromOperatingActivities").cloned().unwrap_or(json!(null)));
            row.insert("change_in_operating_liabilities".to_string(), report.get("changeInOperatingLiabilities").cloned().unwrap_or(json!(null)));
            row.insert("change_in_operating_assets".to_string(), report.get("changeInOperatingAssets").cloned().unwrap_or(json!(null)));
            row.insert("depreciation_depletion_and_amortization".to_string(), report.get("depreciationDepletionAndAmortization").cloned().unwrap_or(json!(null)));
            row.insert("capital_expenditures".to_string(), report.get("capitalExpenditures").cloned().unwrap_or(json!(null)));
            row.insert("change_in_receivables".to_string(), report.get("changeInReceivables").cloned().unwrap_or(json!(null)));
            row.insert("change_in_inventory".to_string(), report.get("changeInInventory").cloned().unwrap_or(json!(null)));
            row.insert("profit_loss".to_string(), report.get("profitLoss").cloned().unwrap_or(json!(null)));
            row.insert("cashflow_from_investment".to_string(), report.get("cashflowFromInvestment").cloned().unwrap_or(json!(null)));
            row.insert("cashflow_from_financing".to_string(), report.get("cashflowFromFinancing").cloned().unwrap_or(json!(null)));
            data_array.push(Value::Object(row));
        }
    }
    
    // Sort by fiscal date ending (most recent first)
    data_array.sort_by(|a, b| {
        let date_a = a.get("fiscal_date_ending").and_then(|v| v.as_str()).unwrap_or("");
        let date_b = b.get("fiscal_date_ending").and_then(|v| v.as_str()).unwrap_or("");
        date_b.cmp(date_a)
    });
    
    let schema = vec![
        ColumnDef { name: "period_type".to_string(), alias: "Period Type".to_string(), dtype: "string".to_string() },
        ColumnDef { name: "fiscal_date_ending".to_string(), alias: "Fiscal Date Ending".to_string(), dtype: "string".to_string() },
        ColumnDef { name: "operating_cashflow".to_string(), alias: "Operating Cash Flow".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "payments_for_operating_activities".to_string(), alias: "Payments for Operating Activities".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "proceeds_from_operating_activities".to_string(), alias: "Proceeds from Operating Activities".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "change_in_operating_liabilities".to_string(), alias: "Change in Operating Liabilities".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "change_in_operating_assets".to_string(), alias: "Change in Operating Assets".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "depreciation_depletion_and_amortization".to_string(), alias: "Depreciation & Amortization".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "capital_expenditures".to_string(), alias: "Capital Expenditures".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "change_in_receivables".to_string(), alias: "Change in Receivables".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "change_in_inventory".to_string(), alias: "Change in Inventory".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "profit_loss".to_string(), alias: "Profit/Loss".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "cashflow_from_investment".to_string(), alias: "Cash Flow from Investment".to_string(), dtype: "number".to_string() },
        ColumnDef { name: "cashflow_from_financing".to_string(), alias: "Cash Flow from Financing".to_string(), dtype: "number".to_string() },
    ];
    
    Ok((json!(data_array), metadata, schema))
}

/// Universal tool caller
pub async fn call_tool<Client: Request>(client: &AlphaVantage<Client>, request: Value) -> Result<ToolCallResult> {
    let tool = request
        .get("tool")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::Custom("Missing 'tool' field".to_string()))?;

    let params = request
        .get("params")
        .ok_or_else(|| Error::Custom("Missing 'params' field".to_string()))?;

    match tool {
        // Time Series Endpoints
        "time_series_intraday" => {
            let symbol = params
                .get("symbol")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Custom("Missing 'symbol' parameter".to_string()))?;

            let interval = params
                .get("interval")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Custom("Missing 'interval' parameter".to_string()))?;

            let interval_enum = match interval {
                "1min" => Interval::OneMin,
                "5min" => Interval::FiveMin,
                "15min" => Interval::FifteenMin,
                "30min" => Interval::ThirtyMin,
                "60min" => Interval::SixtyMin,
                _ => return Err(Error::Custom(format!("Invalid interval: {interval}"))),
            };

            let mut query = rest::time_series::intraday(client, symbol, interval_enum);

            if let Some(outputsize) = params.get("outputsize").and_then(|v| v.as_str()) {
                let size = match outputsize {
                    "compact" => OutputSize::Compact,
                    "full" => OutputSize::Full,
                    _ => return Err(Error::Custom(format!("Invalid outputsize: {outputsize}"))),
                };
                query = query.outputsize(size);
            }

            let response = query.get().await?;
            let response_json: Value =
                serde_json::from_str(&response).map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))?;
            let (data, metadata, schema) = transform_intraday_response(response_json, interval)?;
            Ok(ToolCallResult::DataFrame { data, schema, metadata })
        }
        "time_series_daily" => {
            let symbol = params
                .get("symbol")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Custom("Missing 'symbol' parameter".to_string()))?;

            let mut query = rest::time_series::daily(client, symbol);

            if let Some(outputsize) = params.get("outputsize").and_then(|v| v.as_str()) {
                let size = match outputsize {
                    "compact" => OutputSize::Compact,
                    "full" => OutputSize::Full,
                    _ => return Err(Error::Custom(format!("Invalid outputsize: {outputsize}"))),
                };
                query = query.outputsize(size);
            }

            let response = query.get().await?;
            let response_json: Value =
                serde_json::from_str(&response).map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))?;
            let (data, metadata, schema) = transform_daily_response(response_json)?;
            Ok(ToolCallResult::DataFrame { data, schema, metadata })
        }
        "time_series_weekly" => {
            let symbol = params
                .get("symbol")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Custom("Missing 'symbol' parameter".to_string()))?;

            let query = rest::time_series::weekly(client, symbol);
            let response = query.get().await?;
            let response_json: Value =
                serde_json::from_str(&response).map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))?;
            let (data, metadata, schema) = transform_weekly_response(response_json)?;
            Ok(ToolCallResult::DataFrame { data, schema, metadata })
        }
        "time_series_monthly" => {
            let symbol = params
                .get("symbol")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Custom("Missing 'symbol' parameter".to_string()))?;

            let query = rest::time_series::monthly(client, symbol);
            let response = query.get().await?;
            let response_json: Value =
                serde_json::from_str(&response).map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))?;
            let (data, metadata, schema) = transform_monthly_response(response_json)?;
            Ok(ToolCallResult::DataFrame { data, schema, metadata })
        }

        // Fundamental Data Endpoints
        "company_overview" => {
            let symbol = params
                .get("symbol")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Custom("Missing 'symbol' parameter".to_string()))?;

            let query = fundamentals::company_overview(client, symbol);
            let response = query.get().await?;
            let response_json: Value =
                serde_json::from_str(&response).map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))?;
            let (data, metadata, schema) = transform_company_overview_response(response_json)?;
            Ok(ToolCallResult::DataFrame { data, schema, metadata })
        }
        "earnings" => {
            let symbol = params
                .get("symbol")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Custom("Missing 'symbol' parameter".to_string()))?;

            let query = fundamentals::earnings(client, symbol);
            let response = query.get().await?;
            let response_json: Value =
                serde_json::from_str(&response).map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))?;
            let (data, metadata, schema) = transform_earnings_response(response_json)?;
            Ok(ToolCallResult::DataFrame { data, schema, metadata })
        }
        "earnings_estimates" => {
            let symbol = params
                .get("symbol")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Custom("Missing 'symbol' parameter".to_string()))?;

            let mut query = fundamentals::earnings_estimates(client, symbol);

            if let Some(horizon) = params.get("horizon").and_then(|v| v.as_str()) {
                query = query.horizon(horizon.to_string());
            }

            let response = query.get().await?;
            let response_json: Value =
                serde_json::from_str(&response).map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))?;
            let (data, metadata, schema) = transform_earnings_estimates_response(response_json)?;
            Ok(ToolCallResult::DataFrame { data, schema, metadata })
        }
        "income_statement" => {
            let symbol = params
                .get("symbol")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Custom("Missing 'symbol' parameter".to_string()))?;

            let query = fundamentals::income_statement(client, symbol);
            let response = query.get().await?;
            let response_json: Value =
                serde_json::from_str(&response).map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))?;
            let (data, metadata, schema) = transform_income_statement_response(response_json)?;
            Ok(ToolCallResult::DataFrame { data, schema, metadata })
        }
        "balance_sheet" => {
            let symbol = params
                .get("symbol")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Custom("Missing 'symbol' parameter".to_string()))?;

            let query = fundamentals::balance_sheet(client, symbol);
            let response = query.get().await?;
            let response_json: Value =
                serde_json::from_str(&response).map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))?;
            let (data, metadata, schema) = transform_balance_sheet_response(response_json)?;
            Ok(ToolCallResult::DataFrame { data, schema, metadata })
        }
        "cash_flow" => {
            let symbol = params
                .get("symbol")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Custom("Missing 'symbol' parameter".to_string()))?;

            let query = fundamentals::cash_flow(client, symbol);
            let response = query.get().await?;
            let response_json: Value =
                serde_json::from_str(&response).map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))?;
            let (data, metadata, schema) = transform_cash_flow_response(response_json)?;
            Ok(ToolCallResult::DataFrame { data, schema, metadata })
        }

        _ => Err(Error::Custom(format!("Unknown tool: {tool}"))),
    }
}
