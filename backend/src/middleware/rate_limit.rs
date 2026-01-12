use axum::{
    extract::{Request},
    http::{StatusCode, HeaderMap},
    middleware::Next,
    response::{Response, IntoResponse},
};
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};
use dashmap::DashMap;
use redis::{aio::ConnectionManager};
use serde::{Deserialize, Serialize};
use tracing::{debug, warn, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub anonymous_limit: u32,
    pub authenticated_limit: u32,
    pub premium_limit: u32,
    pub window_secs: u64,
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            anonymous_limit: 100,
            authenticated_limit: 500,
            premium_limit: 2000,
            window_secs: 60,
            burst_size: 30,
        }
    }
}

impl RateLimitConfig {
    pub fn read_heavy() -> Self {
        Self {
            anonymous_limit: 150,
            authenticated_limit: 1000,
            premium_limit: 5000,
            window_secs: 60,
            burst_size: 50,
        }
    }

    pub fn write_heavy() -> Self {
        Self {
            anonymous_limit: 50,
            authenticated_limit: 200,
            premium_limit: 1000,
            window_secs: 60,
            burst_size: 20,
        }
    }

    pub fn expensive() -> Self {
        Self {
            anonymous_limit: 60,
            authenticated_limit: 300,
            premium_limit: 1500,
            window_secs: 60,
            burst_size: 20,
        }
    }
}

#[derive(Clone)]
pub enum RateLimiterBackend {
    Memory(MemoryBackend),
    Hybrid(HybridBackend),
}

impl RateLimiterBackend {
    pub fn memory() -> Self {
        Self::Memory(MemoryBackend {
            store: Arc::new(DashMap::new()),
        })
    }

    pub async fn hybrid(redis_url: &str) -> Self {
        match redis::Client::open(redis_url) {
            Ok(client) => match ConnectionManager::new(client).await {
                Ok(_conn) => Self::Hybrid(HybridBackend {
                    memory: MemoryBackend {
                        store: Arc::new(DashMap::new()),
                    },
                }),
                Err(e) => {
                    warn!("Failed to connect to Redis, using memory backend: {}", e);
                    Self::Memory(MemoryBackend {
                        store: Arc::new(DashMap::new()),
                    })
                }
            },
            Err(e) => {
                warn!("Failed to parse Redis URL, using memory backend: {}", e);
                Self::Memory(MemoryBackend {
                    store: Arc::new(DashMap::new()),
                })
            }
        }
    }

    async fn check_rate_limit(
        &self,
        key: &str,
        limit: u32,
        window_secs: u64,
        burst_size: u32,
    ) -> Result<RateLimitResult, String> {
        match self {
            Self::Memory(backend) => backend.check(key, limit, window_secs, burst_size).await,
            Self::Hybrid(backend) => backend.check(key, limit, window_secs, burst_size).await,
        }
    }
}

#[derive(Clone)]
pub struct MemoryBackend {
    store: Arc<DashMap<String, RateLimitEntry>>,
}

impl MemoryBackend {
    async fn check(
        &self,
        key: &str,
        limit: u32,
        window_secs: u64,
        burst_size: u32,
    ) -> Result<RateLimitResult, String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let window_start = (now / window_secs) * window_secs;
        let burst_window_start = (now / 10) * 10;
        let total_limit = limit + burst_size;

        let mut entry = self.store.entry(key.to_string()).or_insert(RateLimitEntry {
            count: 0,
            burst_count: 0,
            window_start,
            burst_window_start,
        });

        if entry.window_start < window_start {
            entry.count = 0;
            entry.burst_count = 0;
            entry.window_start = window_start;
            entry.burst_window_start = burst_window_start;
        }

        if entry.burst_window_start < burst_window_start {
            entry.burst_count = 0;
            entry.burst_window_start = burst_window_start;
        }

        if entry.count >= total_limit {
            let reset_at = window_start + window_secs;
            return Ok(RateLimitResult {
                allowed: false,
                limit: total_limit,
                remaining: 0,
                reset_at,
            });
        }

        entry.count += 1;
        
        if entry.count > limit {
            entry.burst_count += 1;
        }
        
        let remaining = total_limit.saturating_sub(entry.count);
        let reset_at = window_start + window_secs;

        Ok(RateLimitResult {
            allowed: true,
            limit: total_limit,
            remaining,
            reset_at,
        })
    }
}

#[derive(Clone)]
pub struct HybridBackend {
    memory: MemoryBackend,
}

impl HybridBackend {
    async fn check(
        &self,
        key: &str,
        limit: u32,
        window_secs: u64,
        burst_size: u32,
    ) -> Result<RateLimitResult, String> {
        self.memory.check(key, limit, window_secs, burst_size).await
    }
}

#[derive(Debug, Clone)]
struct RateLimitEntry {
    count: u32,
    burst_count: u32,
    window_start: u64,
    burst_window_start: u64,
}

#[derive(Debug, Clone)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub limit: u32,
    pub remaining: u32,
    pub reset_at: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UserTier {
    Anonymous,
    Authenticated,
    Premium,
}

#[derive(Clone)]
pub struct RateLimitLayer {
    backend: RateLimiterBackend,
    config: RateLimitConfig,
}

impl RateLimitLayer {
    pub fn new(backend: RateLimiterBackend, config: RateLimitConfig) -> Self {
        Self { backend, config }
    }

    pub async fn with_defaults(redis_url: Option<&str>) -> Self {
        let backend = if let Some(url) = redis_url {
            RateLimiterBackend::hybrid(url).await
        } else {
            RateLimiterBackend::memory()
        };

        Self::new(backend, RateLimitConfig::default())
    }

    pub async fn read_heavy(redis_url: Option<&str>) -> Self {
        let backend = if let Some(url) = redis_url {
            RateLimiterBackend::hybrid(url).await
        } else {
            RateLimiterBackend::memory()
        };

        Self::new(backend, RateLimitConfig::read_heavy())
    }

    pub async fn write_heavy(redis_url: Option<&str>) -> Self {
        let backend = if let Some(url) = redis_url {
            RateLimiterBackend::hybrid(url).await
        } else {
            RateLimiterBackend::memory()
        };

        Self::new(backend, RateLimitConfig::write_heavy())
    }

    pub async fn expensive(redis_url: Option<&str>) -> Self {
        let backend = if let Some(url) = redis_url {
            RateLimiterBackend::hybrid(url).await
        } else {
            RateLimiterBackend::memory()
        };

        Self::new(backend, RateLimitConfig::expensive())
    }

    pub async fn middleware(
        &self,
        headers: HeaderMap,
        request: Request,
        next: Next,
    ) -> Response {
        let ip_addr = headers
            .get("x-forwarded-for")
            .and_then(|h| h.to_str().ok())
            .and_then(|s| s.split(',').next())
            .and_then(|s| s.trim().parse::<std::net::IpAddr>().ok())
            .or_else(|| {
                headers
                    .get("x-real-ip")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.parse::<std::net::IpAddr>().ok())
            })
            .unwrap_or_else(|| "127.0.0.1".parse().unwrap());

        let (user_key, tier) = self.extract_user_info(&headers, ip_addr);

        let limit = match tier {
            UserTier::Anonymous => self.config.anonymous_limit,
            UserTier::Authenticated => self.config.authenticated_limit,
            UserTier::Premium => self.config.premium_limit,
        };

        debug!(
            "Rate limit check - User: {}, Tier: {:?}, Limit: {}",
            user_key, tier, limit
        );

        match self
            .backend
            .check_rate_limit(
                &user_key,
                limit,
                self.config.window_secs,
                self.config.burst_size,
            )
            .await
        {
            Ok(result) => {
                if result.allowed {
                    let mut response = next.run(request).await;
                    self.add_rate_limit_headers(&mut response, &result);
                    response
                } else {
                    debug!("Rate limit exceeded for user: {}", user_key);
                    self.rate_limit_exceeded_response(&result)
                }
            }
            Err(e) => {
                error!("Rate limit check error: {}", e);
                next.run(request).await
            }
        }
    }

    fn extract_user_info(&self, headers: &HeaderMap, ip_addr: std::net::IpAddr) -> (String, UserTier) {
        if let Some(auth_header) = headers.get("authorization") {
            if let Ok(auth_str) = auth_header.to_str() {
                if auth_str.starts_with("Bearer premium_") {
                    return (format!("premium:{}", auth_str), UserTier::Premium);
                }
                if auth_str.starts_with("Bearer ") {
                    return (format!("auth:{}", auth_str), UserTier::Authenticated);
                }
            }
        }

        if let Some(api_key) = headers.get("x-api-key") {
            if let Ok(key_str) = api_key.to_str() {
                if key_str.starts_with("premium_") {
                    return (format!("premium:{}", key_str), UserTier::Premium);
                }
                return (format!("api:{}", key_str), UserTier::Authenticated);
            }
        }

        if let Some(wallet) = headers.get("x-wallet-address") {
            if let Ok(wallet_str) = wallet.to_str() {
                return (format!("wallet:{}", wallet_str), UserTier::Authenticated);
            }
        }

        (format!("ip:{}", ip_addr), UserTier::Anonymous)
    }

    fn add_rate_limit_headers(&self, response: &mut Response, result: &RateLimitResult) {
        let headers = response.headers_mut();
        headers.insert(
            "X-RateLimit-Limit",
            result.limit.to_string().parse().unwrap(),
        );
        headers.insert(
            "X-RateLimit-Remaining",
            result.remaining.to_string().parse().unwrap(),
        );
        headers.insert(
            "X-RateLimit-Reset",
            result.reset_at.to_string().parse().unwrap(),
        );
    }

    fn rate_limit_exceeded_response(&self, result: &RateLimitResult) -> Response {
        let retry_after = result.reset_at
            - SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();

        let body = serde_json::json!({
            "error": "Rate limit exceeded",
            "message": format!(
                "Too many requests. Please try again in {} seconds.",
                retry_after
            ),
            "limit": result.limit,
            "reset_at": result.reset_at,
            "retry_after": retry_after,
        });

        let mut response = (
            StatusCode::TOO_MANY_REQUESTS,
            [(axum::http::header::CONTENT_TYPE, "application/json")],
            serde_json::to_string(&body).unwrap(),
        )
            .into_response();

        self.add_rate_limit_headers(&mut response, result);
        response
            .headers_mut()
            .insert("Retry-After", retry_after.to_string().parse().unwrap());

        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_backend() {
        let backend = RateLimiterBackend::memory();
        let config = RateLimitConfig {
            anonymous_limit: 5,
            authenticated_limit: 10,
            premium_limit: 100,
            window_secs: 60,
            burst_size: 2,
        };

        for i in 1..=5 {
            let result = backend
                .check_rate_limit("test_user", config.anonymous_limit, config.window_secs, config.burst_size)
                .await
                .unwrap();
            assert!(result.allowed, "Request {} should be allowed", i);
        }

        for i in 1..=2 {
            let result = backend
                .check_rate_limit("test_user", config.anonymous_limit, config.window_secs, config.burst_size)
                .await
                .unwrap();
            assert!(result.allowed, "Burst request {} should be allowed", i);
        }

        let result = backend
            .check_rate_limit("test_user", config.anonymous_limit, config.window_secs, config.burst_size)
            .await
            .unwrap();
        assert!(!result.allowed, "Request should be rate limited");
    }

    #[tokio::test]
    async fn test_different_users() {
        let backend = RateLimiterBackend::memory();
        let config = RateLimitConfig::default();

        for _ in 1..=config.anonymous_limit {
            backend
                .check_rate_limit("user1", config.anonymous_limit, config.window_secs, config.burst_size)
                .await
                .unwrap();
        }

        let result = backend
            .check_rate_limit("user2", config.anonymous_limit, config.window_secs, config.burst_size)
            .await
            .unwrap();
        assert!(result.allowed, "Different user should not be rate limited");
    }
}
