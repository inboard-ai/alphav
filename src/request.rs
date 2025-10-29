//! HTTP request trait and request parameter types

use crate::error::Result;
use crate::response::Response;

use std::future::Future;

pub mod common;
pub mod fundamentals;
pub mod time_series;

/// Trait for HTTP clients that can make requests to the Alpha Vantage API.
///
/// Implement this trait to use custom HTTP clients with the Alpha Vantage client.
pub trait Request: Send + Sync {
    /// Associated response type
    type Response: Response;

    /// Create a new instance of the HTTP client
    fn new() -> Self
    where
        Self: Sized;

    /// Make an HTTP GET request to the given URL
    fn get(&self, url: &str) -> impl Future<Output = Result<Self::Response>> + Send;

    /// Make an HTTP POST request to the given URL with a JSON body
    fn post(&self, url: &str, body: &str) -> impl Future<Output = Result<Self::Response>> + Send;
}

/// HTTP response implementation
pub struct HttpResponse {
    status: u16,
    body: String,
    request_id: Option<String>,
}

impl Response for HttpResponse {
    fn status(&self) -> u16 {
        self.status
    }

    fn body(&self) -> &str {
        &self.body
    }

    fn request_id(&self) -> &Option<String> {
        &self.request_id
    }
}

#[cfg(feature = "reqwest")]
impl Request for reqwest::Client {
    type Response = HttpResponse;

    fn new() -> Self {
        reqwest::Client::new()
    }

    async fn get(&self, url: &str) -> Result<Self::Response> {
        let response = self.get(url).send().await?;
        let status = response.status().as_u16();
        let request_id = response
            .headers()
            .get("X-Request-Id")
            .and_then(|h| h.to_str().ok().map(|s| s.to_string()));
        let body = response.text().await?;
        Ok(HttpResponse {
            status,
            body,
            request_id,
        })
    }

    async fn post(&self, url: &str, body: &str) -> Result<Self::Response> {
        let response = self.post(url).body(body.to_string()).send().await?;
        let status = response.status().as_u16();
        let request_id = response
            .headers()
            .get("X-Request-Id")
            .and_then(|h| h.to_str().ok().map(|s| s.to_string()));
        let body = response.text().await?;
        Ok(HttpResponse {
            status,
            body,
            request_id,
        })
    }
}

#[cfg(feature = "hyper")]
/// Hyper client wrapper
#[derive(Clone)]
pub struct HyperClient {
    client: std::sync::Arc<
        hyper_util::client::legacy::Client<
            hyper_tls::HttpsConnector<hyper_util::client::legacy::connect::HttpConnector>,
            http_body_util::Full<hyper::body::Bytes>,
        >,
    >,
}

#[cfg(feature = "hyper")]
impl Request for HyperClient {
    type Response = HttpResponse;

    fn new() -> Self {
        let https = hyper_tls::HttpsConnector::new();
        let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new()).build(https);
        Self {
            client: std::sync::Arc::new(client),
        }
    }

    async fn get(&self, url: &str) -> Result<Self::Response> {
        use http_body_util::BodyExt;

        let uri: hyper::Uri = url
            .parse()
            .map_err(|e| crate::error::Error::Custom(format!("Invalid URL: {e}")))?;

        let response = self
            .client
            .get(uri)
            .await
            .map_err(|e| crate::error::Error::Custom(format!("HTTP request failed: {e}")))?;

        let status = response.status().as_u16();
        let request_id = response
            .headers()
            .get("X-Request-Id")
            .and_then(|h| h.to_str().ok().map(|s| s.to_string()));

        let body_bytes = response
            .into_body()
            .collect()
            .await
            .map_err(|e| crate::error::Error::Custom(format!("Failed to read response body: {e}")))?
            .to_bytes();

        let body = String::from_utf8(body_bytes.to_vec())
            .map_err(|e| crate::error::Error::Custom(format!("Invalid UTF-8 in response: {e}")))?;

        Ok(HttpResponse {
            status,
            body,
            request_id,
        })
    }

    async fn post(&self, url: &str, body_str: &str) -> Result<Self::Response> {
        use http_body_util::BodyExt;

        let uri: hyper::Uri = url
            .parse()
            .map_err(|e| crate::error::Error::Custom(format!("Invalid URL: {e}")))?;

        let req = hyper::Request::builder()
            .method(hyper::Method::POST)
            .uri(uri)
            .header("content-type", "application/json")
            .body(http_body_util::Full::new(hyper::body::Bytes::from(
                body_str.to_string(),
            )))
            .map_err(|e| crate::error::Error::Custom(format!("Failed to build request: {e}")))?;

        let response = self
            .client
            .request(req)
            .await
            .map_err(|e| crate::error::Error::Custom(format!("HTTP request failed: {e}")))?;

        let status = response.status().as_u16();
        let request_id = response
            .headers()
            .get("X-Request-Id")
            .and_then(|h| h.to_str().ok().map(|s| s.to_string()));

        let body_bytes = response
            .into_body()
            .collect()
            .await
            .map_err(|e| crate::error::Error::Custom(format!("Failed to read response body: {e}")))?
            .to_bytes();

        let body = String::from_utf8(body_bytes.to_vec())
            .map_err(|e| crate::error::Error::Custom(format!("Invalid UTF-8 in response: {e}")))?;

        Ok(HttpResponse {
            status,
            body,
            request_id,
        })
    }
}
