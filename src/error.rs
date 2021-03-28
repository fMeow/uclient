use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("HTTP client error: {0}")]
    HttpClient(String),
}
