use rusqlite::{params, Connection, Result};
use std::fmt;
use tracing::{debug, info, warn};

pub trait DatabaseConnection {
    fn connect(&mut self) -> Result<()>;
    fn set_up(&self) -> Result<()>;
}

pub trait ProtipHandler {
    fn add_protip(&mut self, task_id: &str, content: &str) -> Result<()>;
    fn remove_protip(&mut self, protip_id: u32) -> Result<()>;
    fn get_protip(&mut self, task_id: &str) -> Vec<Protip>;
    fn get_tasks(&mut self) -> Vec<String>;
}

pub struct Database {
    conn: Option<Connection>,
}

impl Database {
    pub fn new() -> Self {
        Database { conn: None }
    }
}

impl DatabaseConnection for Database {
    fn connect(&mut self) -> Result<()> {
        self.conn = Some(Connection::open("janosik.db3")?);
        info!("Database connected");
        Ok(())
    }

    fn set_up(&self) -> Result<()> {
        self.conn.as_ref().unwrap().execute(
            "CREATE TABLE IF NOT EXISTS protip (
                  id              INTEGER PRIMARY KEY,
                  task_id         TEXT NOT NULL,
                  content         TEXT NOT NULL
                  )",
            params![],
        )?;

        info!("Database initialized");
        Ok(())
    }
}

impl ProtipHandler for Database {
    fn add_protip(&mut self, task_id: &str, content: &str) -> Result<()> {
        self.conn.as_ref().unwrap().execute(
            "INSERT INTO protip (task_id, content) VALUES (?1, ?2)",
            params![task_id, content],
        )?;

        info!("Adding protip: '{}' to task '{}'", content, task_id);
        Ok(())
    }

    fn remove_protip(&mut self, protip_id: u32) -> Result<()> {
        self.conn
            .as_ref()
            .unwrap()
            .execute("DELETE FROM protip WHERE id = ?1", params![protip_id])?;

        warn!("Removed protip: '{}'", protip_id);
        Ok(())
    }

    fn get_protip(&mut self, task_id: &str) -> Vec<Protip> {
        let mut stmt = self
            .conn
            .as_ref()
            .unwrap()
            .prepare("SELECT id, task_id, content FROM protip WHERE task_id = ?1")
            .unwrap();
        let protip_iter = stmt
            .query_map(params![task_id], |row| {
                Ok(Protip {
                    id: row.get(0)?,
                    task_id: row.get(1)?,
                    content: row.get(2)?,
                })
            })
            .unwrap();

        let mut protips = Vec::new();

        for protip in protip_iter {
            debug!("Found protip {:?}", protip.as_ref().unwrap());
            protips.push(protip.unwrap());
        }

        protips
    }

    fn get_tasks(&mut self) -> Vec<String> {
        let mut stmt = self
            .conn
            .as_ref()
            .unwrap()
            .prepare("SELECT DISTINCT task_id FROM protip")
            .unwrap();
        let protip_iter = stmt.query_map(params![], |row| Ok(row.get(0)?)).unwrap();

        let mut protips = Vec::new();

        for protip in protip_iter {
            debug!("Found protip {:?}", protip.as_ref().unwrap());
            protips.push(protip.unwrap());
        }

        protips
    }
}

#[derive(Debug)]
pub struct Protip {
    id: i32,
    task_id: String,
    content: String,
}

impl fmt::Display for Protip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}. {}", self.id, self.content)
    }
}
