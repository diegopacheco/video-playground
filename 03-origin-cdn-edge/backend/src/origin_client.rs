use crate::error::CdnError;
use bytes::Bytes;
use std::future::Future;

#[derive(Debug, Clone)]
pub struct OriginResponse {
    pub body: Bytes,
    pub content_type: String,
    pub cache_control: Option<String>,
}

pub trait OriginClient: Send + Sync + 'static {
    fn fetch(&self, key: &str) -> impl Future<Output = Result<OriginResponse, CdnError>> + Send;
}

pub struct HttpOrigin {
    client: reqwest::Client,
    base_url: String,
}

impl HttpOrigin {
    pub fn new(base_url: &str) -> Self {
        HttpOrigin {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }
}

impl OriginClient for HttpOrigin {
    fn fetch(&self, key: &str) -> impl Future<Output = Result<OriginResponse, CdnError>> + Send {
        let request = self.client.get(format!("{}/seg/{}", self.base_url, key));
        async move {
            let response = request
                .send()
                .await
                .map_err(|e| CdnError::OriginUnreachable(e.to_string()))?;
            let status = response.status();
            if !status.is_success() {
                return Err(CdnError::OriginStatus(status.as_u16()));
            }
            let header = |name: &str| {
                response
                    .headers()
                    .get(name)
                    .and_then(|v| v.to_str().ok())
                    .map(str::to_string)
            };
            let content_type = header("content-type").unwrap_or_else(|| "application/octet-stream".to_string());
            let cache_control = header("cache-control");
            let body = response
                .bytes()
                .await
                .map_err(|e| CdnError::OriginUnreachable(e.to_string()))?;
            Ok(OriginResponse {
                body,
                content_type,
                cache_control,
            })
        }
    }
}
