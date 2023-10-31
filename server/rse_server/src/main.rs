mod search;

use actix_web::App;
use actix_web::HttpServer;
use actix_web::Responder;
use actix_web::{get, web};

use crate::search::{Info, Output};

#[get("/")]
async fn handle_query(info: web::Query<Info>) -> impl Responder {
    let info = info.into_inner();

    let results = match info.search().await {
        Ok(search_results) => search_results,
        Err(err) => Output {
            query: info.query,
            pages: None,
            error: Some(error::Error::Internal(err.to_string())),
        },
    };

    web::Json(results)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    HttpServer::new(|| App::new().service(handle_query))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
