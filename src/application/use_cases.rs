use std::sync::Arc;

use anyhow::Result;

use crate::{
    application::ports::ProxyGateway,
    domain::{ProxyRequest, ProxyResponse},
};

pub struct ForwardProxyRequestUseCase {
    gateway: Arc<dyn ProxyGateway>,
}

impl ForwardProxyRequestUseCase {
    pub fn new(gateway: Arc<dyn ProxyGateway>) -> Self {
        Self { gateway }
    }

    pub async fn execute(&self, request: ProxyRequest) -> Result<ProxyResponse> {
        self.gateway.forward(request).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use anyhow::Result;
    use async_trait::async_trait;

    use crate::{
        application::{ports::ProxyGateway, use_cases::ForwardProxyRequestUseCase},
        domain::{ProxyRequest, ProxyResponse},
    };

    struct MockGateway {
        response: ProxyResponse,
    }

    #[async_trait]
    impl ProxyGateway for MockGateway {
        async fn forward(&self, _request: ProxyRequest) -> Result<ProxyResponse> {
            Ok(self.response.clone())
        }
    }

    #[tokio::test]
    async fn forwards_request_through_gateway() {
        let use_case = ForwardProxyRequestUseCase::new(Arc::new(MockGateway {
            response: ProxyResponse {
                status: 200,
                headers: vec![("content-type".to_string(), "application/json".to_string())],
                body: br#"{"ok":true}"#.to_vec(),
            },
        }));

        let response = use_case
            .execute(ProxyRequest {
                method: "GET".to_string(),
                path_and_query: "/api/tags".to_string(),
                headers: vec![],
                body: Vec::new(),
            })
            .await
            .expect("use case should return response");

        assert_eq!(response.status, 200);
        assert_eq!(response.body, br#"{"ok":true}"#.to_vec());
    }
}
