//! Market Summary Example
//!
//! This example demonstrates comprehensive financial analysis including:
//! - Historical vs forward earnings
//! - Enterprise value calculation from first principles
//! - Valuation multiples (EV/Revenue, P/E)
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
    let api_key =
        std::env::var("ALPHAVANTAGE_API_KEY").expect("ALPHAVANTAGE_API_KEY must be set in .env or environment");

    let client = AlphaVantage::default().with_key(api_key);
    let symbol = "AAPL";

    // Fetch all required data
    let overview = rest::fundamentals::company_overview(&client, symbol).get().await?;
    let overview_json: serde_json::Value = serde_json::from_str(&overview)?;

    use alphav::request::Request;
    let quote_url = format!(
        "https://www.alphavantage.co/query?function=GLOBAL_QUOTE&symbol={}&apikey={}",
        symbol,
        client.api_key().unwrap()
    );
    let quote_response = client.client().get(&quote_url).await?;
    let quote_json: serde_json::Value = serde_json::from_str(quote_response.body())?;

    // Fetch data as DataFrames for easier manipulation
    let estimates_df = rest::fundamentals::earnings_estimates(&client, symbol)
        .horizon("3month")
        .as_dataframe()
        .get()
        .await?;

    let income_df = rest::fundamentals::income_statement(&client, symbol)
        .as_dataframe()
        .get()
        .await?;

    let balance_df = rest::fundamentals::balance_sheet(&client, symbol)
        .as_dataframe()
        .get()
        .await?;

    // Print formatted market summary
    println!("\n{:═<80}", "");
    println!("FINANCIAL ANALYSIS: {}", symbol);
    println!("{:═<80}\n", "");

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
    println!("Current Market Data");
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
                println!("Change: ${} ({})", change, change_pct);
            }
        }
    }

    // Market Cap & Shares
    if let Some(market_cap) = overview_json.get("MarketCapitalization").and_then(|v| v.as_str()) {
        if let Ok(cap) = market_cap.parse::<f64>() {
            println!("Market Cap: ${:.2}B", cap / 1_000_000_000.0);
        }
    }

    if let Some(shares) = overview_json.get("SharesOutstanding").and_then(|v| v.as_str()) {
        if let Ok(shares_num) = shares.parse::<f64>() {
            println!("Shares Outstanding: {:.2}M", shares_num / 1_000_000.0);
        }
    }

    // Extract financial data for analysis using DataFrames
    println!("\n{:─<80}", "");
    println!("Earnings Analysis");
    println!("{:─<80}\n", "");

    // Calculate shares outstanding for EPS calculations
    let shares_outstanding = overview_json
        .get("SharesOutstanding")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    // First check for "historical fiscal year" in estimates (might be more recent)
    let historical_fy_estimate = if estimates_df.height() == 0 {
        None
    } else {
        let horizon_col = estimates_df.column("horizon")?.str()?;
        let mut hist_fy_row = None;

        for i in 0..estimates_df.height() {
            if let Some(horizon) = horizon_col.get(i) {
                if horizon.contains("historical fiscal year") {
                    hist_fy_row = Some(i);
                    break;
                }
            }
        }

        hist_fy_row.and_then(|row| {
            let date = estimates_df
                .column("date")
                .ok()?
                .str()
                .ok()?
                .get(row)
                .map(|s| s.to_string());
            let eps = estimates_df
                .column("eps_estimate_average")
                .ok()?
                .str()
                .ok()?
                .get(row)
                .and_then(|s| s.parse::<f64>().ok());
            let revenue = estimates_df
                .column("revenue_estimate_average")
                .ok()?
                .str()
                .ok()?
                .get(row)
                .and_then(|s| s.parse::<f64>().ok());

            match (date, eps, revenue) {
                (Some(d), Some(e), Some(r)) => Some((d, e, r)),
                _ => None,
            }
        })
    };

    // Get last fiscal year actual data from income statement (first row)
    let income_fy_data = if income_df.height() > 0 {
        let fiscal_date = income_df
            .column("fiscalDateEnding")?
            .str()?
            .get(0)
            .map(|s| s.to_string());
        let revenue = income_df
            .column("totalRevenue")?
            .str()?
            .get(0)
            .and_then(|s| s.parse::<f64>().ok());
        let net_income = income_df
            .column("netIncome")?
            .str()?
            .get(0)
            .and_then(|s| s.parse::<f64>().ok());

        match (fiscal_date, revenue, net_income) {
            (Some(date), Some(rev), Some(ni)) => Some((date, rev, ni)),
            _ => None,
        }
    } else {
        None
    };

    // Use historical estimate if it's more recent than income statement data
    let last_fy_data = match (historical_fy_estimate.as_ref(), income_fy_data.as_ref()) {
        (Some((est_date, eps, revenue)), Some((inc_date, _, _))) if est_date > inc_date => {
            let net_income = eps * shares_outstanding;
            Some((est_date.clone(), *revenue, net_income))
        }
        (Some((est_date, eps, revenue)), None) => {
            let net_income = eps * shares_outstanding;
            Some((est_date.clone(), *revenue, net_income))
        }
        _ => income_fy_data.clone(),
    };

    // Get next fiscal year estimate - filter for "next fiscal year" horizon
    let next_fy_estimate = if estimates_df.height() == 0 {
        None
    } else {
        let horizon_col = estimates_df.column("horizon")?.str()?;
        let mut next_fy_row = None;

        for i in 0..estimates_df.height() {
            if let Some(horizon) = horizon_col.get(i) {
                if horizon.contains("next fiscal year") {
                    next_fy_row = Some(i);
                    break;
                }
            }
        }

        next_fy_row.and_then(|row| {
            let date = estimates_df
                .column("date")
                .ok()?
                .str()
                .ok()?
                .get(row)
                .map(|s| s.to_string());
            let eps = estimates_df
                .column("eps_estimate_average")
                .ok()?
                .str()
                .ok()?
                .get(row)
                .and_then(|s| s.parse::<f64>().ok());
            let revenue = estimates_df
                .column("revenue_estimate_average")
                .ok()?
                .str()
                .ok()?
                .get(row)
                .and_then(|s| s.parse::<f64>().ok());

            match (date, eps, revenue) {
                (Some(d), Some(e), Some(r)) => Some((d, e, r)),
                _ => None,
            }
        })
    };

    // Prepare data for transposed table (metrics as rows, dates as columns)
    let (fy_actual_year, rev_actual, ni_actual) = if let Some((ref date, revenue, net_income)) = last_fy_data {
        // Check if the date is actually reported (actual) or still an estimate
        // Compare with the last reported income statement date - if our date is later, it's an estimate
        let is_actual = if let Some((ref inc_date, _, _)) = income_fy_data {
            date.as_str() <= inc_date.as_str()
        } else {
            false // No income statement data means it's an estimate
        };
        let suffix = if is_actual { "A" } else { "E" };
        (
            format!("FY{}{}", &date[0..4], suffix),
            revenue / 1_000_000_000.0,
            net_income / 1_000_000_000.0,
        )
    } else {
        ("N/A".to_string(), 0.0, 0.0)
    };

    let (fy_estimate_year, rev_estimate, ni_estimate) = if let Some((ref date, eps, revenue)) = next_fy_estimate {
        let est_net_income = eps * shares_outstanding;
        (
            format!("FY{}E", &date[0..4]),
            revenue / 1_000_000_000.0,
            est_net_income / 1_000_000_000.0,
        )
    } else {
        ("N/A".to_string(), 0.0, 0.0)
    };

    // Print transposed table with metrics as rows
    println!("{:<25} {:>15} {:>15}", "", fy_actual_year, fy_estimate_year);
    println!("{:─<58}", "");
    println!("{:<25} {:>14.2}B {:>14.2}B", "Revenue", rev_actual, rev_estimate);
    println!("{:<25} {:>14.2}B {:>14.2}B", "Net Income", ni_actual, ni_estimate);

    // Enterprise Value Calculation
    println!("\n{:─<80}", "");
    println!("Enterprise Value Calculation");
    println!("{:─<80}\n", "");

    // Get balance sheet data - most recent annual report (first row)
    let (total_cash, total_debt) = if balance_df.height() > 0 {
        let cash = balance_df
            .column("cashAndCashEquivalentsAtCarryingValue")
            .or_else(|_| balance_df.column("cashAndShortTermInvestments"))
            .ok()
            .and_then(|col| col.str().ok())
            .and_then(|s| s.get(0))
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let debt = balance_df
            .column("shortLongTermDebtTotal")
            .or_else(|_| balance_df.column("longTermDebt"))
            .ok()
            .and_then(|col| col.str().ok())
            .and_then(|s| s.get(0))
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        (cash, debt)
    } else {
        (0.0, 0.0)
    };

    let market_cap = overview_json
        .get("MarketCapitalization")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    let net_debt = total_debt - total_cash;
    let enterprise_value = market_cap + net_debt;

    println!("Market Cap:              ${:>12.2}B", market_cap / 1_000_000_000.0);
    println!("Total Cash:              ${:>12.2}B", total_cash / 1_000_000_000.0);
    println!("Total Debt:              ${:>12.2}B", total_debt / 1_000_000_000.0);
    println!("Net Debt:                ${:>12.2}B", net_debt / 1_000_000_000.0);
    println!("{:─<40}", "");
    println!(
        "Enterprise Value:        ${:>12.2}B",
        enterprise_value / 1_000_000_000.0
    );

    // Calculate valuation multiples from first principles
    println!("\n{:─<75}", "");

    // Get current price for P/E calculation
    let current_price = quote_json
        .get("Global Quote")
        .and_then(|q| q.get("05. price"))
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    // Calculate trailing metrics
    let (trailing_fy, trailing_ev_rev, trailing_pe) = if let Some((ref date, revenue, net_income)) = last_fy_data {
        let fy_year = &date[0..4];
        let ev_rev = if revenue > 0.0 { enterprise_value / revenue } else { 0.0 };
        let eps = if shares_outstanding > 0.0 {
            net_income / shares_outstanding
        } else {
            0.0
        };
        let pe = if eps > 0.0 { current_price / eps } else { 0.0 };
        (fy_year.to_string(), ev_rev, pe)
    } else {
        ("N/A".to_string(), 0.0, 0.0)
    };

    // Calculate forward metrics
    let (forward_fy, forward_ev_rev, forward_pe) = if let Some((ref date, eps, revenue)) = next_fy_estimate {
        let fy_year = &date[0..4];
        let ev_rev = if revenue > 0.0 { enterprise_value / revenue } else { 0.0 };
        let pe = if eps > 0.0 { current_price / eps } else { 0.0 };
        (fy_year.to_string(), ev_rev, pe)
    } else {
        ("N/A".to_string(), 0.0, 0.0)
    };

    // Print 2x2 table with title and headers on same line
    println!("Valuation Multiples{:>35} {:>15}", fy_actual_year, fy_estimate_year);
    println!("{:─<75}", "");
    println!(
        "{:<43} {:>14.2}x {:>14.2}x",
        "EV / Revenue", trailing_ev_rev, forward_ev_rev
    );
    println!("{:<43} {:>14.1}x {:>14.1}x", "P / E", trailing_pe, forward_pe);

    println!("\n{:═<75}", "");

    Ok(())
}
