mod query;
mod utils;

use crate::query::Query;
use actix_web::App;
use actix_web::HttpServer;
use actix_web::Responder;
use actix_web::{get, web};

#[get("/")]
async fn handle_query(query: web::Query<Query>) -> impl Responder {
    format!("Hello, {}!", query.text)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    HttpServer::new(|| App::new().service(handle_query))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
