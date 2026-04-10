use reqwest::{Client, Proxy};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProxyType {
    Http,
    Socks5,
    Direct,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyConfig {
    pub proxy_type: ProxyType,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Default for ProxyConfig {
    fn default() -> Self {
        Self {
            proxy_type: ProxyType::Direct,
            host: String::new(),
            port: 0,
            username: None,
            password: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetryConfig {
    pub max_retries: u32,
    pub retry_delay_ms: u64,
    pub fallback_urls: Vec<String>,
    pub auto_retry: bool,
    pub skip_failed_segments: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            retry_delay_ms: 2000,
            fallback_urls: vec![],
            auto_retry: true,
            skip_failed_segments: false,
        }
    }
}

/// Build a reqwest::Client with the given proxy configuration.
pub fn build_client(proxy_config: &ProxyConfig) -> Client {
    let mut builder = Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36");

    match &proxy_config.proxy_type {
        ProxyType::Direct => {},
        ProxyType::Http => {
            let url = format!("http://{}:{}", proxy_config.host, proxy_config.port);
            if let Ok(proxy) = Proxy::all(&url) {
                builder = builder.proxy(proxy);
            }
        },
        ProxyType::Socks5 => {
            let url = format!("socks5://{}:{}", proxy_config.host, proxy_config.port);
            if let Ok(proxy) = Proxy::all(&url) {
                builder = builder.proxy(proxy);
            }
        },
    }

    builder.build().unwrap_or_else(|_| Client::new())
}
