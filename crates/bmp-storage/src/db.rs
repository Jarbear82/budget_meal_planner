use crate::migrations::run_migrations;
use rusqlite::{Connection, Result};
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Storage {
    conn: Arc<Mutex<Connection>>,
}

impl Storage {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        run_migrations(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        run_migrations(&conn)?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn default_db_path() -> Result<std::path::PathBuf> {
        if let Ok(env_path) = std::env::var("BMP_DB_PATH") {
            if !env_path.trim().is_empty() {
                let path = std::path::PathBuf::from(env_path);
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                return Ok(path);
            }
        }

        let base_dir = dirs::data_dir()
            .or_else(dirs::home_dir)
            .unwrap_or_else(|| std::path::PathBuf::from("."));

        let app_dir = base_dir.join("budget_meal_planner");
        std::fs::create_dir_all(&app_dir).map_err(|e| {
            rusqlite::Error::ToSqlConversionFailure(Box::new(e))
        })?;

        Ok(app_dir.join("data.sqlite"))
    }

    pub fn open_default() -> Result<Self> {
        if std::env::var("BMP_IN_MEMORY").map(|v| v == "1").unwrap_or(false) {
            return Self::in_memory();
        }

        let db_path = Self::default_db_path()?;
        Self::open(db_path)
    }

    pub fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap()
    }

    pub fn with_transaction<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&rusqlite::Transaction) -> Result<T>,
    {
        let mut conn = self.conn();
        let tx = conn.transaction()?;
        let res = f(&tx)?;
        tx.commit()?;
        Ok(res)
    }
}
