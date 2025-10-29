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
        
        // Try to extract common array fields (estimates, results, etc.)
        let data = json_value
            .get("estimates")
            .or_else(|| json_value.get("results"))
            .or_else(|| json_value.get("data"))
            .unwrap_or(&json_value);
        
        let json_bytes = serde_json::to_vec(data)?;
        let df = JsonReader::new(Cursor::new(json_bytes))
            .finish()
            .map_err(|e| crate::error::Error::Custom(format!("Failed to parse JSON as DataFrame: {e}")))?;
        Ok(df)
    }
}
