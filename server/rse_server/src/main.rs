mod utils;

use actix_web::get;
use actix_web::App;
use actix_web::HttpResponse;
use actix_web::HttpServer;
use actix_web::Responder;
use log::info;

use utils::db_manager::{get_database_url, get_pool};

#[get("/")]
async fn root() -> impl Responder {
    HttpResponse::Ok().body("Hello, World!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // Establish database connection.
    info!("Connecting to the database...");

    let Ok(url) = get_database_url() else {
        panic!("Missing environment variables!");
    };
    let Ok(pool) = get_pool(&url).await else {
        panic!("Failed to connect to the database!");
    };

    info!("Successfully connected to the database!");

    HttpServer::new(|| App::new().service(root))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
