use crate::BoxResult;
use rusqlite::Connection;

pub trait DatabaseConnection {
    fn new() -> Self;
    fn connect(&mut self) -> BoxResult;
    fn execute(&self, query: &str) -> BoxResult;
    fn raw(&self) -> &Connection; // :/
}
