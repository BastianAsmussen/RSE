use anyhow::Result;
use diesel::{Connection, PgConnection};
use diesel_async::{
    pooled_connection::{bb8::Pool, AsyncDieselConnectionManager},
    AsyncPgConnection,
};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use tracing::debug;

mod schema;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub async fn init() -> Result<Pool<AsyncPgConnection>> {
    let url = std::env::var("DATABASE_URL").expect("`DATABASE_URL` must be set!");

    // Run database migrations.
    {
        debug!("Connecting to database...");
        let mut conn = PgConnection::establish(&url)?;

        debug!("Running pending migrations...");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run pending database migrations!");
    }

    debug!("Creating database pool...");
    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(url);
    let pool = Pool::builder().build(config).await?;

    Ok(pool)
}
