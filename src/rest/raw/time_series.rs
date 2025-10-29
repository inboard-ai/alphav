//! Time series endpoint implementations returning raw JSON strings

use crate::client::AlphaVantage;
use crate::processor::Raw;
use crate::request::Request;
use crate::request::time_series::{TimeSeriesIntraday, TimeSeriesDaily, TimeSeriesWeekly, TimeSeriesMonthly};
use crate::request::common::Interval;

/// Get intraday time series for a stock
///
/// Returns a request builder that will return results as raw JSON string.
///
/// # Example
/// ```no_run
/// # use alphav::AlphaVantage;
/// # use alphav::execute::Execute;
/// # use alphav::request::common::Interval;
/// # async fn example() {
/// # let client = AlphaVantage::default().with_key("api-key");
/// let json = alphav::rest::time_series::intraday(&client, "AAPL", Interval::FiveMin)
///     .get()
///     .await
///     .unwrap();
/// # }
/// ```
pub fn intraday<'a, Client: Request>(
    client: &'a AlphaVantage<Client>,
    symbol: impl Into<String>,
    interval: Interval,
) -> TimeSeriesIntraday<'a, Client, Raw> {
    TimeSeriesIntraday::new(client, symbol, interval)
}

/// Get daily time series for a stock
///
/// Returns a request builder that will return results as raw JSON string.
pub fn daily<'a, Client: Request>(
    client: &'a AlphaVantage<Client>,
    symbol: impl Into<String>,
) -> TimeSeriesDaily<'a, Client, Raw> {
    TimeSeriesDaily::new(client, symbol)
}

/// Get weekly time series for a stock
///
/// Returns a request builder that will return results as raw JSON string.
pub fn weekly<'a, Client: Request>(
    client: &'a AlphaVantage<Client>,
    symbol: impl Into<String>,
) -> TimeSeriesWeekly<'a, Client, Raw> {
    TimeSeriesWeekly::new(client, symbol)
}

/// Get monthly time series for a stock
///
/// Returns a request builder that will return results as raw JSON string.
pub fn monthly<'a, Client: Request>(
    client: &'a AlphaVantage<Client>,
    symbol: impl Into<String>,
) -> TimeSeriesMonthly<'a, Client, Raw> {
    TimeSeriesMonthly::new(client, symbol)
}

#[cfg(all(test, feature = "dotenvy"))]
mod tests {
    use super::*;

    fn setup() -> AlphaVantage<reqwest::Client> {
        AlphaVantage::new().expect("Failed to create client. Make sure ALPHAVANTAGE_API_KEY is set in .env file")
    }

    #[tokio::test]
    #[ignore] // Run with: cargo test -- --ignored --test-threads=1
    async fn test_intraday() {
        let client = setup();
        let result = intraday(&client, "AAPL", Interval::FiveMin)
            .get()
            .await;
        assert!(result.is_ok(), "Failed to fetch intraday data: {result:?}");
    }

    #[tokio::test]
    #[ignore]
    async fn test_daily() {
        let client = setup();
        let result = daily(&client, "AAPL").get().await;
        assert!(result.is_ok(), "Failed to fetch daily data: {result:?}");
    }

    #[tokio::test]
    #[ignore]
    async fn test_weekly() {
        let client = setup();
        let result = weekly(&client, "AAPL").get().await;
        assert!(result.is_ok(), "Failed to fetch weekly data: {result:?}");
    }

    #[tokio::test]
    #[ignore]
    async fn test_monthly() {
        let client = setup();
        let result = monthly(&client, "AAPL").get().await;
        assert!(result.is_ok(), "Failed to fetch monthly data: {result:?}");
    }
}
