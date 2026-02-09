use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// A boxed future that is Send, used for middleware return types.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Represents the next handler in the middleware chain.
#[derive(Clone)]
pub struct Next<'a> {
    inner: Arc<dyn Fn(reqwest::Request) -> BoxFuture<'a, Result<reqwest::Response, crate::error::Error>> + Send + Sync + 'a>,
}

impl<'a> Next<'a> {
    pub fn new(
        f: impl Fn(reqwest::Request) -> BoxFuture<'a, Result<reqwest::Response, crate::error::Error>> + Send + Sync + 'a,
    ) -> Self {
        Self { inner: Arc::new(f) }
    }

    pub fn run(&self, request: reqwest::Request) -> BoxFuture<'a, Result<reqwest::Response, crate::error::Error>> {
        (self.inner)(request)
    }
}

/// Middleware trait for intercepting and transforming requests.
///
/// Used by Bedrock and Vertex integrations to rewrite requests
/// (e.g., sign with SigV4, inject OAuth tokens, rewrite URLs).
pub trait Middleware: Send + Sync {
    fn handle<'a>(
        &'a self,
        request: reqwest::Request,
        next: Next<'a>,
    ) -> BoxFuture<'a, Result<reqwest::Response, crate::error::Error>>;
}

/// Execute a request through a middleware chain, calling the final handler at the end.
pub fn execute_middleware_chain<'a>(
    middlewares: &'a [Box<dyn Middleware>],
    request: reqwest::Request,
    handler: impl Fn(reqwest::Request) -> BoxFuture<'a, Result<reqwest::Response, crate::error::Error>> + Send + Sync + 'a,
) -> BoxFuture<'a, Result<reqwest::Response, crate::error::Error>> {
    if middlewares.is_empty() {
        return handler(request);
    }

    let next = build_chain(middlewares, handler);
    next.run(request)
}

/// Recursively build the middleware chain from the inside out.
fn build_chain<'a>(
    middlewares: &'a [Box<dyn Middleware>],
    handler: impl Fn(reqwest::Request) -> BoxFuture<'a, Result<reqwest::Response, crate::error::Error>> + Send + Sync + 'a,
) -> Next<'a> {
    if middlewares.is_empty() {
        return Next::new(handler);
    }

    let (first, rest) = middlewares.split_first().unwrap();
    let inner_next = build_chain(rest, handler);

    Next::new(move |req| {
        let inner_clone = inner_next.clone();
        first.handle(req, inner_clone)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AddHeaderMiddleware {
        name: &'static str,
        value: &'static str,
    }

    impl Middleware for AddHeaderMiddleware {
        fn handle<'a>(
            &'a self,
            mut request: reqwest::Request,
            next: Next<'a>,
        ) -> BoxFuture<'a, Result<reqwest::Response, crate::error::Error>> {
            Box::pin(async move {
                request
                    .headers_mut()
                    .insert(self.name, self.value.parse().unwrap());
                next.run(request).await
            })
        }
    }

    #[tokio::test]
    async fn test_empty_middleware_chain() {
        let middlewares: Vec<Box<dyn Middleware>> = vec![];
        let handler = |_req: reqwest::Request| -> BoxFuture<'_, Result<reqwest::Response, crate::error::Error>> {
            Box::pin(async {
                Ok(reqwest::Response::from(
                    http::Response::builder()
                        .status(200)
                        .body("")
                        .unwrap(),
                ))
            })
        };

        let req = reqwest::Request::new(reqwest::Method::GET, "https://example.com".parse().unwrap());
        let resp = execute_middleware_chain(&middlewares, req, handler).await.unwrap();
        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn test_single_middleware() {
        let middlewares: Vec<Box<dyn Middleware>> = vec![Box::new(AddHeaderMiddleware {
            name: "x-test",
            value: "hello",
        })];

        let handler = |req: reqwest::Request| -> BoxFuture<'_, Result<reqwest::Response, crate::error::Error>> {
            Box::pin(async move {
                // Verify the middleware added the header
                assert_eq!(req.headers().get("x-test").unwrap(), "hello");
                Ok(reqwest::Response::from(
                    http::Response::builder()
                        .status(200)
                        .body("")
                        .unwrap(),
                ))
            })
        };

        let req = reqwest::Request::new(reqwest::Method::GET, "https://example.com".parse().unwrap());
        let resp = execute_middleware_chain(&middlewares, req, handler).await.unwrap();
        assert_eq!(resp.status(), 200);
    }
}
