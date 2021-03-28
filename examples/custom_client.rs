#![allow(unused_imports)]
#![allow(unused_parens)]
use http::{HeaderMap, HeaderValue, Method};
#[cfg(feature = "async_reqwest")]
use reqwest::Client;
use url::Url;

use std::convert::TryInto;
use uclient::{ClientError, ClientExt};

/// when use async http client, `blocking` feature MUST be disabled
// This cfg is only to make rust compiler happy in Github Action, you can just ignore it
#[cfg(feature = "async_reqwest")]
#[derive(Debug, Clone)]
pub struct ReqwestClient(pub Client, HeaderMap);

/// This cfg is only to make rust compiler happy in Github Action, you can just
/// ignore it
#[cfg(feature = "async_reqwest")]
#[async_trait::async_trait]
impl ClientExt for ReqwestClient {
    fn new<U: Into<Option<HeaderMap>>>(headers: U) -> Result<Self, ClientError> {
        let headers = match headers.into() {
            Some(h) => h,
            None => HeaderMap::new(),
        };

        Client::builder()
            .build()
            .map(|c| ReqwestClient(c, headers))
            .map_err(|e| ClientError::HttpClient(format!("{:?}", e)))
    }

    fn headers(&mut self) -> &mut HeaderMap<HeaderValue> {
        &mut self.1
    }

    async fn request(
        &self,
        mut request: http::Request<String>,
    ) -> Result<http::Response<String>, ClientError> {
        let headers = request.headers_mut();
        for (header, value) in self.1.iter() {
            if !headers.contains_key(header) {
                headers.insert(header, value.clone());
            }
        }

        let req = request.try_into().unwrap();

        let resp = self
            .0
            .execute(req)
            .await
            .map_err(|e| ClientError::HttpClient(format!("{:?}", e)))?;
        let status_code = resp.status();
        let headers = resp.headers().clone();
        let version = resp.version();
        let content = resp
            .text()
            .await
            .map_err(|e| ClientError::HttpClient(format!("{:?}", e)))?;
        let mut build = http::Response::builder();

        for header in headers.iter() {
            build = build.header(header.0, header.1);
        }

        http::response::Builder::from(build)
            .status(status_code)
            .version(version)
            .body(content)
            .map_err(|e| ClientError::HttpClient(format!("{:?}", e)))
    }
}

// This cfg is only to make rust compiler happy in Github Action, you can just
// ignore it
#[cfg(feature = "async_reqwest")]
#[tokio::main]
async fn main() {
    let client = ReqwestClient::new(None).unwrap();
    let res = client
        .get(url::Url::parse("https://www.rust-lang.org/").unwrap(), "")
        .await
        .unwrap();
    println!("{}", res.body());
}

#[cfg(not(feature = "async_reqwest"))]
fn main() {}
