//! Main Alpha Vantage API client
use crate::request::Request;

/// The main Alpha Vantage API client.
///
/// When the `reqwest` feature is enabled, this uses `reqwest::Client` as the default HTTP client.
/// When the `hyper` feature is enabled, this uses `HyperClient` as the default HTTP client.
/// Otherwise, you must provide your own HTTP client that implements [`Request`].
#[cfg(feature = "reqwest")]
#[derive(Debug, Clone)]
pub struct AlphaVantage<Client: Request = reqwest::Client> {
    client: Client,
    api_key: Option<String>,
}

/// The main Alpha Vantage API client.
///
/// When the `reqwest` feature is enabled, this uses `reqwest::Client` as the default HTTP client.
/// When the `hyper` feature is enabled, this uses `HyperClient` as the default HTTP client.
/// Otherwise, you must provide your own HTTP client that implements [`Request`].
#[cfg(all(feature = "hyper", not(feature = "reqwest")))]
#[derive(Debug, Clone)]
pub struct AlphaVantage<Client: Request = crate::request::HyperClient> {
    client: Client,
    api_key: Option<String>,
}

/// The main Alpha Vantage API client.
///
/// When the `reqwest` feature is enabled, this uses `reqwest::Client` as the default HTTP client.
/// When the `hyper` feature is enabled, this uses `HyperClient` as the default HTTP client.
/// Otherwise, you must provide your own HTTP client that implements [`Request`].
#[cfg(not(any(feature = "reqwest", feature = "hyper")))]
#[derive(Debug, Clone)]
pub struct AlphaVantage<Client: Request> {
    client: Client,
    api_key: Option<String>,
}

// Implementation for any Client that implements Request
impl<Client: Request> AlphaVantage<Client> {
    /// Create a new Alpha Vantage client using the default HTTP client.
    ///
    /// This method is only available when the `dotenvy` feature is enabled.
    /// It loads the API key from the `ALPHAVANTAGE_API_KEY` environment variable using dotenvy.
    ///
    /// # Errors
    ///
    /// Returns an error if the environment variable cannot be loaded or if the API key is missing.
    #[cfg(feature = "dotenvy")]
    pub fn new() -> crate::Result<Self> {
        dotenvy::dotenv().ok(); // Try to load .env file, ignore errors

        let api_key = std::env::var("ALPHAVANTAGE_API_KEY").map_err(|_| crate::Error::MissingApiKey)?;

        Ok(Self {
            client: Client::new(),
            api_key: Some(api_key),
        })
    }

    #[cfg(not(feature = "dotenvy"))]
    /// Create a new Alpha Vantage client with the default HTTP client.
    ///
    /// You must manually set the API key using [`with_key`](Self::with_key).
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            api_key: None,
        }
    }

    /// Sets the HTTP client for this instance.
    pub fn with_client(mut self, client: Client) -> Self {
        self.client = client;
        self
    }

    /// Set the API key for this instance.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use alphav::AlphaVantage;
    ///
    /// let client = AlphaVantage::default().with_key("my_api_key");
    /// ```
    pub fn with_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Get the API key for this instance.
    pub fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    /// Get a reference to the underlying HTTP client.
    pub fn client(&self) -> &Client {
        &self.client
    }
}

// Default implementation for reqwest
#[cfg(feature = "reqwest")]
impl Default for AlphaVantage<reqwest::Client> {
    fn default() -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key: None,
        }
    }
}

// Default implementation for hyper
#[cfg(all(feature = "hyper", not(feature = "reqwest")))]
impl Default for AlphaVantage<crate::request::HyperClient> {
    fn default() -> Self {
        Self {
            client: crate::request::HyperClient::new(),
            api_key: None,
        }
    }
}

// Default implementation when no HTTP client feature is enabled
#[cfg(not(any(feature = "reqwest", feature = "hyper")))]
impl<Client: Request> Default for AlphaVantage<Client> {
    /// Create a default Alpha Vantage client with no API key set.
    ///
    /// You must call [`with_key`](Self::with_key) to set the API key before making requests.
    fn default() -> Self {
        Self {
            client: Client::new(),
            api_key: None,
        }
    }
}
