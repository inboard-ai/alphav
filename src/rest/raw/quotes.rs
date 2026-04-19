//! Quote endpoint implementations returning raw JSON strings

use crate::client::AlphaVantage;
use crate::processor::Raw;
use crate::request::Request;
use crate::request::realtime_bulk_quotes::RealtimeBulkQuotes;

/// Get realtime quotes for up to 100 symbols in a single API call.
///
/// Wraps Alpha Vantage's `REALTIME_BULK_QUOTES` endpoint. Requires a premium
/// Alpha Vantage subscription.
///
/// # Example
/// ```no_run
/// # use alphav::AlphaVantage;
/// # use alphav::execute::Execute;
/// # async fn example() {
/// # let client = AlphaVantage::default().with_key("api-key");
/// let json = alphav::rest::quotes::realtime_bulk(&client, ["AON", "SPY", "IBM"])
///     .get()
///     .await
///     .unwrap();
/// # }
/// ```
pub fn realtime_bulk<'a, Client, I, S>(
    client: &'a AlphaVantage<Client>,
    symbols: I,
) -> RealtimeBulkQuotes<'a, Client, Raw>
where
    Client: Request,
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    RealtimeBulkQuotes::new(client, symbols)
}
