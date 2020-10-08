pub mod database_connection;
pub mod protip_handler;
pub mod sqlite_connection;

use crate::database::database_connection::DatabaseConnection;
use crate::BoxResult;

use tokio::sync::Mutex;

pub struct Database<T: DatabaseConnection> {
    mutex: Mutex<T>,
}

impl<T: DatabaseConnection> Database<T> {
    pub fn new() -> Self {
        Database {
            mutex: Mutex::new(T::new()),
        }
    }

    pub async fn connect(&self) -> BoxResult {
        let mut db = self.mutex.lock().await;
        db.connect()
    }

    async fn execute(&self, query: &str) -> BoxResult {
        let db = self.mutex.lock().await;
        db.execute(query)
    }
}
