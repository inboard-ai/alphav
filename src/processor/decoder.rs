//! Decoder processor using the decoder crate
use crate::error::Result;
use crate::processor::Processor;
use crate::response::Response;

/// Decoder processor that uses the decoder crate to parse JSON responses
pub struct Decoder<T> {
    decoder_fn: Box<dyn Fn(decoder::Value) -> decoder::Result<T> + Send + Sync>,
}

impl<T> Decoder<T> {
    /// Create a new decoder with the given decoder function
    pub fn new(decoder_fn: impl Fn(decoder::Value) -> decoder::Result<T> + Send + Sync + 'static) -> Self {
        Self {
            decoder_fn: Box::new(decoder_fn),
        }
    }
}

impl<T> Processor for Decoder<T> {
    type Output = T;

    fn process<R: Response>(&self, response: Result<R>) -> Result<T> {
        let resp = response?;
        if resp.status() != 200 {
            return Err(crate::error::Error::ApiError {
                request_id: resp.request_id().to_owned(),
                status: resp.status().to_owned(),
                message: resp.body().to_owned(),
            });
        }

        let value: decoder::Value = serde_json::from_str(resp.body())?;
        let decoded = (self.decoder_fn)(value)?;
        Ok(decoded)
    }
}
