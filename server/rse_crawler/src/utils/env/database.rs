use diesel::ConnectionResult;
use diesel_async::{AsyncConnection, AsyncMysqlConnection};

/// Get the database URL from the environment variables.
///
/// # Returns
///
/// * The database URL.
///
/// # Notes
///
/// * If the `DATABASE_URL` environment variable isn't set, the program will panic.
/// * The database URL is expected to be a valid `MySQL` URL.
pub fn get_database_url() -> String {
    std::env::var_os("DATABASE_URL")
        .expect("DATABASE_URL must be set")
        .to_str()
        .expect("DATABASE_URL must be valid UTF-8")
        .to_string()
}

/// Establish a connection to the database.
///
/// # Returns
///
/// * The database connection.
///
/// # Notes
///
/// * If the `DATABASE_URL` environment variable isn't set, the program will panic.
/// * The database URL is expected to be a valid `MySQL` URL.
/// * The database connection is expected to be a valid `MySQL` connection.
pub async fn establish_connection() -> ConnectionResult<AsyncMysqlConnection> {
    let database_url = get_database_url();

    AsyncMysqlConnection::establish(&database_url).await
}
