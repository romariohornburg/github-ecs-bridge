use std::fmt;
use thiserror::Error;

#[derive(Clone)]
pub struct Settings {
    /// GitHub webhook secret for HMAC-SHA256 signature validation.
    pub webhook_secret: String,
    /// Elasticsearch base URL (e.g. https://localhost:9200).
    pub elasticsearch_url: String,
    /// Pre-encoded Elasticsearch API key (the full base64 `id:api_key` string).
    pub elasticsearch_api_key: String,
    /// Target index or data stream name.
    pub elasticsearch_index: String,
    /// TCP bind address.
    pub listen_addr: String,
    /// Enable the unauthenticated /debug/webhook endpoint. Default: false.
    pub debug_endpoint_enabled: bool,
    /// Elasticsearch HTTP request timeout in seconds. Default: 30.
    pub es_timeout_secs: u64,
}

impl fmt::Debug for Settings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Settings")
            .field("webhook_secret", &"***")
            .field("elasticsearch_url", &self.elasticsearch_url)
            .field("elasticsearch_api_key", &"***")
            .field("elasticsearch_index", &self.elasticsearch_index)
            .field("listen_addr", &self.listen_addr)
            .field("debug_endpoint_enabled", &self.debug_endpoint_enabled)
            .field("es_timeout_secs", &self.es_timeout_secs)
            .finish()
    }
}

impl Settings {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();

        let webhook_secret = std::env::var("WEBHOOK_SECRET")
            .map_err(|_| ConfigError::MissingEnv("WEBHOOK_SECRET"))?;
        if webhook_secret.len() < 20 {
            return Err(ConfigError::InvalidValue(
                "WEBHOOK_SECRET must be at least 20 characters",
            ));
        }

        let elasticsearch_url = std::env::var("ELASTICSEARCH_URL")
            .map_err(|_| ConfigError::MissingEnv("ELASTICSEARCH_URL"))?;
        if !elasticsearch_url.starts_with("http") {
            return Err(ConfigError::InvalidValue(
                "ELASTICSEARCH_URL must start with http:// or https://",
            ));
        }

        let elasticsearch_api_key = std::env::var("ELASTICSEARCH_API_KEY")
            .map_err(|_| ConfigError::MissingEnv("ELASTICSEARCH_API_KEY"))?;
        let elasticsearch_index = std::env::var("ELASTICSEARCH_INDEX")
            .unwrap_or_else(|_| "logs-github.audit-default".into());
        let listen_addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| "0.0.0.0:3001".into());
        let debug_endpoint_enabled = std::env::var("DEBUG_ENDPOINT_ENABLED")
            .map(|v| v.eq_ignore_ascii_case("true") || v == "1")
            .unwrap_or(false);
        let es_timeout_secs = std::env::var("ES_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30u64);

        Ok(Self {
            webhook_secret,
            elasticsearch_url,
            elasticsearch_api_key,
            elasticsearch_index,
            listen_addr,
            debug_endpoint_enabled,
            es_timeout_secs,
        })
    }
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnv(&'static str),
    #[error("Invalid configuration: {0}")]
    InvalidValue(&'static str),
}
