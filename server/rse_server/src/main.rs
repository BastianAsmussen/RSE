mod utils;

use actix_web::{get, HttpRequest, web};
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::Responder;
use actix_web::web::Path;
use log::{error, info};
use mysql_async::{Params, Pool};

use utils::db_manager::{get_pool};

#[get("/")]
async fn root() -> impl Responder {
    HttpResponse::Ok().body("Hello, World!")
}

#[get("/query/{query}")]
async fn query(path: Path<String>) -> impl Responder {
    let Ok(pool) = get_pool().await else {
        error!("Failed to connect to the database!");

        return HttpResponse::InternalServerError().body("Failed to connect to the database!");
    };

    let Ok(conn) = pool.get_conn().await else {
        error!("Failed to get a connection from the pool!");

        return HttpResponse::InternalServerError().body("Failed to connect to the database!");
    };

    info!("Successfully connected to the database!");

    let query = path.into_inner();
    info!("Query: {}", query);

    HttpResponse::Ok().body("Hello, World!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    HttpServer::new(|| App::new().service(root))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
