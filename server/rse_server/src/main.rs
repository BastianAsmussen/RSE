mod info;

use actix_web::App;
use actix_web::HttpServer;
use actix_web::Responder;
use actix_web::{get, web};

use crate::info::Info;

#[get("/")]
async fn handle_query(info: web::Query<Info>) -> impl Responder {
    let info = info.into_inner();

    let search_results = match info.search().await {
        Ok(results) => results,
        Err(err) => {
            return format!("{err}");
        }
    };

    format!("{search_results:#?}")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    HttpServer::new(|| App::new().service(handle_query))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
