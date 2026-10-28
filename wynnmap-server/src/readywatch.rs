use std::time::Duration;

use axum::{body::Body, extract::Request, response::Response};
use futures::{TryFutureExt, future::BoxFuture};
use reqwest::header;
use tokio::{sync::watch, time::timeout};
use tower::{Layer, Service};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReadyState {
    Ready,
    Waiting,
    Loading,
}

impl ReadyState {
    pub const fn ready(&self) -> bool {
        matches!(self, ReadyState::Ready)
    }
}

#[derive(Clone)]
pub struct ReadyLayer {
    watch: watch::Receiver<ReadyState>,
}

impl ReadyLayer {
    pub const fn new(watch: watch::Receiver<ReadyState>) -> Self {
        Self { watch }
    }
}

impl<S> Layer<S> for ReadyLayer {
    type Service = ReadyMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ReadyMiddleware {
            watch: self.watch.clone(),
            inner,
        }
    }
}

#[derive(Clone)]
pub struct ReadyMiddleware<S> {
    watch: watch::Receiver<ReadyState>,
    inner: S,
}

impl<S> Service<Request> for ReadyMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;

    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        let mut watch = self.watch.clone();
        let future = self.inner.call(req);

        Box::pin(async move {
            match timeout(
                Duration::from_secs(60),
                watch.wait_for(ReadyState::ready).map_ok(|_| ()),
            )
            .await
            {
                Ok(Ok(_)) => {
                    let res = future.await?;
                    Ok(res)
                }
                Ok(Err(_)) => Ok(error_500()),
                Err(_) => Ok(error_503()),
            }
        })
    }
}

fn error_500() -> Response {
    Response::builder()
        .status(500)
        .header(header::CONTENT_TYPE, "text/plain")
        .header(header::CACHE_CONTROL, "no-store")
        .body(Body::from("500: Internal Server Error"))
        .unwrap()
}

fn error_503() -> Response {
    Response::builder()
        .status(503)
        .header(header::CONTENT_TYPE, "text/plain")
        .header(header::CACHE_CONTROL, "no-store")
        .header(header::RETRY_AFTER, "60")
        .body(Body::from("503: Service Unavailable"))
        .unwrap()
}
