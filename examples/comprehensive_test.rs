//! Comprehensive test of all tool_use endpoints
use alphav::AlphaVantage;
use alphav::tool_use::{ToolCallResult, call_tool, list_tools};
use emporium_core::ToolResult;
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
        ("time_series_intraday", json!({
            "tool": "time_series_intraday",
            "params": {
                "symbol": "AAPL",
                "interval": "5min",
                "outputsize": "compact"
            }
        })),
        ("time_series_daily", json!({
            "tool": "time_series_daily",
            "params": {
                "symbol": "AAPL",
                "outputsize": "compact"
            }
        })),
        ("time_series_weekly", json!({
            "tool": "time_series_weekly",
            "params": {
                "symbol": "AAPL"
            }
        })),
        ("time_series_monthly", json!({
            "tool": "time_series_monthly",
            "params": {
                "symbol": "AAPL"
            }
        })),
        
        // Fundamental Data Endpoints
        ("company_overview", json!({
            "tool": "company_overview",
            "params": {
                "symbol": "AAPL"
            }
        })),
        ("earnings", json!({
            "tool": "earnings",
            "params": {
                "symbol": "AAPL"
            }
        })),
        ("earnings_estimates", json!({
            "tool": "earnings_estimates",
            "params": {
                "symbol": "AAPL"
            }
        })),
        ("earnings_estimates_with_horizon", json!({
            "tool": "earnings_estimates",
            "params": {
                "symbol": "AAPL",
                "horizon": "3month"
            }
        })),
        ("income_statement", json!({
            "tool": "income_statement",
            "params": {
                "symbol": "AAPL"
            }
        })),
        ("balance_sheet", json!({
            "tool": "balance_sheet",
            "params": {
                "symbol": "AAPL"
            }
        })),
        ("cash_flow", json!({
            "tool": "cash_flow",
            "params": {
                "symbol": "AAPL"
            }
        })),
    ];

    let mut results = HashMap::new();
    let mut successful = 0;
    let mut failed = 0;

    for (test_name, request) in test_cases {
        println!("📊 Testing: {}", test_name);
        println!("   Request: {}", serde_json::to_string_pretty(&request)?);
        
        match call_tool(&client, request).await {
            Ok(ToolCallResult::DataFrame { data, schema, metadata }) => {
                println!("   ✅ Success!");
                
                // Print schema information
                if !schema.is_empty() {
                    println!("   📋 Schema ({} columns):", schema.len());
                    for col in &schema {
                        println!("     - {} as {} ({})", col.name, col.alias, col.dtype);
                    }
                } else {
                    println!("   📋 Schema: Empty (raw JSON response)");
                }
                
                // Print metadata if available
                if let Some(meta) = &metadata {
                    println!("   📝 Metadata: {}", serde_json::to_string_pretty(meta)?);
                }
                
                // Convert to emporium DataFrame
                let emp = emporium_core::ToolResult::columnar(data.clone(), schema.clone(), metadata.clone());
                match emp {
                    ToolResult::DataFrame(proto) => {
                        match proto.to_dataframe() {
                            Ok(df) => {
                                println!("   📊 DataFrame conversion: ✅ Success");
                                println!("   📏 DataFrame shape: {} rows × {} columns", df.height(), df.width());
                                
                                // Show first few rows for time series data (if it's an array)
                                if data.is_array() && !schema.is_empty() {
                                    println!("   🔍 Sample data (first 3 rows):");
                                    println!("{}", df.head(Some(3)));
                                } else {
                                    // For fundamental data, just show the structure
                                    println!("   🔍 Data type: Raw JSON object");
                                    if let Some(obj) = data.as_object() {
                                        println!("   📊 Top-level keys: {}", obj.keys().take(10).cloned().collect::<Vec<_>>().join(", "));
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
                    ToolResult::Text(text) => {
                        println!("   📄 Text result: {}", text);
                        results.insert(test_name.to_string(), "✅ Text result".to_string());
                        successful += 1;
                    }
                }
            }
            Ok(ToolCallResult::Text(text)) => {
                println!("   📄 Text response: {}", text);
                results.insert(test_name.to_string(), "✅ Text response".to_string());
                successful += 1;
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
