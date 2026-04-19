//! Aon plc vs. S&P 500 — 12-month comparison.
//!
//! Demonstrates:
//!   1. The new `REALTIME_BULK_QUOTES` endpoint, which fetches snapshots for
//!      multiple symbols in a single API call (up to 100 per request).
//!   2. Fetching 12 months of historical monthly close prices for each symbol
//!      in parallel (historical data is not available via the bulk endpoint).
//!
//! `AON` is the NYSE ticker for Aon plc. `SPY` (the SPDR S&P 500 ETF) is used
//! as a proxy for the S&P 500 — the direct `SPX` index feed is premium-only.
//!
//! Run with:
//! ```sh
//! cargo run --example aon_vs_sp500
//! ```
//!
//! Note: `REALTIME_BULK_QUOTES` requires a premium Alpha Vantage subscription.
//! On the free tier, the bulk section will show the premium notice and the
//! historical table will still render.

use alphav::{AlphaVantage, rest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let api_key =
        std::env::var("ALPHAVANTAGE_API_KEY").expect("ALPHAVANTAGE_API_KEY must be set in .env or environment");

    let client = AlphaVantage::default().with_key(api_key);

    const AON: &str = "AON";
    const SPY: &str = "SPY";

    println!("\n{:═<72}", "");
    println!("AON vs S&P 500 (via SPY)");
    println!("{:═<72}\n", "");

    // 1. Realtime snapshot for both symbols in a single bulk call.
    println!("Fetching realtime bulk quotes for {AON}, {SPY}...");
    let bulk_json = rest::quotes::realtime_bulk(&client, [AON, SPY]).get().await?;
    let bulk: serde_json::Value = serde_json::from_str(&bulk_json)?;

    println!("\n{:─<72}", "");
    println!("Realtime snapshot (REALTIME_BULK_QUOTES)");
    println!("{:─<72}", "");

    // Alpha Vantage returns HTTP 200 with a `message` field and illustrative
    // sample data if your plan lacks the Realtime US Market Data add-on, so
    // we always surface that message before printing any numbers.
    let sample_warning = bulk
        .get("message")
        .and_then(|v| v.as_str())
        .filter(|m| m.to_ascii_uppercase().contains("SAMPLE"));
    if let Some(msg) = sample_warning {
        println!("  WARNING: Alpha Vantage returned illustrative sample data.");
        println!("  {msg}");
        println!();
    }

    if let Some(quotes) = bulk.get("data").and_then(|v| v.as_array()) {
        let label = if sample_warning.is_some() { " (sample)" } else { "" };
        println!("{:<8} {:>14} {:>14} {:>14}", "Symbol", "Price", "Change", "Change %");
        for q in quotes {
            let sym = q.get("symbol").and_then(|v| v.as_str()).unwrap_or("?");
            let price = q.get("close").and_then(|v| v.as_str()).unwrap_or("—");
            let change = q.get("change").and_then(|v| v.as_str()).unwrap_or("—");
            let change_pct = q.get("change_percent").and_then(|v| v.as_str()).unwrap_or("—");
            println!("{sym:<8}{label} {price:>14} {change:>14} {change_pct:>14}");
        }
    } else {
        let note = bulk
            .get("Information")
            .or_else(|| bulk.get("Note"))
            .and_then(|v| v.as_str())
            .unwrap_or("bulk quotes endpoint returned no data (premium required?)");
        println!("  {note}");
    }

    // 2. 12 months of monthly closes for each symbol, fetched in parallel.
    println!("\nFetching 12 months of monthly closes for {AON} and {SPY}...");
    let (aon_raw, spy_raw) = tokio::try_join!(
        rest::time_series::monthly(&client, AON).get(),
        rest::time_series::monthly(&client, SPY).get(),
    )?;

    let aon_closes = monthly_closes(&aon_raw)?;
    let spy_closes = monthly_closes(&spy_raw)?;

    println!("\n{:─<72}", "");
    println!("Monthly closes (last 12 months)");
    println!("{:─<72}", "");
    println!("{:<12} {:>16} {:>16} {:>16}", "Month", "AON (USD)", "SPY (USD)", "Ratio");
    println!("{:─<72}", "");

    for (month, aon_px) in &aon_closes {
        let spy_px = spy_closes.iter().find(|(m, _)| m == month).map(|(_, p)| *p);
        match spy_px {
            Some(spy) => {
                let ratio = if spy > 0.0 { aon_px / spy } else { 0.0 };
                println!("{month:<12} {aon_px:>16.2} {spy:>16.2} {ratio:>16.3}");
            }
            None => {
                println!("{month:<12} {aon_px:>16.2} {:>16} {:>16}", "—", "—");
            }
        }
    }

    if let (Some((_, aon_start)), Some((_, aon_end))) = (aon_closes.last(), aon_closes.first()) {
        let aon_return = (aon_end / aon_start - 1.0) * 100.0;
        let spy_return = match (spy_closes.last(), spy_closes.first()) {
            (Some((_, s)), Some((_, e))) => (e / s - 1.0) * 100.0,
            _ => 0.0,
        };
        println!("\n{:─<72}", "");
        println!("{:<12} {:>16} {:>16}", "12m return", "AON", "SPY");
        println!("{:<12} {:>15.2}% {:>15.2}%", "", aon_return, spy_return);
    }

    println!("\n{:═<72}\n", "");
    Ok(())
}

/// Parse a `TIME_SERIES_MONTHLY` JSON response and return the last 12
/// (month, close) rows ordered newest-first.
fn monthly_closes(raw: &str) -> Result<Vec<(String, f64)>, Box<dyn std::error::Error>> {
    let v: serde_json::Value = serde_json::from_str(raw)?;
    let series = v
        .get("Monthly Time Series")
        .and_then(|s| s.as_object())
        .ok_or_else(|| {
            let note = v
                .get("Information")
                .or_else(|| v.get("Note"))
                .and_then(|n| n.as_str())
                .unwrap_or("missing 'Monthly Time Series' field");
            format!("unexpected monthly response: {note}")
        })?;

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
