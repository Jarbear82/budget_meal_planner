use rusqlite::Result;
use std::str::FromStr;
use uuid::Uuid;

pub(crate) fn parse_uuid(s: &str) -> Result<Uuid> {
    Uuid::from_str(s).map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))
}

pub mod analytics_repo;
pub mod backup_repo;
pub mod item_repo;
pub mod meal_repo;
pub mod pantry_repo;
pub mod recipe_repo;
