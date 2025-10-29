//! REST API endpoints for Alpha Vantage
pub mod raw;

#[cfg(feature = "decoder")]
pub mod decoded;

#[cfg(feature = "table")]
pub mod table;

// Re-export raw module for convenience.
pub use raw::*;
