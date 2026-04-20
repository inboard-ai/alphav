//! Aon plc vs. S&P 500 — 12-month comparison.
//!
//! Pulls 12 months of monthly closes for Aon plc (`AON`, NYSE) and the S&P 500
//! index (`SPX` via Alpha Vantage's `INDEX_DATA` endpoint), in parallel, and
//! prints a side-by-side comparison with 12-month returns.
//!
//! Alpha Vantage does not offer any historical bulk endpoint: every historical
//! time series call is one symbol at a time. Parallel fetching via
//! `tokio::try_join!` is the idiomatic way to get multi-symbol history.
//!
//! Run with:
//! ```sh
//! cargo run --example aon_vs_sp500
//! ```

use alphav::{AlphaVantage, Response, rest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let api_key =
        std::env::var("ALPHAVANTAGE_API_KEY").expect("ALPHAVANTAGE_API_KEY must be set in .env or environment");

    let client = AlphaVantage::default().with_key(api_key);

    const AON: &str = "AON";
    const SPX: &str = "SPX";

    println!("\n{:═<72}", "");
    println!("AON vs S&P 500 — 12-month monthly close comparison");
    println!("{:═<72}\n", "");

    println!("Fetching {AON} (TIME_SERIES_MONTHLY) and {SPX} (INDEX_DATA) in parallel...");
    let (aon_raw, spx_raw) = tokio::try_join!(
        rest::time_series::monthly(&client, AON).get(),
        fetch_index_monthly(&client, SPX),
    )?;

    let aon_closes = time_series_closes(&aon_raw)?;
    let spx_closes = index_data_closes(&spx_raw)?;

    println!("\n{:─<72}", "");
    println!("Monthly closes (last 12 months)");
    println!("{:─<72}", "");
    println!("{:<12} {:>16} {:>16}", "Month", "AON (USD)", "S&P 500");
    println!("{:─<72}", "");

    for (month, aon_px) in &aon_closes {
        let spx_px = spx_closes.iter().find(|(m, _)| m == month).map(|(_, p)| *p);
        match spx_px {
            Some(spx) => println!("{month:<12} {aon_px:>16.2} {spx:>16.2}"),
            None => println!("{month:<12} {aon_px:>16.2} {:>16}", "—"),
        }
    }

    if let (Some((_, aon_start)), Some((_, aon_end))) = (aon_closes.last(), aon_closes.first()) {
        let aon_return = (aon_end / aon_start - 1.0) * 100.0;
        let spx_return = match (spx_closes.last(), spx_closes.first()) {
            (Some((_, s)), Some((_, e))) => (e / s - 1.0) * 100.0,
            _ => 0.0,
        };
        println!("\n{:─<72}", "");
        println!("{:<12} {:>16} {:>16}", "12m return", "AON", "S&P 500");
        println!("{:<12} {:>15.2}% {:>15.2}%", "", aon_return, spx_return);
    }

    println!("\n{:═<72}\n", "");
    Ok(())
}

/// Hit `INDEX_DATA` directly via the underlying HTTP client. This endpoint
/// isn't wrapped by a first-class SDK builder yet; we call it as a raw URL,
/// matching the pattern used in `examples/market_summary.rs`.
async fn fetch_index_monthly(client: &AlphaVantage, symbol: &str) -> alphav::Result<String> {
    use alphav::request::Request as _;
    let api_key = client
        .api_key()
        .ok_or_else(|| alphav::Error::Custom("API key not set".to_string()))?;
    let url = format!(
        "https://www.alphavantage.co/query?function=INDEX_DATA&symbol={symbol}&interval=monthly&apikey={api_key}"
    );
    let response = client.client().get(&url).await?;
    if response.status() != 200 {
        return Err(alphav::Error::ApiError {
            status: response.status(),
            message: response.body().to_owned(),
            request_id: response.request_id().clone(),
        });
    }
    Ok(response.body().to_owned())
}

/// Parse a `TIME_SERIES_MONTHLY` JSON response into the last 12 (month, close)
/// rows, newest-first.
fn time_series_closes(raw: &str) -> Result<Vec<(String, f64)>, Box<dyn std::error::Error>> {
    let v: serde_json::Value = serde_json::from_str(raw)?;
    let series = v
        .get("Monthly Time Series")
        .and_then(|s| s.as_object())
        .ok_or_else(|| format!("unexpected TIME_SERIES_MONTHLY response: {raw}"))?;

    let mut rows: Vec<(String, f64)> = series
        .iter()
        .filter_map(|(date, bar)| {
            let close = bar.get("4. close")?.as_str()?.parse::<f64>().ok()?;
            Some((date.clone(), close))
        })
        .collect();
    rows.sort_by(|a, b| b.0.cmp(&a.0));
    rows.truncate(12);
    Ok(rows)
}

/// Parse an `INDEX_DATA` JSON response into the last 12 (month, close) rows,
/// newest-first. `INDEX_DATA` uses a flat array shape that differs from
/// `TIME_SERIES_MONTHLY`.
fn index_data_closes(raw: &str) -> Result<Vec<(String, f64)>, Box<dyn std::error::Error>> {
    let v: serde_json::Value = serde_json::from_str(raw)?;
    let data = v
        .get("data")
        .and_then(|s| s.as_array())
        .ok_or_else(|| format!("unexpected INDEX_DATA response: {raw}"))?;

    let mut rows: Vec<(String, f64)> = data
        .iter()
        .filter_map(|bar| {
            let date = bar.get("date")?.as_str()?.to_string();
            let close = bar.get("close")?.as_str()?.parse::<f64>().ok()?;
            Some((date, close))
        })
        .collect();
    rows.sort_by(|a, b| b.0.cmp(&a.0));
    rows.truncate(12);
    Ok(rows)
}
