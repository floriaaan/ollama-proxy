use anyhow::Result;
use async_trait::async_trait;

use crate::domain::{ProxyRequest, ProxyResponse};

#[async_trait]
pub trait ProxyGateway: Send + Sync {
    async fn forward(&self, request: ProxyRequest) -> Result<ProxyResponse>;
}
