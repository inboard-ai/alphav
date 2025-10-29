//! Common types used across multiple endpoints
use std::str::FromStr;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Sort order
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SortOrder {
    /// Ascending
    Asc,
    /// Descending
    Desc,
}

impl From<&str> for SortOrder {
    fn from(value: &str) -> Self {
        match value {
            "asc" => SortOrder::Asc,
            "desc" => SortOrder::Desc,
            _ => SortOrder::Asc, // default to ascending
        }
    }
}

impl From<String> for SortOrder {
    fn from(value: String) -> Self {
        SortOrder::from(value.as_str())
    }
}

/// Output size for Alpha Vantage time series
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum OutputSize {
    /// Compact (latest 100 data points)
    Compact,
    /// Full (20+ years of historical data)
    Full,
}

impl FromStr for OutputSize {
    type Err = crate::error::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "compact" => Ok(OutputSize::Compact),
            "full" => Ok(OutputSize::Full),
            _ => Err(crate::error::Error::Custom(format!("Invalid output size: {s}"))),
        }
    }
}

/// Time series interval
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Interval {
    /// 1 minute
    #[serde(rename = "1min")]
    OneMin,
    /// 5 minutes
    #[serde(rename = "5min")]
    FiveMin,
    /// 15 minutes
    #[serde(rename = "15min")]
    FifteenMin,
    /// 30 minutes
    #[serde(rename = "30min")]
    ThirtyMin,
    /// 60 minutes
    #[serde(rename = "60min")]
    SixtyMin,
}

impl FromStr for Interval {
    type Err = crate::error::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "1min" => Ok(Interval::OneMin),
            "5min" => Ok(Interval::FiveMin),
            "15min" => Ok(Interval::FifteenMin),
            "30min" => Ok(Interval::ThirtyMin),
            "60min" => Ok(Interval::SixtyMin),
            _ => Err(crate::error::Error::Custom(format!("Invalid interval: {s}"))),
        }
    }
}

impl std::fmt::Display for Interval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Interval::OneMin => write!(f, "1min"),
            Interval::FiveMin => write!(f, "5min"),
            Interval::FifteenMin => write!(f, "15min"),
            Interval::ThirtyMin => write!(f, "30min"),
            Interval::SixtyMin => write!(f, "60min"),
        }
    }
}
