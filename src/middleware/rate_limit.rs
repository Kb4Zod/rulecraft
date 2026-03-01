use axum::{
    extract::ConnectInfo,
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};
use governor::{
    clock::{DefaultClock, QuantaInstant},
    middleware::NoOpMiddleware,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use std::{
    collections::HashMap,
    net::{IpAddr, SocketAddr},
    num::NonZeroU32,
    sync::Arc,
};
use tokio::sync::RwLock;

type SharedRateLimiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock, NoOpMiddleware<QuantaInstant>>;

#[derive(Clone)]
pub struct RateLimitConfig {
    pub ai_requests_per_hour: u32,
    pub search_requests_per_minute: u32,
    pub general_requests_per_minute: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            ai_requests_per_hour: 5,
            search_requests_per_minute: 30,
            general_requests_per_minute: 60,
        }
    }
}

#[derive(Clone)]
pub struct RateLimitState {
    ai_limiters: Arc<RwLock<HashMap<IpAddr, Arc<SharedRateLimiter>>>>,
    search_limiters: Arc<RwLock<HashMap<IpAddr, Arc<SharedRateLimiter>>>>,
    general_limiters: Arc<RwLock<HashMap<IpAddr, Arc<SharedRateLimiter>>>>,
    config: RateLimitConfig,
}

impl RateLimitState {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            ai_limiters: Arc::new(RwLock::new(HashMap::new())),
            search_limiters: Arc::new(RwLock::new(HashMap::new())),
            general_limiters: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    async fn get_or_create_limiter(
        limiters: &RwLock<HashMap<IpAddr, Arc<SharedRateLimiter>>>,
        ip: IpAddr,
        requests_per_period: u32,
        period_secs: u64,
    ) -> Arc<SharedRateLimiter> {
        // Fast path: check if limiter exists
        {
            let read_guard = limiters.read().await;
            if let Some(limiter) = read_guard.get(&ip) {
                return Arc::clone(limiter);
            }
        }

        // Slow path: create new limiter
        let replenish_interval = period_secs * 1_000_000_000 / requests_per_period as u64;
        let quota = Quota::with_period(std::time::Duration::from_nanos(replenish_interval))
            .expect("Invalid quota period")
            .allow_burst(NonZeroU32::new(requests_per_period).unwrap_or(NonZeroU32::MIN));

        let limiter = Arc::new(RateLimiter::direct(quota));

        let mut write_guard = limiters.write().await;
        // Double-check pattern: another task might have created it
        if let Some(existing) = write_guard.get(&ip) {
            return Arc::clone(existing);
        }
        write_guard.insert(ip, Arc::clone(&limiter));
        limiter
    }

    pub async fn check_rate_limit(&self, ip: IpAddr, path: &str, method: &str) -> Result<(), RateLimitError> {
        let (limiter, limit_type) = if path.starts_with("/scenario") && method == "POST" {
            // AI endpoint: strictest limit
            let limiter = Self::get_or_create_limiter(
                &self.ai_limiters,
                ip,
                self.config.ai_requests_per_hour,
                3600, // 1 hour
            )
            .await;
            (limiter, RateLimitType::Ai)
        } else if path.starts_with("/api/search") || path.starts_with("/search") {
            // Search endpoints: medium limit
            let limiter = Self::get_or_create_limiter(
                &self.search_limiters,
                ip,
                self.config.search_requests_per_minute,
                60, // 1 minute
            )
            .await;
            (limiter, RateLimitType::Search)
        } else {
            // General endpoints: relaxed limit
            let limiter = Self::get_or_create_limiter(
                &self.general_limiters,
                ip,
                self.config.general_requests_per_minute,
                60, // 1 minute
            )
            .await;
            (limiter, RateLimitType::General)
        };

        match limiter.check() {
            Ok(_) => Ok(()),
            Err(_) => Err(RateLimitError { limit_type }),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RateLimitType {
    Ai,
    Search,
    General,
}

#[derive(Debug)]
pub struct RateLimitError {
    pub limit_type: RateLimitType,
}

impl IntoResponse for RateLimitError {
    fn into_response(self) -> Response {
        let (retry_after, message) = match self.limit_type {
            RateLimitType::Ai => ("3600", "AI rate limit exceeded. Please try again later."),
            RateLimitType::Search => ("60", "Search rate limit exceeded. Please try again in a minute."),
            RateLimitType::General => ("60", "Rate limit exceeded. Please try again in a minute."),
        };

        (
            StatusCode::TOO_MANY_REQUESTS,
            [("Retry-After", retry_after), ("Content-Type", "application/json")],
            format!(r#"{{"error": "{}"}}"#, message),
        )
            .into_response()
    }
}

/// Extract client IP from request, considering proxy headers
pub fn extract_client_ip<B>(req: &Request<B>) -> IpAddr {
    // Try X-Forwarded-For header first (for proxied requests through Cloudflare/nginx)
    if let Some(forwarded) = req.headers().get("X-Forwarded-For") {
        if let Ok(value) = forwarded.to_str() {
            // Take the first IP in the chain (original client)
            if let Some(ip_str) = value.split(',').next() {
                if let Ok(ip) = ip_str.trim().parse::<IpAddr>() {
                    return ip;
                }
            }
        }
    }

    // Try CF-Connecting-IP header (Cloudflare specific)
    if let Some(cf_ip) = req.headers().get("CF-Connecting-IP") {
        if let Ok(value) = cf_ip.to_str() {
            if let Ok(ip) = value.trim().parse::<IpAddr>() {
                return ip;
            }
        }
    }

    // Try X-Real-IP header
    if let Some(real_ip) = req.headers().get("X-Real-IP") {
        if let Ok(value) = real_ip.to_str() {
            if let Ok(ip) = value.trim().parse::<IpAddr>() {
                return ip;
            }
        }
    }

    // Fall back to connection info
    if let Some(ConnectInfo(addr)) = req.extensions().get::<ConnectInfo<SocketAddr>>() {
        return addr.ip();
    }

    // Ultimate fallback - use loopback (shouldn't happen in production)
    IpAddr::from([127, 0, 0, 1])
}
