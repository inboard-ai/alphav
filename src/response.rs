//! Response types for Alpha Vantage API

/// Trait for HTTP response objects
pub trait Response {
    /// Get the HTTP status code
    fn status(&self) -> u16;

    /// Get the response body as a string
    fn body(&self) -> &str;

    /// The ID of the corresponding request
    fn request_id(&self) -> &Option<String>;
}
