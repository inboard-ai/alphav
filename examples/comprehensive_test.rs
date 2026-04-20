//! Comprehensive test of all tool_use endpoints
use alphav::AlphaVantage;
use alphav::tool_use::{ToolResult, call_tool, list_tools};
use serde_json::json;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check for API key
    let api_key = match std::env::var("ALPHAVANTAGE_API_KEY") {
        Ok(key) => key,
        Err(_) => {
            println!("❌ No API key found. Set ALPHAVANTAGE_API_KEY environment variable to run tests.");
            return Ok(());
        }
    };

    let client = AlphaVantage::default().with_key(api_key);

    println!("🚀 Running comprehensive test of all Alpha Vantage endpoints\n");
    println!("Available tools:");
    for tool in list_tools() {
        println!("  - {} ({}): {}", tool.id, tool.name, tool.description);
    }
    println!();

    // Define test cases for each endpoint
    let test_cases = vec![
        // Time Series Endpoints
        (
            "time_series_intraday",
            json!({
                "tool": "time_series_intraday",
                "params": {
                    "symbol": "AAPL",
                    "interval": "5min",
                    "outputsize": "compact"
                }
            }),
        ),
        (
            "time_series_daily",
            json!({
                "tool": "time_series_daily",
                "params": {
                    "symbol": "AAPL",
                    "outputsize": "compact"
                }
            }),
        ),
        (
            "time_series_weekly",
            json!({
                "tool": "time_series_weekly",
                "params": {
                    "symbol": "AAPL"
                }
            }),
        ),
        (
            "time_series_monthly",
            json!({
                "tool": "time_series_monthly",
                "params": {
                    "symbol": "AAPL"
                }
            }),
        ),
        // Fundamental Data Endpoints
        (
            "company_overview",
            json!({
                "tool": "company_overview",
                "params": {
                    "symbol": "AAPL"
                }
            }),
        ),
        (
            "earnings",
            json!({
                "tool": "earnings",
                "params": {
                    "symbol": "AAPL"
                }
            }),
        ),
        (
            "earnings_estimates",
            json!({
                "tool": "earnings_estimates",
                "params": {
                    "symbol": "AAPL"
                }
            }),
        ),
        (
            "earnings_estimates_with_horizon",
            json!({
                "tool": "earnings_estimates",
                "params": {
                    "symbol": "AAPL",
                    "horizon": "3month"
                }
            }),
        ),
        (
            "income_statement",
            json!({
                "tool": "income_statement",
                "params": {
                    "symbol": "AAPL"
                }
            }),
        ),
        (
            "balance_sheet",
            json!({
                "tool": "balance_sheet",
                "params": {
                    "symbol": "AAPL"
                }
            }),
        ),
        (
            "cash_flow",
            json!({
                "tool": "cash_flow",
                "params": {
                    "symbol": "AAPL"
                }
            }),
        ),
    ];

    let mut results = HashMap::new();
    let mut successful = 0;
    let mut failed = 0;

    for (test_name, request) in test_cases {
        println!("📊 Testing: {}", test_name);
        println!("   Request: {}", serde_json::to_string_pretty(&request)?);

        match call_tool(&client, request).await {
            Ok(ToolResult::DataFrame(df_out)) => {
                println!("   ✅ Success!");

                if !df_out.schema.is_empty() {
                    println!("   📋 Schema ({} columns):", df_out.schema.len());
                    for col in &df_out.schema {
                        println!("     - {} as {} ({})", col.name, col.alias, col.dtype);
                    }
                } else {
                    println!("   📋 Schema: Empty (raw JSON response)");
                }

                if let Some(meta) = &df_out.metadata {
                    println!("   📝 Metadata: {}", serde_json::to_string_pretty(meta)?);
                }

                let data = df_out.data.clone();
                let schema_was_empty = df_out.schema.is_empty();
                match df_out.to_dataframe() {
                    Ok(df) => {
                        println!("   📊 DataFrame conversion: ✅ Success");
                        println!("   📏 DataFrame shape: {} rows × {} columns", df.height(), df.width());

                        if data.is_array() && !schema_was_empty {
                            println!("   🔍 Sample data (first 3 rows):");
                            println!("{}", df.head(Some(3)));
                        } else {
                            println!("   🔍 Data type: Raw JSON object");
                            if let Some(obj) = data.as_object() {
                                println!(
                                    "   📊 Top-level keys: {}",
                                    obj.keys().take(10).cloned().collect::<Vec<_>>().join(", ")
                                );
                            }
                        }

                        results.insert(test_name.to_string(), "✅ Success".to_string());
                        successful += 1;
                    }
                    Err(e) => {
                        println!("   ❌ DataFrame conversion failed: {}", e);
                        results.insert(test_name.to_string(), format!("❌ DataFrame error: {}", e));
                        failed += 1;
                    }
                }
            }
            Ok(ToolResult::Text(text)) => {
                println!("   📄 Text response: {}", text.content);
                results.insert(test_name.to_string(), "✅ Text response".to_string());
                successful += 1;
            }
            Ok(other) => {
                println!("   ⚠️  Unknown variant: {other:?}");
                results.insert(test_name.to_string(), "⚠️ Unknown variant".to_string());
                failed += 1;
            }
            Err(e) => {
                println!("   ❌ Error: {}", e);
                results.insert(test_name.to_string(), format!("❌ Error: {}", e));
                failed += 1;
            }
        }

        println!("   ⏱️  Waiting 1 second before next test...\n");
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    // Print summary
    println!("📈 Test Summary");
    println!("===============");
    println!("✅ Successful: {}", successful);
    println!("❌ Failed: {}", failed);
    println!("📊 Total: {}", successful + failed);
    println!();

    println!("📋 Detailed Results:");
    for (test_name, result) in results {
        println!("  {} -> {}", test_name, result);
    }

    if failed > 0 {
        println!("\n⚠️  Some tests failed. Check the output above for details.");
    } else {
        println!("\n🎉 All tests passed!");
    }

    Ok(())
}
