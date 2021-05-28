use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP client error: {0}")]
    HttpClient(String),
    #[cfg(feature = "multipart")]
    #[error("Multipart error: {0}")]
    MultiPart(#[from] mime_multipart::Error),
    #[error("Invalid file")]
    InvalidFile,
    #[error("Payload Error")]
    PayloadError,
}
