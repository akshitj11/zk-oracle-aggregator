use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

use axum::extract::Request;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use tokio::sync::Mutex;
use tower::{Layer, Service};

/// Simple token-bucket rate limit (10 requests per second burst).
pub fn rate_limit_layer() -> RateLimitLayer {
    RateLimitLayer {
        state: Arc::new(Mutex::new(RateBucket {
            tokens: 10,
            last_refill: Instant::now(),
        })),
    }
}

#[derive(Clone)]
pub struct RateLimitLayer {
    state: Arc<Mutex<RateBucket>>,
}

struct RateBucket {
    tokens: u32,
    last_refill: Instant,
}

impl RateBucket {
    fn allow(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        if elapsed >= Duration::from_secs(1) {
            self.tokens = 10;
            self.last_refill = now;
        }
        if self.tokens == 0 {
            return false;
        }
        self.tokens -= 1;
        true
    }
}

impl<S> Layer<S> for RateLimitLayer {
    type Service = RateLimitMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        RateLimitMiddleware {
            inner,
            state: self.state.clone(),
        }
    }
}

#[derive(Clone)]
pub struct RateLimitMiddleware<S> {
    inner: S,
    state: Arc<Mutex<RateBucket>>,
}

impl<S> Service<Request> for RateLimitMiddleware<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        if req.uri().path() == "/health" {
            let fut = self.inner.call(req);
            return Box::pin(async move { fut.await });
        }

        let state = self.state.clone();
        let fut = self.inner.call(req);
        Box::pin(async move {
            let mut bucket = state.lock().await;
            if !bucket.allow() {
                return Ok(
                    (StatusCode::TOO_MANY_REQUESTS, "rate limit exceeded").into_response(),
                );
            }
            fut.await
        })
    }
}

/// Require `X-API-Key` when `ORACLE_API_KEY` is configured.
pub fn auth_layer(state: crate::state::AppState) -> AuthLayer {
    AuthLayer { state }
}

#[derive(Clone)]
pub struct AuthLayer {
    state: crate::state::AppState,
}

impl<S> Layer<S> for AuthLayer {
    type Service = AuthMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthMiddleware {
            inner,
            state: self.state.clone(),
        }
    }
}

#[derive(Clone)]
pub struct AuthMiddleware<S> {
    inner: S,
    state: crate::state::AppState,
}

impl<S> Service<Request> for AuthMiddleware<S>
where
    S: Service<Request, Response = Response> + Clone + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        if req.uri().path() == "/health" {
            let fut = self.inner.call(req);
            return Box::pin(async move { fut.await });
        }

        let expected = self.state.api_key();
        if let Some(key) = expected {
            let provided = req
                .headers()
                .get("x-api-key")
                .and_then(|v| v.to_str().ok());
            if provided != Some(key) {
                return Box::pin(async move {
                    Ok((
                        StatusCode::UNAUTHORIZED,
                        axum::Json(crate::api::ApiErrorBody {
                            error: "unauthorized".to_string(),
                        }),
                    )
                        .into_response())
                });
            }
        }

        let fut = self.inner.call(req);
        Box::pin(async move { fut.await })
    }
}
