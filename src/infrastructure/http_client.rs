use std::time::Duration;

use anyhow::{Context, Result};
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue, AUTHORIZATION};

use crate::{
    application::ports::ProxyGateway,
    domain::{sanitize_outbound_headers, ProxyRequest, ProxyResponse},
};

pub struct ReqwestProxyGateway {
    client: reqwest::Client,
    host: String,
    token: String,
}

impl ReqwestProxyGateway {
    pub fn new(host: String, token: String, timeout: Option<Duration>) -> Result<Self> {
        let mut client_builder = reqwest::Client::builder();

        if let Some(timeout) = timeout {
            client_builder = client_builder.timeout(timeout);
        }

        let client = client_builder
            .build()
            .context("failed to build reqwest client")?;

        Ok(Self {
            client,
            host,
            token,
        })
    }

    fn build_headers(&self, request_headers: &[(String, String)]) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();

        for (name, value) in sanitize_outbound_headers(request_headers) {
            let header_name = HeaderName::from_bytes(name.as_bytes())
                .with_context(|| format!("invalid header name: {name}"))?;
            let header_value = HeaderValue::from_str(&value)
                .with_context(|| format!("invalid header value for {name}"))?;
            headers.append(header_name, header_value);
        }

        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.token))
                .context("failed to set Authorization header")?,
        );

        Ok(headers)
    }
}

#[async_trait]
impl ProxyGateway for ReqwestProxyGateway {
    async fn forward(&self, request: ProxyRequest) -> Result<ProxyResponse> {
        let upstream_url = format!("{}{}", self.host, request.path_and_query);
        let method = reqwest::Method::from_bytes(request.method.as_bytes())
            .with_context(|| format!("unsupported method: {}", request.method))?;

        let headers = self.build_headers(&request.headers)?;

        let response = self
            .client
            .request(method, upstream_url)
            .headers(headers)
            .body(request.body)
            .send()
            .await
            .context("upstream request failed")?;

        let status = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .filter_map(|(name, value)| {
                value
                    .to_str()
                    .ok()
                    .map(|v| (name.to_string(), v.to_string()))
            })
            .collect::<Vec<_>>();
        let body = response
            .bytes()
            .await
            .context("failed to read upstream body")?;

        Ok(ProxyResponse {
            status,
            headers,
            body: body.to_vec(),
        })
    }
}
