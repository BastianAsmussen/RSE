mod info;
mod utils;

use actix_web::App;
use actix_web::HttpServer;
use actix_web::Responder;
use actix_web::{get, web};

use crate::info::Info;

#[get("/")]
async fn handle_query(info: web::Query<Info>) -> impl Responder {
    format!("{info:#?}")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    HttpServer::new(|| App::new().service(handle_query))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
