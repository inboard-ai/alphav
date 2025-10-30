//! LLM Tool Use Interface for Alpha Vantage API
//!
//! This module provides a progressive discovery interface for the Alpha Vantage API.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::client::AlphaVantage;
use crate::error::{Error, Result};
use crate::request::Request;
use crate::request::common::{Interval, OutputSize};
use crate::rest;
use crate::rest::fundamentals;

/// Tool information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    /// Tool unique identifier
    pub id: String,
    /// Human-readable tool name
    pub name: String,
    /// Description of what the tool does
    pub description: String,
    /// JSON Schema for the tool's parameters
    pub schema: Value,
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

/// Universal tool caller
pub async fn call_tool<Client: Request>(client: &AlphaVantage<Client>, request: Value) -> Result<Value> {
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
            serde_json::from_str(&response).map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))
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
            serde_json::from_str(&response).map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))
        }
        "time_series_weekly" => {
            let symbol = params
                .get("symbol")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Custom("Missing 'symbol' parameter".to_string()))?;

            let query = rest::time_series::weekly(client, symbol);
            let response = query.get().await?;
            serde_json::from_str(&response).map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))
        }
        "time_series_monthly" => {
            let symbol = params
                .get("symbol")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Custom("Missing 'symbol' parameter".to_string()))?;

            let query = rest::time_series::monthly(client, symbol);
            let response = query.get().await?;
            serde_json::from_str(&response).map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))
        }

        // Fundamental Data Endpoints
        "company_overview" => {
            let symbol = params
                .get("symbol")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Custom("Missing 'symbol' parameter".to_string()))?;

            let query = fundamentals::company_overview(client, symbol);
            let response = query.get().await?;
            serde_json::from_str(&response).map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))
        }
        "earnings" => {
            let symbol = params
                .get("symbol")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Custom("Missing 'symbol' parameter".to_string()))?;

            let query = fundamentals::earnings(client, symbol);
            let response = query.get().await?;
            serde_json::from_str(&response).map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))
        }
        "earnings_estimates" => {
            let symbol = params
                .get("symbol")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Custom("Missing 'symbol' parameter".to_string()))?;

            let mut query = fundamentals::earnings_estimates(client, symbol);

            if let Some(horizon) = params.get("horizon").and_then(|v| v.as_str()) {
                query = query.horizon(horizon);
            }

            let response = query.get().await?;
            serde_json::from_str(&response).map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))
        }
        "income_statement" => {
            let symbol = params
                .get("symbol")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Custom("Missing 'symbol' parameter".to_string()))?;

            let query = fundamentals::income_statement(client, symbol);
            let response = query.get().await?;
            serde_json::from_str(&response).map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))
        }
        "balance_sheet" => {
            let symbol = params
                .get("symbol")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Custom("Missing 'symbol' parameter".to_string()))?;

            let query = fundamentals::balance_sheet(client, symbol);
            let response = query.get().await?;
            serde_json::from_str(&response).map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))
        }
        "cash_flow" => {
            let symbol = params
                .get("symbol")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::Custom("Missing 'symbol' parameter".to_string()))?;

            let query = fundamentals::cash_flow(client, symbol);
            let response = query.get().await?;
            serde_json::from_str(&response).map_err(|e| Error::Custom(format!("Failed to parse response: {e}")))
        }

        _ => Err(Error::Custom(format!("Unknown tool: {tool}"))),
    }
}
