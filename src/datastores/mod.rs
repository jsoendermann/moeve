mod datastore;
mod sqlite;

use anyhow::Result;

use crate::datastores::sqlite::SqliteDatastore;
pub use datastore::Datastore;

pub fn get_datastore() -> Result<impl Datastore> {
    SqliteDatastore::new("moeve.sqlite3")
}
