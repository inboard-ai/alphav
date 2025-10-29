use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::client::AlphaVantage;
use crate::error::Result;
use crate::execute::Execute;
use crate::processor::{Processor, Raw};
use crate::request::Request;
use crate::request::common::OutputSize;

/// Time series daily request builder
pub struct TimeSeriesDaily<'a, Client: Request, P: Processor = Raw> {
    client: &'a AlphaVantage<Client>,
    /// Stock symbol
    pub symbol: String,
    /// Output size (compact or full)
    pub outputsize: Option<OutputSize>,
    /// Data type (json or csv)
    pub datatype: Option<String>,
    processor: P,
}

// Constructor - always starts with Raw
impl<'a, C: Request> TimeSeriesDaily<'a, C, Raw> {
    /// Create new time series daily request (returns raw JSON by default)
    pub fn new(
        client: &'a AlphaVantage<C>,
        symbol: impl Into<String>,
    ) -> Self {
        Self {
            client,
            symbol: symbol.into(),
            outputsize: None,
            datatype: None,
            processor: Raw,
        }
    }
}

// Processor conversion and builder methods work on any processor type
impl<'a, C: Request, P: Processor + 'a> TimeSeriesDaily<'a, C, P> {
    /// Execute the request and return the result
    pub fn get(self) -> impl std::future::Future<Output = Result<P::Output>> + 'a {
        Execute::get(self)
    }

    /// Set output size
    pub fn outputsize(mut self, size: OutputSize) -> Self {
        self.outputsize = Some(size);
        self
    }

    /// Set datatype (json or csv)
    pub fn datatype(mut self, datatype: impl Into<String>) -> Self {
        self.datatype = Some(datatype.into());
        self
    }
}

impl<'a, C: Request, P: Processor + 'a> Execute for TimeSeriesDaily<'a, C, P> {
    type Output = P::Output;

    #[allow(refining_impl_trait_reachable)]
    async fn get(self) -> Result<P::Output> {
        // Build URL
        let api_key = self
            .client
            .api_key()
            .ok_or_else(|| crate::error::Error::Custom("API key not set".to_string()))?;

        let mut params = vec![
            format!("function=TIME_SERIES_DAILY"),
            format!("symbol={}", self.symbol),
            format!("apikey={}", api_key),
        ];

        if let Some(size) = self.outputsize {
            params.push(format!("outputsize={:?}", size).to_lowercase());
        }
        if let Some(datatype) = self.datatype {
            params.push(format!("datatype={}", datatype));
        }

        let url = format!("https://www.alphavantage.co/query?{}", params.join("&"));

        // Make request using Request trait
        let response = self.client.client().get(&url).await;

        // Process using associated Processor type
        self.processor.process(response)
    }
}

/// JSON-serializable parameters (no client reference)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Params {
    /// Stock symbol
    pub symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Output size (compact or full)
    pub outputsize: Option<OutputSize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Data type (json or csv)
    pub datatype: Option<String>,
}
