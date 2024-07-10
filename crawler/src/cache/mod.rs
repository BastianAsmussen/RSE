use anyhow::Result;
use redis::Client;

pub fn init() -> Result<Client> {
    let url = std::env::var("CACHE_URL").expect("`CACHE_URL` must be set!");
    let client = Client::open(url)?;

    Ok(client)
}
