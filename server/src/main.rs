mod util;

use util::db::get_database_url;
use util::db::get_pool;

fn main() {
    // Establish database connection.
    let Ok(url) = get_database_url() else {
        panic!("Missing enviornment variables!");
    };
    let pool = get_pool(&url);
}
