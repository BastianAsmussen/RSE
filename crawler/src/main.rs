use anyhow::Result;

mod db;
mod cache;

mod crawler;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let db = db::init().await.expect("Failed to connect to database!");
    let cache = cache::init().expect("Failed to connect to cache!");

    println!("Connected to databases!");

    Ok(())
}
