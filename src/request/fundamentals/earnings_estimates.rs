use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::client::AlphaVantage;
use crate::error::Result;
use crate::execute::Execute;
use crate::processor::{Processor, Raw};
use crate::request::Request;

/// Earnings estimates request builder
pub struct EarningsEstimates<'a, Client: Request, P: Processor = Raw> {
    client: &'a AlphaVantage<Client>,
    /// Stock symbol
    pub symbol: String,
    /// Horizon (e.g., 3month, 12month)
    pub horizon: Option<String>,
    processor: P,
}

// Constructor - always starts with Raw
impl<'a, C: Request> EarningsEstimates<'a, C, Raw> {
    /// Create new earnings estimates request (returns raw JSON by default)
    pub fn new(client: &'a AlphaVantage<C>, symbol: impl Into<String>) -> Self {
        Self {
            client,
            symbol: symbol.into(),
            horizon: None,
            processor: Raw,
        }
    }
}

// Processor conversion and builder methods work on any processor type
impl<'a, C: Request, P: Processor + 'a> EarningsEstimates<'a, C, P> {
    /// Execute the request and return the result
    pub fn get(self) -> impl std::future::Future<Output = Result<P::Output>> + 'a {
        Execute::get(self)
    }

    /// Set horizon (e.g., "3month", "12month")
    pub fn horizon(mut self, horizon: impl Into<String>) -> Self {
        self.horizon = Some(horizon.into());
        self
    }

    /// Convert to DataFrame output (Polars DataFrame)
    #[cfg(feature = "table")]
    pub fn as_dataframe(self) -> EarningsEstimates<'a, C, crate::processor::Table> {
        EarningsEstimates {
            client: self.client,
            symbol: self.symbol,
            horizon: self.horizon,
            processor: crate::processor::Table,
        }
    }
}

impl<'a, C: Request, P: Processor + 'a> Execute for EarningsEstimates<'a, C, P> {
    type Output = P::Output;

    #[allow(refining_impl_trait_reachable)]
    async fn get(self) -> Result<P::Output> {
        // Build URL
        let api_key = self
            .client
            .api_key()
            .ok_or_else(|| crate::error::Error::Custom("API key not set".to_string()))?;

        let mut params = vec![
            format!("function=EARNINGS_ESTIMATES"),
            format!("symbol={}", self.symbol),
            format!("apikey={}", api_key),
        ];

        if let Some(horizon) = self.horizon {
            params.push(format!("horizon={horizon}"));
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
    /// Horizon (e.g., 3month, 12month)
    pub horizon: Option<String>,
}
