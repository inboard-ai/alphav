//! Realtime bulk quotes request builder.
//!
//! Wraps Alpha Vantage's `REALTIME_BULK_QUOTES` endpoint, which accepts up to
//! 100 comma-separated symbols in a single API call and returns their current
//! quotes. This endpoint requires a premium Alpha Vantage subscription.
//!
//! Note: this endpoint only returns realtime snapshots. Historical bars still
//! have to be fetched one symbol at a time via the `TIME_SERIES_*` endpoints.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::client::AlphaVantage;
use crate::error::{Error, Result};
use crate::execute::Execute;
use crate::processor::{Processor, Raw};
use crate::request::Request;

/// Maximum number of symbols accepted by `REALTIME_BULK_QUOTES` in a single call.
pub const MAX_BULK_SYMBOLS: usize = 100;

/// Realtime bulk quotes request builder.
///
/// Accepts a list of symbols (up to [`MAX_BULK_SYMBOLS`]) which are sent to
/// Alpha Vantage as a comma-separated `symbol=` parameter.
pub struct RealtimeBulkQuotes<'a, Client: Request, P: Processor = Raw> {
    client: &'a AlphaVantage<Client>,
    /// Stock symbols to fetch quotes for
    pub symbols: Vec<String>,
    /// Data type (json or csv)
    pub datatype: Option<String>,
    processor: P,
}

impl<'a, C: Request> RealtimeBulkQuotes<'a, C, Raw> {
    /// Create a new realtime bulk quotes request (returns raw JSON by default).
    ///
    /// Accepts any iterable of string-like values.
    pub fn new<I, S>(client: &'a AlphaVantage<C>, symbols: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            client,
            symbols: symbols.into_iter().map(Into::into).collect(),
            datatype: None,
            processor: Raw,
        }
    }
}

impl<'a, C: Request, P: Processor + 'a> RealtimeBulkQuotes<'a, C, P> {
    /// Execute the request and return the result
    pub fn get(self) -> impl std::future::Future<Output = Result<P::Output>> + 'a {
        Execute::get(self)
    }

    /// Set datatype (json or csv)
    pub fn datatype(mut self, datatype: impl Into<String>) -> Self {
        self.datatype = Some(datatype.into());
        self
    }
}

impl<'a, C: Request, P: Processor + 'a> Execute for RealtimeBulkQuotes<'a, C, P> {
    type Output = P::Output;

    #[allow(refining_impl_trait_reachable)]
    async fn get(self) -> Result<P::Output> {
        if self.symbols.is_empty() {
            return Err(Error::Custom(
                "REALTIME_BULK_QUOTES requires at least one symbol".to_string(),
            ));
        }
        if self.symbols.len() > MAX_BULK_SYMBOLS {
            return Err(Error::Custom(format!(
                "REALTIME_BULK_QUOTES accepts at most {MAX_BULK_SYMBOLS} symbols, got {}",
                self.symbols.len()
            )));
        }

        let api_key = self
            .client
            .api_key()
            .ok_or_else(|| Error::Custom("API key not set".to_string()))?;

        let symbols = self.symbols.join(",");
        let mut params = vec![
            "function=REALTIME_BULK_QUOTES".to_string(),
            format!("symbol={symbols}"),
            format!("apikey={api_key}"),
        ];

        if let Some(datatype) = self.datatype {
            params.push(format!("datatype={datatype}"));
        }

        let url = format!("https://www.alphavantage.co/query?{}", params.join("&"));

        let response = self.client.client().get(&url).await;

        self.processor.process(response)
    }
}

/// JSON-serializable parameters (no client reference)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Params {
    /// Stock symbols to fetch quotes for
    pub symbols: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Data type (json or csv)
    pub datatype: Option<String>,
}
