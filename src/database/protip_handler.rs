use crate::database::database_connection::DatabaseConnection;
use crate::database::Database;
use crate::BoxResult;
use async_trait::async_trait;
use core::result::Result::Ok;
use rusqlite::params;
use std::fmt;
use tracing::{debug, info, warn};

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

#[async_trait]
pub trait ProtipHandler {
    async fn set_up_protip_table(&self) -> BoxResult;
    async fn add_protip(&self, task_id: &str, content: &str) -> BoxResult;
    async fn remove_protip(&self, protip_id: u32) -> BoxResult;
    async fn get_protip(&self, task_id: &str) -> Vec<Protip>;
    async fn get_tasks(&self) -> Vec<String>;
}

#[async_trait]
impl<T: DatabaseConnection + std::marker::Send> ProtipHandler for Database<T> {
    async fn set_up_protip_table(&self) -> BoxResult {
        self.execute(
            "CREATE TABLE IF NOT EXISTS protip (
                  id              INTEGER PRIMARY KEY,
                  task_id         TEXT NOT NULL,
                  content         TEXT NOT NULL
                  )",
        )
        .await?;

        info!("Protip database initialized");
        Ok(())
    }

    async fn add_protip(&self, task_id: &str, content: &str) -> BoxResult {
        self.execute(&format!(
            "INSERT INTO protip (task_id, content) VALUES (\"{}\", \"{}\")",
            task_id, content
        ))
        .await?;

        info!("Added protip: '{}' to task '{}'", content, task_id);
        Ok(())
    }

    async fn remove_protip(&self, protip_id: u32) -> BoxResult {
        self.execute(&format!("DELETE FROM protip WHERE id = {}", protip_id))
            .await?;

        warn!("Removed protip: '{}'", protip_id);
        Ok(())
    }

    async fn get_protip(&self, task_id: &str) -> Vec<Protip> {
        let db = self.mutex.lock().await;
        let conn = db.raw();
        let mut stmt = conn
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

    async fn get_tasks(&self) -> Vec<String> {
        let db = self.mutex.lock().await;
        let conn = db.raw();

        let mut stmt = conn.prepare("SELECT DISTINCT task_id FROM protip").unwrap();
        let protip_iter = stmt.query_map(params![], |row| Ok(row.get(0)?)).unwrap();

        let mut protips = Vec::new();

        for protip in protip_iter {
            debug!("Found protip {:?}", protip.as_ref().unwrap());
            protips.push(protip.unwrap());
        }

        protips
    }
}
