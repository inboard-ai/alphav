//! Execute trait for running API requests
//!
//! The `Execute` trait provides the `.get()` method used by all endpoint builders
//! to execute requests and return results.

use crate::error::Result;

/// Trait for executing API requests
///
/// Implemented by all endpoint request builders.
/// Provides the `.get()` method to execute the request and return the result.
pub trait Execute {
    /// The output type of the request
    type Output;

    /// Execute the request and return the result
    fn get(self) -> impl std::future::Future<Output = Result<Self::Output>>;
}
