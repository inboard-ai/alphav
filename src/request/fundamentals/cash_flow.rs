use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::client::AlphaVantage;
use crate::error::Result;
use crate::execute::Execute;
use crate::processor::{Processor, Raw};
use crate::request::Request;

/// Cash flow request builder
pub struct CashFlow<'a, Client: Request, P: Processor = Raw> {
    client: &'a AlphaVantage<Client>,
    /// Stock symbol
    pub symbol: String,
    processor: P,
}

// Constructor - always starts with Raw
impl<'a, C: Request> CashFlow<'a, C, Raw> {
    /// Create new cash flow request (returns raw JSON by default)
    pub fn new(client: &'a AlphaVantage<C>, symbol: impl Into<String>) -> Self {
        Self {
            client,
            symbol: symbol.into(),
            processor: Raw,
        }
    }
}

// Processor conversion and builder methods work on any processor type
impl<'a, C: Request, P: Processor + 'a> CashFlow<'a, C, P> {
    /// Execute the request and return the result
    pub fn get(self) -> impl std::future::Future<Output = Result<P::Output>> + 'a {
        Execute::get(self)
    }
}

impl<'a, C: Request, P: Processor + 'a> Execute for CashFlow<'a, C, P> {
    type Output = P::Output;

    #[allow(refining_impl_trait_reachable)]
    async fn get(self) -> Result<P::Output> {
        // Build URL
        let api_key = self
            .client
            .api_key()
            .ok_or_else(|| crate::error::Error::Custom("API key not set".to_string()))?;

        let params = [
            "function=CASH_FLOW".to_string(),
            format!("symbol={}", self.symbol),
            format!("apikey={api_key}"),
        ];

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
}
