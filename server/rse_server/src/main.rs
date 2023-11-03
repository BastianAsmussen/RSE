mod search;

use actix_web::App;
use actix_web::HttpServer;
use actix_web::Responder;
use actix_web::{get, web};
use common::errors::Error;
use log::info;

use crate::search::{Info, Output};

#[get("/")]
async fn handle_query(info: web::Query<Info>) -> impl Responder {
    let info = info.into_inner();

    let results = match info.search().await {
        Ok(search_results) => search_results,
        Err(err) => Output {
            query: info.query,
            error: Some(Error::Internal(err.to_string())),
            pages: None,
        },
    };

    web::Json(results)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let (ip, port) = common::utils::env::web::get_address();

    info!("Starting web server...");
    info!("Listening on \"http://{ip}:{port}\"...");
    HttpServer::new(|| App::new().service(handle_query))
        .bind((ip, port))?
        .run()
        .await
}
