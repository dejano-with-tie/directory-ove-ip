use actix::prelude::*;
use actix_web::{App, get, HttpResponse, HttpServer, post, Responder, web};

#[get("/health")]
pub async fn health() -> impl Responder {
    HttpResponse::Ok()
}

#[post("/echo")]
pub async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}
