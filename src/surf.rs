//! Surf HTTP client
use std::str::FromStr;

use http::{
    header::{HeaderMap, HeaderValue},
    Method, Request, Response, StatusCode, Version,
};

use crate::{ClientExt, Error};
use std::io::{BufReader, Read};

#[derive(Debug, Clone)]
pub struct SurfClient {
    headers: HeaderMap,
}

#[async_trait::async_trait]
impl ClientExt for SurfClient {
    fn new<U: Into<Option<HeaderMap>>>(headers: U) -> Result<Self, Error> {
        let headers = match headers.into() {
            Some(h) => h,
            None => HeaderMap::new(),
        };

        Ok(SurfClient { headers })
    }

    fn headers(&mut self) -> &mut HeaderMap<HeaderValue> {
        &mut self.headers
    }

    async fn request_reader<T>(&self, request: Request<T>) -> Result<Response<String>, Error>
    where
        T: Read + Send + Sync + 'static,
    {
        use ::surf::http::headers::HeaderName as SurfHeaderName;

        let method = request.method().clone();
        let url = request.uri().to_owned().to_string();

        let req = match method {
            Method::GET => ::surf::get(url),
            Method::POST => ::surf::post(url),
            Method::PUT => ::surf::put(url),
            Method::DELETE => ::surf::delete(url),
            Method::PATCH => ::surf::patch(url),
            Method::CONNECT => ::surf::connect(url),
            Method::HEAD => ::surf::head(url),
            Method::OPTIONS => ::surf::options(url),
            Method::TRACE => ::surf::trace(url),
            m @ _ => return Err(Error::HttpClient(format!("invalid method {}", m))),
        };

        let req = self.headers.iter().fold(req, |req, (k, v)| {
            req.header(
                SurfHeaderName::from_str(k.as_str()).unwrap(),
                v.to_str().unwrap(),
            )
        });
        let req = request.headers().iter().fold(req, |req, (k, v)| {
            req.header(
                SurfHeaderName::from_str(k.as_str()).unwrap(),
                v.to_str().unwrap(),
            )
        });

        let text = request.into_body();
        let body =
            surf::Body::from_reader(futures::io::AllowStdIo::new(BufReader::new(text)), None);
        let mut resp = req
            .body(body)
            .await
            .map_err(|e| Error::HttpClient(format!("{:?}", e)))?;

        let status_code = resp.status();
        let status = u16::from(status_code);

        let version = resp.version();
        let content = resp
            .body_string()
            .await
            .map_err(|e| Error::HttpClient(format!("{:?}", e)))?;

        let mut build = http::Response::builder();
        for (name, value) in resp.iter() {
            let mut iter = value.iter();
            let acc = iter.next().map(|v| v.as_str()).unwrap_or("").to_owned();
            let s = iter.fold(acc, |acc, x| format!("{};{}", acc, x.as_str()));
            build = build.header(name.as_str(), s);
        }

        let http_version = version.map(|v| match v {
            ::surf::http::Version::Http0_9 => Version::HTTP_09,
            ::surf::http::Version::Http1_0 => Version::HTTP_10,
            ::surf::http::Version::Http1_1 => Version::HTTP_11,
            ::surf::http::Version::Http2_0 => Version::HTTP_2,
            ::surf::http::Version::Http3_0 => Version::HTTP_3,
            _ => unreachable!(),
        });

        let mut resp =
            http::response::Builder::from(build).status(StatusCode::from_u16(status).unwrap());
        if version.is_some() {
            resp = resp.version(http_version.unwrap());
        }
        resp.body(content)
            .map_err(|e| Error::HttpClient(format!("{:?}", e)))
    }
}
