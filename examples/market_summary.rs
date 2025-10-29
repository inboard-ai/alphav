//! Market Summary Example
//!
//! This example demonstrates how to fetch comprehensive market data for a stock
//! including earnings estimates, company fundamentals, and current price.
//!
//! This example requires the `table` feature to be enabled.
//!
//! Run with:
//! ```sh
//! cargo run --example market_summary --features=table
//! ```

use alphav::{AlphaVantage, Response, rest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load API key from environment
    dotenvy::dotenv().ok();
    let api_key = std::env::var("ALPHAVANTAGE_API_KEY")
        .expect("ALPHAVANTAGE_API_KEY must be set in .env or environment");

    let client = AlphaVantage::default().with_key(api_key);
    let symbol = "IBM";

    println!("📊 Fetching market summary for {}...", symbol);

    // Fetch company overview (includes market cap, shares outstanding, fundamentals)
    println!("🏢 Fetching company overview...");
    let overview = rest::fundamentals::company_overview(&client, symbol)
        .get()
        .await?;

    let overview_json: serde_json::Value = serde_json::from_str(&overview)?;

    // Fetch current quote
    println!("💰 Fetching current price...");
    let quote_url = format!(
        "https://www.alphavantage.co/query?function=GLOBAL_QUOTE&symbol={}&apikey={}",
        symbol,
        client.api_key().unwrap()
    );

    use alphav::request::Request;
    let quote_response = client.client().get(&quote_url).await?;
    let quote_json: serde_json::Value = serde_json::from_str(quote_response.body())?;

    // Fetch earnings estimates as DataFrame
    println!("📈 Fetching earnings estimates...");
    let estimates_df = rest::fundamentals::earnings_estimates(&client, symbol)
        .horizon("3month")
        .as_dataframe()
        .get()
        .await?;

    // Print formatted market summary
    println!("\n{:═<60}", "");
    println!("📊 MARKET SUMMARY: {}", symbol);
    println!("{:═<60}\n", "");

    // Company Information
    if let Some(name) = overview_json.get("Name").and_then(|v| v.as_str()) {
        println!("Company: {}", name);
    }
    if let Some(sector) = overview_json.get("Sector").and_then(|v| v.as_str()) {
        println!("Sector: {}", sector);
    }
    if let Some(industry) = overview_json.get("Industry").and_then(|v| v.as_str()) {
        println!("Industry: {}", industry);
    }

    println!("\n{:─<60}", "");
    println!("💰 Current Market Data");
    println!("{:─<60}\n", "");

    // Current Price
    if let Some(price) = quote_json
        .get("Global Quote")
        .and_then(|q| q.get("05. price"))
        .and_then(|v| v.as_str())
    {
        println!("Current Price: ${}", price);

        if let Some(change) = quote_json
            .get("Global Quote")
            .and_then(|q| q.get("09. change"))
            .and_then(|v| v.as_str())
        {
            if let Some(change_pct) = quote_json
                .get("Global Quote")
                .and_then(|q| q.get("10. change percent"))
                .and_then(|v| v.as_str())
            {
                let arrow = if change.starts_with('-') {
                    "🔻"
                } else {
                    "🔺"
                };
                println!("Change: {} ${} ({})", arrow, change, change_pct);
            }
        }
    }

    // Market Cap & Shares
    if let Some(market_cap) = overview_json
        .get("MarketCapitalization")
        .and_then(|v| v.as_str())
    {
        if let Ok(cap) = market_cap.parse::<f64>() {
            println!("Market Cap: ${:.2}B", cap / 1_000_000_000.0);
        }
    }

    if let Some(shares) = overview_json
        .get("SharesOutstanding")
        .and_then(|v| v.as_str())
    {
        if let Ok(shares_num) = shares.parse::<f64>() {
            println!("Shares Outstanding: {:.2}M", shares_num / 1_000_000.0);
        }
    }

    println!("\n{:─<60}", "");
    println!("📊 Key Metrics");
    println!("{:─<60}\n", "");

    // Valuation Metrics
    if let Some(pe) = overview_json.get("PERatio").and_then(|v| v.as_str()) {
        if pe != "None" && !pe.is_empty() {
            println!("P/E Ratio: {}", pe);
        }
    }

    if let Some(peg) = overview_json.get("PEGRatio").and_then(|v| v.as_str()) {
        if peg != "None" && !peg.is_empty() {
            println!("PEG Ratio: {}", peg);
        }
    }

    if let Some(pb) = overview_json
        .get("PriceToBookRatio")
        .and_then(|v| v.as_str())
    {
        if pb != "None" && !pb.is_empty() {
            println!("P/B Ratio: {}", pb);
        }
    }

    // Profitability
    if let Some(profit_margin) = overview_json.get("ProfitMargin").and_then(|v| v.as_str()) {
        if profit_margin != "None" && !profit_margin.is_empty() {
            if let Ok(margin) = profit_margin.parse::<f64>() {
                println!("Profit Margin: {:.2}%", margin * 100.0);
            }
        }
    }

    if let Some(roe) = overview_json
        .get("ReturnOnEquityTTM")
        .and_then(|v| v.as_str())
    {
        if roe != "None" && !roe.is_empty() {
            if let Ok(roe_val) = roe.parse::<f64>() {
                println!("Return on Equity: {:.2}%", roe_val * 100.0);
            }
        }
    }

    // Revenue & EBITDA
    if let Some(revenue) = overview_json.get("RevenueTTM").and_then(|v| v.as_str()) {
        if let Ok(rev) = revenue.parse::<f64>() {
            println!("Revenue (TTM): ${:.2}B", rev / 1_000_000_000.0);
        }
    }

    if let Some(ebitda) = overview_json.get("EBITDA").and_then(|v| v.as_str()) {
        if let Ok(ebitda_val) = ebitda.parse::<f64>() {
            println!("EBITDA: ${:.2}B", ebitda_val / 1_000_000_000.0);
        }
    }

    // Dividend Info
    if let Some(div_yield) = overview_json.get("DividendYield").and_then(|v| v.as_str()) {
        if div_yield != "None" && !div_yield.is_empty() && div_yield != "0" {
            if let Ok(yield_val) = div_yield.parse::<f64>() {
                println!("Dividend Yield: {:.2}%", yield_val * 100.0);
            }
        }
    }

    // Earnings Estimates
    println!("\n{:─<60}", "");
    println!("📈 Earnings Estimates (DataFrame)");
    println!("{:─<60}\n", "");

    println!("{}", estimates_df);

    println!("\n{:═<60}", "");
    println!("\n✅ Market summary complete!\n");

    Ok(())
}
