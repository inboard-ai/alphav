//! Fundamental data endpoint implementations returning raw JSON strings

use crate::client::AlphaVantage;
use crate::processor::Raw;
use crate::request::Request;
use crate::request::fundamentals::{
    BalanceSheet, CashFlow, CompanyOverview, Earnings, EarningsEstimates, IncomeStatement,
};

/// Get earnings estimates for a stock
///
/// Returns a request builder that will return results as raw JSON string.
///
/// # Example
/// ```no_run
/// # use alphav::AlphaVantage;
/// # use alphav::execute::Execute;
/// # async fn example() {
/// # let client = AlphaVantage::default().with_key("api-key");
/// let json = alphav::rest::fundamentals::earnings_estimates(&client, "AAPL")
///     .horizon("3month")
///     .get()
///     .await
///     .unwrap();
/// # }
/// ```
pub fn earnings_estimates<'a, Client: Request>(
    client: &'a AlphaVantage<Client>,
    symbol: impl Into<String>,
) -> EarningsEstimates<'a, Client, Raw> {
    EarningsEstimates::new(client, symbol)
}

/// Get earnings data for a stock
pub fn earnings<'a, Client: Request>(
    client: &'a AlphaVantage<Client>,
    symbol: impl Into<String>,
) -> Earnings<'a, Client, Raw> {
    Earnings::new(client, symbol)
}

/// Get company overview for a stock
pub fn company_overview<'a, Client: Request>(
    client: &'a AlphaVantage<Client>,
    symbol: impl Into<String>,
) -> CompanyOverview<'a, Client, Raw> {
    CompanyOverview::new(client, symbol)
}

/// Get income statement for a stock
pub fn income_statement<'a, Client: Request>(
    client: &'a AlphaVantage<Client>,
    symbol: impl Into<String>,
) -> IncomeStatement<'a, Client, Raw> {
    IncomeStatement::new(client, symbol)
}

/// Get balance sheet for a stock
pub fn balance_sheet<'a, Client: Request>(
    client: &'a AlphaVantage<Client>,
    symbol: impl Into<String>,
) -> BalanceSheet<'a, Client, Raw> {
    BalanceSheet::new(client, symbol)
}

/// Get cash flow statement for a stock
pub fn cash_flow<'a, Client: Request>(
    client: &'a AlphaVantage<Client>,
    symbol: impl Into<String>,
) -> CashFlow<'a, Client, Raw> {
    CashFlow::new(client, symbol)
}

#[cfg(all(test, feature = "dotenvy"))]
mod tests {
    use super::*;

    fn setup() -> AlphaVantage<reqwest::Client> {
        AlphaVantage::new().expect("Failed to create client. Make sure ALPHAVANTAGE_API_KEY is set in .env file")
    }

    #[tokio::test]
    #[ignore]
    async fn test_earnings_estimates() {
        let client = setup();
        let result = earnings_estimates(&client, "AAPL").get().await;
        assert!(result.is_ok(), "Failed to fetch earnings estimates: {result:?}");
    }

    #[tokio::test]
    #[ignore]
    async fn test_company_overview() {
        let client = setup();
        let result = company_overview(&client, "AAPL").get().await;
        assert!(result.is_ok(), "Failed to fetch company overview: {result:?}");
    }
}
