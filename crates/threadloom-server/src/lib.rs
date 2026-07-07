#[cfg(feature = "lambda")]
pub use lambda_http;
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
    use lambda_http::{service_fn, Body, Error, Request, Response};
    use std::sync::Arc;

    pub async fn run(server: Server) -> Result<(), Error> {
        let server = Arc::new(server);
        let handler = service_fn(move |req: Request| {
            let server_clone = Arc::clone(&server);
            async move {
                let path = if let Some(route) = req.headers().get("x-threadloom-route") {
                    route.to_str().unwrap_or(req.uri().path()).to_string()
                } else {
                    req.uri().path().to_string()
                };
                let mut found_handler = None;
                for (route_path, handler) in &server_clone.routes {
                    if route_path == &path {
                        found_handler = Some(Arc::clone(handler));
                        break;
                    }
                }

                if let Some(h) = found_handler {
                    let (parts, body) = req.into_parts();
                    let bytes = match body {
                        Body::Text(t) => t.into_bytes(),
                        Body::Binary(b) => b,
                        Body::Empty => vec![],
                    };
                    let portable_req = http::Request::from_parts(parts, bytes);
                    let res = h.handle(portable_req).await;
                    let (parts, res_bytes) = res.into_parts();
                    
                    let body = match String::from_utf8(res_bytes.clone()) {
                        Ok(text) => Body::Text(text),
                        Err(_) => Body::Binary(res_bytes),
                    };
                    
                    Ok::<Response<Body>, Error>(Response::from_parts(
                        parts,
                        body,
                    ))
                } else {
                    Ok(Response::builder().status(404).body(Body::Empty).unwrap())
                }
            }
        });
        lambda_http::run(handler).await
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
    pub async fn run(_server: Server) -> Result<(), crate::lambda_http::Error> {
        Ok(())
    }
}
