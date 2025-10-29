//! Table processor using Polars DataFrames
use crate::error::Result;
use crate::processor::Processor;
use crate::response::Response;
use polars_core::frame::DataFrame;
use polars_io::prelude::*;
use std::io::Cursor;

/// Table processor that converts JSON responses to Polars DataFrames
pub struct Table;

impl Processor for Table {
    type Output = DataFrame;

    fn process<R: Response>(&self, response: Result<R>) -> Result<DataFrame> {
        let resp = response?;
        if resp.status() != 200 {
            return Err(crate::error::Error::ApiError {
                request_id: resp.request_id().to_owned(),
                status: resp.status().to_owned(),
                message: resp.body().to_owned(),
            });
        }

        let json_value: serde_json::Value = serde_json::from_str(resp.body())?;

        // Try to extract common array fields (estimates, results, annualReports, etc.)
        let data = json_value
            .get("estimates")
            .or_else(|| json_value.get("results"))
            .or_else(|| json_value.get("annualReports"))
            .or_else(|| json_value.get("quarterlyReports"))
            .or_else(|| json_value.get("data"))
            .unwrap_or(&json_value);

        // Ensure we have an array
        if !data.is_array() {
            return Err(crate::error::Error::Custom(format!(
                "Expected array data for DataFrame conversion, got: {}",
                if data.is_object() {
                    "object"
                } else if data.is_null() {
                    "null"
                } else {
                    "other"
                }
            )));
        }

        // Allow empty arrays - we must preserve original API responses per Alpha Vantage guidelines
        let json_bytes = serde_json::to_vec(data)?;
        let json_preview = String::from_utf8_lossy(&json_bytes[..json_bytes.len().min(200)]).to_string();
        let df = JsonReader::new(Cursor::new(json_bytes)).finish().map_err(|e| {
            crate::error::Error::Custom(format!(
                "Failed to parse JSON as DataFrame: {}. Data preview: {}",
                e, json_preview
            ))
        })?;
        Ok(df)
    }
}
