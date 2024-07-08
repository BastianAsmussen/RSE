mod crawler;

use anyhow::Result;

mod db;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt::init();

    let db = db::init().await.expect("Failed to connect to database!");
    println!("Connected to database!");

    Ok(())
}
