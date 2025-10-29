# alphav

Rust client library for the [Alpha Vantage API](https://www.alphavantage.co/).

## Features

- **Lightweight HTTP client** - Uses `hyper` by default (or optionally `reqwest`)
- **Type-safe API** - Strongly typed request builders with compile-time guarantees
- **Flexible output** - Raw JSON, decoded structs, or Polars DataFrames
- **Async/await** - Built on `tokio` for high-performance async I/O
- **Modular** - Optional features let you include only what you need

## Quick Start

```rust
use alphav::{AlphaVantage, rest};
use alphav::request::common::Interval;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AlphaVantage::default().with_key("your_api_key");
    
    // Get intraday time series
    let json = rest::time_series::intraday(&client, "AAPL", Interval::FiveMin)
        .get()
        .await?;
    
    println!("{}", json);
    Ok(())
}
```

## Fundamental Data

The library has comprehensive support for fundamental data, with particular focus on earnings estimates:

```rust
// Get earnings estimates
let estimates = rest::fundamentals::earnings_estimates(&client, "AAPL")
    .horizon("3month")
    .get()
    .await?;

// Get company overview
let overview = rest::fundamentals::company_overview(&client, "AAPL")
    .get()
    .await?;

// Get actual earnings
let earnings = rest::fundamentals::earnings(&client, "AAPL")
    .get()
    .await?;

// Financial statements
let income = rest::fundamentals::income_statement(&client, "AAPL").get().await?;
let balance = rest::fundamentals::balance_sheet(&client, "AAPL").get().await?;
let cashflow = rest::fundamentals::cash_flow(&client, "AAPL").get().await?;
```

## Time Series Data

```rust
use alphav::request::common::{Interval, OutputSize};

// Intraday data (1min, 5min, 15min, 30min, 60min intervals)
let intraday = rest::time_series::intraday(&client, "AAPL", Interval::FiveMin)
    .outputsize(OutputSize::Compact)
    .get()
    .await?;

// Daily data
let daily = rest::time_series::daily(&client, "AAPL")
    .outputsize(OutputSize::Full)
    .get()
    .await?;

// Weekly and monthly data
let weekly = rest::time_series::weekly(&client, "AAPL").get().await?;
let monthly = rest::time_series::monthly(&client, "AAPL").get().await?;
```

## Setup

1. Get a free API key from [Alpha Vantage](https://www.alphavantage.co/support/#api-key)

2. Set your API key:

```bash
# Copy the example env file
cp .env.example .env

# Edit .env and add your key
ALPHAVANTAGE_API_KEY=your_api_key_here
```

3. Add to your `Cargo.toml`:

```toml
[dependencies]
alphav = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Features

- **`hyper`** (default) - Uses `hyper` as the HTTP client (lightweight)
- **`reqwest`** - Alternative HTTP client with more features
- **`decoder`** (default) - Enables typed response decoding
- **`dotenvy`** - Load API keys from `.env` files
- **`table`** - Polars DataFrame output support

To use `reqwest` instead of `hyper`:

```toml
[dependencies]
alphav = { version = "0.1", default-features = false, features = ["reqwest", "decoder"] }
```

## Testing

Run the integration tests (note: these make real API calls):

```bash
# Make sure ALPHAVANTAGE_API_KEY is set
source ~/.alpha  # or set ALPHAVANTAGE_API_KEY in .env

# Run ignored integration tests
cargo test --test integration_tests -- --ignored --test-threads=1
```

**Important:** Integration tests are marked `#[ignore]` to prevent accidental API quota usage. Run them explicitly when needed.

## Examples

### Market Summary

Get a comprehensive market summary for a stock including fundamentals, current price, and earnings estimates:

```bash
cargo run --example market_summary
```

This example fetches:
- Company overview (name, sector, industry)
- Current price and change
- Market cap and shares outstanding
- Key metrics (P/E, PEG, profit margin, ROE)
- Revenue and EBITDA
- Dividend yield
- Earnings estimates

Example output:
```
📊 Fetching market summary for IBM...

🏢 Fetching company overview...
💰 Fetching current price...
📈 Fetching earnings estimates...

════════════════════════════════════════════════════════════
📊 MARKET SUMMARY: IBM
════════════════════════════════════════════════════════════

Company: International Business Machines Corporation
Sector: TECHNOLOGY
Industry: COMPUTER & OFFICE EQUIPMENT

────────────────────────────────────────────────────────────
💰 Current Market Data
────────────────────────────────────────────────────────────

Current Price: $235.84
Change: 🔺 $2.15 (0.92%)
Market Cap: $216.45B
Shares Outstanding: 917.36M
```

## Design Philosophy

This library follows the same architectural patterns as the [polygon](https://github.com/inboard-ai/polygon) crate:

- **Progressive disclosure** - Start simple, add complexity as needed
- **Type-driven** - Compile-time correctness over runtime checks
- **Zero-cost abstractions** - No performance overhead for convenience
- **Modular features** - Include only what you need

## License

MIT
