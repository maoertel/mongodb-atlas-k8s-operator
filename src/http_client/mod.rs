#![allow(dead_code)]
pub(crate) mod error;

use reqwest::{
    header::{HeaderMap, HeaderValue, ACCEPT, CONTENT_TYPE},
    Client,
};

use crate::http_client::error::Result;

const CONTENT_TYPE_JSON: &str = "application/json; charset=utf-8";

pub(crate) fn default() -> Result<Client> {
    let headers = default_headers();
    create_client(headers)
}

pub(crate) fn accepts(content: &str) -> Result<Client> {
    let headers = default_headers().set_accept(content)?;
    create_client(headers)
}

fn default_headers() -> HeaderMap {
    HeaderMap::new().set_content_type()
}

fn create_client(headers: HeaderMap) -> Result<Client> {
    let client = Client::builder().default_headers(headers).build()?;
    Ok(client)
}

pub(crate) trait HeadersExt {
    fn set_content_type(self) -> HeaderMap;
    fn set_accept(self, content: &str) -> Result<HeaderMap>;
}

impl HeadersExt for HeaderMap {
    fn set_content_type(mut self) -> HeaderMap {
        self.insert(CONTENT_TYPE, HeaderValue::from_static(CONTENT_TYPE_JSON));
        self
    }

    fn set_accept(mut self, content: &str) -> Result<HeaderMap> {
        self.insert(ACCEPT, HeaderValue::from_str(content)?);
        Ok(self)
    }
}
