#![allow(unused_imports)]
#![allow(unused_parens)]
#[cfg(feature = "async_reqwest")]
use futures::StreamExt;
use http::{HeaderMap, HeaderValue, Method};
#[cfg(feature = "async_reqwest")]
use reqwest::{Body, Client};
use url::Url;

use http::request::Parts;
use std::convert::TryInto;
use std::io::Read;
use uclient::{ClientExt, Error};

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
    type Client = Client;

    fn with_client(
        client: Self::Client,
        headers: impl Into<Option<HeaderMap>>,
    ) -> Result<Self, Error> {
        let headers = match headers.into() {
            Some(h) => h,
            None => HeaderMap::new(),
        };

        Ok(ReqwestClient(client, headers))
    }

    fn new<U: Into<Option<HeaderMap>>>(headers: U) -> Result<Self, Error> {
        let headers = match headers.into() {
            Some(h) => h,
            None => HeaderMap::new(),
        };

        Client::builder()
            .build()
            .map(|c| ReqwestClient(c, headers))
            .map_err(|e| Error::HttpClient(format!("{:?}", e)))
    }

    fn headers(&mut self) -> &mut HeaderMap<HeaderValue> {
        &mut self.1
    }

    async fn request_reader<T>(
        &self,
        mut request: http::Request<T>,
    ) -> Result<http::Response<String>, Error>
    where
        T: Read + Send + Sync + 'static,
    {
        // construct reqwest body
        let (parts, body) = request.into_parts();
        let Parts {
            method,
            uri,
            headers,
            ..
        } = parts;
        let url = Url::parse(&uri.to_string()).expect("invalid url");
        let mut request = reqwest::Request::new(method, url);

        // insert header
        let mut prev_name = None;
        for (key, value) in headers {
            match key {
                Some(key) => {
                    request.headers_mut().insert(key.clone(), value);
                    prev_name = Some(key);
                }
                None => match prev_name {
                    Some(ref key) => {
                        request.headers_mut().append(key.clone(), value);
                    }
                    None => unreachable!("HeaderMap::into_iter yielded None first"),
                },
            }
        }

        // convert an object with read trait to stream
        let body_bytes = body.bytes();
        let stream = futures::stream::iter(body_bytes).chunks(2048).map(|x| {
            let len = x.len();
            let out = x.into_iter().filter_map(|b| b.ok()).collect::<Vec<_>>();
            if out.len() == len {
                Ok(out)
            } else {
                Err(crate::Error::PayloadError)
            }
        });
        request.body_mut().replace(Body::wrap_stream(stream));

        let resp = self
            .0
            .execute(request)
            .await
            .map_err(|e| Error::HttpClient(format!("{:?}", e)))?;
        let status_code = resp.status();
        let headers = resp.headers().clone();
        let version = resp.version();
        let content = resp
            .text()
            .await
            .map_err(|e| Error::HttpClient(format!("{:?}", e)))?;
        let mut build = http::Response::builder();

        for header in headers.iter() {
            build = build.header(header.0, header.1);
        }

        http::response::Builder::from(build)
            .status(status_code)
            .version(version)
            .body(content)
            .map_err(|e| Error::HttpClient(format!("{:?}", e)))
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
