use actix_web::{get, web, Responder, HttpResponse};

#[get("/api/hello")]
pub async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello from Actix Backend API!")
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(hello);
}
