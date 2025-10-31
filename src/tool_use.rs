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
            row.insert("open".to_string(), values_obj.get("1. open").cloned().unwrap_or(json!(null)));
            row.insert("high".to_string(), values_obj.get("2. high").cloned().unwrap_or(json!(null)));
            row.insert("low".to_string(), values_obj.get("3. low").cloned().unwrap_or(json!(null)));
            row.insert("close".to_string(), values_obj.get("4. close").cloned().unwrap_or(json!(null)));
            row.insert("volume".to_string(), values_obj.get("5. volume").cloned().unwrap_or(json!(null)));
            data_array.push(Value::Object(row));
        }
    }
    
    data_array.sort_by(|a, b| {
        let ts_a = a.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
        let ts_b = b.get("timestamp").and_then(|v| v.as_str()).unwrap_or("");
        ts_b.cmp(ts_a)
    });
    
    let schema = vec![
        ColumnDef { name: "timestamp".to_string(), alias: String::new(), dtype: "string".to_string() },
        ColumnDef { name: "open".to_string(), alias: String::new(), dtype: "number".to_string() },
        ColumnDef { name: "high".to_string(), alias: String::new(), dtype: "number".to_string() },
        ColumnDef { name: "low".to_string(), alias: String::new(), dtype: "number".to_string() },
        ColumnDef { name: "close".to_string(), alias: String::new(), dtype: "number".to_string() },
        ColumnDef { name: "volume".to_string(), alias: String::new(), dtype: "number".to_string() },
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
            row.insert("open".to_string(), values_obj.get("1. open").cloned().unwrap_or(json!(null)));
            row.insert("high".to_string(), values_obj.get("2. high").cloned().unwrap_or(json!(null)));
            row.insert("low".to_string(), values_obj.get("3. low").cloned().unwrap_or(json!(null)));
            row.insert("close".to_string(), values_obj.get("4. close").cloned().unwrap_or(json!(null)));
            row.insert("volume".to_string(), values_obj.get("5. volume").cloned().unwrap_or(json!(null)));
            data_array.push(Value::Object(row));
        }
    }
    
    data_array.sort_by(|a, b| {
        let date_a = a.get("date").and_then(|v| v.as_str()).unwrap_or("");
        let date_b = b.get("date").and_then(|v| v.as_str()).unwrap_or("");
        date_b.cmp(date_a)
    });
    
    let schema = vec![
        ColumnDef { name: "date".to_string(), alias: String::new(), dtype: "string".to_string() },
        ColumnDef { name: "open".to_string(), alias: String::new(), dtype: "number".to_string() },
        ColumnDef { name: "high".to_string(), alias: String::new(), dtype: "number".to_string() },
        ColumnDef { name: "low".to_string(), alias: String::new(), dtype: "number".to_string() },
        ColumnDef { name: "close".to_string(), alias: String::new(), dtype: "number".to_string() },
        ColumnDef { name: "volume".to_string(), alias: String::new(), dtype: "number".to_string() },
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
            row.insert("open".to_string(), values_obj.get("1. open").cloned().unwrap_or(json!(null)));
            row.insert("high".to_string(), values_obj.get("2. high").cloned().unwrap_or(json!(null)));
            row.insert("low".to_string(), values_obj.get("3. low").cloned().unwrap_or(json!(null)));
            row.insert("close".to_string(), values_obj.get("4. close").cloned().unwrap_or(json!(null)));
            row.insert("volume".to_string(), values_obj.get("5. volume").cloned().unwrap_or(json!(null)));
            data_array.push(Value::Object(row));
        }
    }
    
    data_array.sort_by(|a, b| {
        let date_a = a.get("week_ending").and_then(|v| v.as_str()).unwrap_or("");
        let date_b = b.get("week_ending").and_then(|v| v.as_str()).unwrap_or("");
        date_b.cmp(date_a)
    });
    
    let schema = vec![
        ColumnDef { name: "week_ending".to_string(), alias: String::new(), dtype: "string".to_string() },
        ColumnDef { name: "open".to_string(), alias: String::new(), dtype: "number".to_string() },
        ColumnDef { name: "high".to_string(), alias: String::new(), dtype: "number".to_string() },
        ColumnDef { name: "low".to_string(), alias: String::new(), dtype: "number".to_string() },
        ColumnDef { name: "close".to_string(), alias: String::new(), dtype: "number".to_string() },
        ColumnDef { name: "volume".to_string(), alias: String::new(), dtype: "number".to_string() },
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
            row.insert("open".to_string(), values_obj.get("1. open").cloned().unwrap_or(json!(null)));
            row.insert("high".to_string(), values_obj.get("2. high").cloned().unwrap_or(json!(null)));
            row.insert("low".to_string(), values_obj.get("3. low").cloned().unwrap_or(json!(null)));
            row.insert("close".to_string(), values_obj.get("4. close").cloned().unwrap_or(json!(null)));
            row.insert("volume".to_string(), values_obj.get("5. volume").cloned().unwrap_or(json!(null)));
            data_array.push(Value::Object(row));
        }
    }
    
    data_array.sort_by(|a, b| {
        let date_a = a.get("month").and_then(|v| v.as_str()).unwrap_or("");
        let date_b = b.get("month").and_then(|v| v.as_str()).unwrap_or("");
        date_b.cmp(date_a)
    });
    
    let schema = vec![
        ColumnDef { name: "month".to_string(), alias: String::new(), dtype: "string".to_string() },
        ColumnDef { name: "open".to_string(), alias: String::new(), dtype: "number".to_string() },
        ColumnDef { name: "high".to_string(), alias: String::new(), dtype: "number".to_string() },
        ColumnDef { name: "low".to_string(), alias: String::new(), dtype: "number".to_string() },
        ColumnDef { name: "close".to_string(), alias: String::new(), dtype: "number".to_string() },
        ColumnDef { name: "volume".to_string(), alias: String::new(), dtype: "number".to_string() },
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
            let response_json: Value = serde_json::from_str(&response)
                .map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))?;
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
            let response_json: Value = serde_json::from_str(&response)
                .map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))?;
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
            let response_json: Value = serde_json::from_str(&response)
                .map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))?;
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
            let response_json: Value = serde_json::from_str(&response)
                .map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))?;
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
            let data: Value = serde_json::from_str(&response)
                .map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))?;
            Ok(ToolCallResult::DataFrame { data, schema: vec![], metadata: None })
        }
        "earnings" => {
            let symbol = params
                .get("symbol")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Custom("Missing 'symbol' parameter".to_string()))?;

            let query = fundamentals::earnings(client, symbol);
            let response = query.get().await?;
            let data: Value = serde_json::from_str(&response)
                .map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))?;
            Ok(ToolCallResult::DataFrame { data, schema: vec![], metadata: None })
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
            let data: Value = serde_json::from_str(&response)
                .map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))?;
            Ok(ToolCallResult::DataFrame { data, schema: vec![], metadata: None })
        }
        "income_statement" => {
            let symbol = params
                .get("symbol")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Custom("Missing 'symbol' parameter".to_string()))?;

            let query = fundamentals::income_statement(client, symbol);
            let response = query.get().await?;
            let data: Value = serde_json::from_str(&response)
                .map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))?;
            Ok(ToolCallResult::DataFrame { data, schema: vec![], metadata: None })
        }
        "balance_sheet" => {
            let symbol = params
                .get("symbol")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Custom("Missing 'symbol' parameter".to_string()))?;

            let query = fundamentals::balance_sheet(client, symbol);
            let response = query.get().await?;
            let data: Value = serde_json::from_str(&response)
                .map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))?;
            Ok(ToolCallResult::DataFrame { data, schema: vec![], metadata: None })
        }
        "cash_flow" => {
            let symbol = params
                .get("symbol")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Custom("Missing 'symbol' parameter".to_string()))?;

            let query = fundamentals::cash_flow(client, symbol);
            let response = query.get().await?;
            let data: Value = serde_json::from_str(&response)
                .map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))?;
            Ok(ToolCallResult::DataFrame { data, schema: vec![], metadata: None })
        }

        _ => Err(Error::Custom(format!("Unknown tool: {tool}"))),
    }
}
