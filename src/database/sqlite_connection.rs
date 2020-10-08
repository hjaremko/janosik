use crate::database::database_connection::DatabaseConnection;
use crate::BoxResult;
use rusqlite::{params, Connection};
use tracing::{debug, info, warn};

pub struct SQLiteConnection {
    connection: Option<Connection>,
}

impl DatabaseConnection for SQLiteConnection {
    fn new() -> Self {
        Self { connection: None }
    }

    fn connect(&mut self) -> BoxResult {
        if self.connection.is_some() {
            warn!("Database is already connected!");
            return Ok(());
        }

        info!("Database connected");
        self.connection = Some(Connection::open("janosik.db3")?);
        Ok(())
    }

    fn execute(&self, query: &str) -> BoxResult {
        debug!("Executing query: {}", query);

        self.connection
            .as_ref()
            .unwrap()
            .execute(query, params![])?;
        Ok(())
    }

    fn raw(&self) -> &Connection {
        &self.connection.as_ref().unwrap()
    }
}
