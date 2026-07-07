#[cfg(feature = "lambda")]
pub use vercel_runtime;
#[cfg(feature = "lambda")]
pub use tokio;

pub use http;
use http::{Request, Response};
use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;

pub type Bytes = Vec<u8>;
pub type PortableRequest = Request<Bytes>;
pub type PortableResponse = Response<Bytes>;

pub trait Handler: Send + Sync + 'static {
    fn handle(&self, req: PortableRequest) -> Pin<Box<dyn Future<Output = PortableResponse> + Send + '_>>;
}

pub struct Server {
    pub routes: Vec<(String, Arc<dyn Handler>)>,
}

impl Default for Server {
    fn default() -> Self {
        Self::new()
    }
}

impl Server {
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    pub fn route<H: Handler>(&mut self, path: &str, handler: H) {
        self.routes.push((path.to_string(), Arc::new(handler)));
    }
}

#[cfg(feature = "actix")]
pub mod actix_adapter {
    use super::{Handler, Server};
    use actix_web::{web, HttpRequest, HttpResponse};
    use std::sync::Arc;


    async fn actix_handler_wrapper(
        req: HttpRequest,
        body: web::Bytes,
        handler: web::Data<Arc<dyn Handler>>,
    ) -> HttpResponse {
        let method = http::Method::try_from(req.method().as_str()).unwrap();
        
        let mut builder = http::Request::builder()
            .method(method)
            .uri(req.uri().to_string());

        for (k, v) in req.headers() {
            builder = builder.header(k.as_str(), v.as_bytes());
        }

        let portable_req = builder.body(body.to_vec()).unwrap();
        let portable_res = handler.handle(portable_req).await;

        let mut res_builder = HttpResponse::build(
            actix_web::http::StatusCode::from_u16(portable_res.status().as_u16()).unwrap(),
        );

        for (k, v) in portable_res.headers() {
            res_builder.append_header((k.as_str(), v.as_bytes()));
        }

        res_builder.body(portable_res.into_body())
    }

    pub fn configure(server: &Server, cfg: &mut actix_web::web::ServiceConfig) {
        for (path, handler) in &server.routes {
            let handler_data = web::Data::new(Arc::clone(handler));
            cfg.service(
                web::resource(path)
                    .app_data(handler_data.clone())
                    .route(web::post().to(actix_handler_wrapper))
            );
        }
    }
}

#[cfg(feature = "lambda")]
pub mod lambda_adapter {
    use super::Server;
    use vercel_runtime::{run as vercel_run, AppState, Error, ResponseBody};
    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;
    use std::task::{Context, Poll};

    type HyperRequest = hyper::Request<hyper::body::Incoming>;
    type HyperResponse = hyper::Response<ResponseBody>;

    struct ThreadloomService {
        server: Arc<Server>,
    }

    impl Clone for ThreadloomService {
        fn clone(&self) -> Self {
            Self { server: Arc::clone(&self.server) }
        }
    }

    impl tower::Service<(AppState, HyperRequest)> for ThreadloomService {
        type Response = HyperResponse;
        type Error = Error;
        type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn call(&mut self, (_, req): (AppState, HyperRequest)) -> Self::Future {
            let server = Arc::clone(&self.server);
            Box::pin(async move {
                use http_body_util::BodyExt;

                let path = if let Some(route) = req.headers().get("x-threadloom-route") {
                    route.to_str().unwrap_or(req.uri().path()).to_string()
                } else {
                    req.uri().path().to_string()
                };

                let (parts, body) = req.into_parts();
                let bytes = body.collect().await
                    .map(|c| c.to_bytes().to_vec())
                    .unwrap_or_default();

                let found_handler = server.routes.iter()
                    .find(|(route_path, _)| route_path == &path)
                    .map(|(_, h)| Arc::clone(h));

                if let Some(h) = found_handler {
                    let portable_req = http::Request::from_parts(parts, bytes);
                    let res = h.handle(portable_req).await;
                    let (res_parts, res_bytes) = res.into_parts();
                    Ok(hyper::Response::from_parts(res_parts, ResponseBody::from(res_bytes)))
                } else {
                    Ok(hyper::Response::builder()
                        .status(404)
                        .body(ResponseBody::from("Not Found"))
                        .unwrap())
                }
            })
        }
    }

    pub async fn run(server: Server) -> Result<(), Error> {
        let service = ThreadloomService { server: Arc::new(server) };
        vercel_run(service).await
    }
}


#[cfg(not(feature = "lambda"))]
pub mod lambda_http {
    #[derive(Debug)]
    pub struct Error;
}

#[cfg(not(feature = "lambda"))]
pub mod tokio {
    pub mod runtime {
        pub struct Builder;
        impl Builder {
            pub fn new_current_thread() -> Self { Self }
            pub fn enable_all(&mut self) -> &mut Self { self }
            pub fn build(&mut self) -> Result<Runtime, ()> { Ok(Runtime) }
        }
        pub struct Runtime;
        impl Runtime {
            pub fn block_on<F>(&self, _f: F) -> Result<(), crate::lambda_http::Error> { Ok(()) }
        }
    }
}

#[cfg(not(feature = "lambda"))]
pub mod lambda_adapter {
    use super::Server;
    pub async fn run(_server: Server) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}
