use crate::error::ServiceResult;
use crate::event_bus::EventBus;
use bmp_domain::{DatabaseBackup, DomainEvent};
use bmp_storage::Storage;
use std::path::Path;

pub struct BackupService {
    storage: Storage,
    event_bus: EventBus,
}

impl BackupService {
    pub fn new(storage: Storage, event_bus: EventBus) -> Self {
        Self { storage, event_bus }
    }

    /// Exports all database tables to a structured JSON string.
    pub fn export_json(&self) -> ServiceResult<String> {
        let backup = self.storage.export_all()?;
        let json_str = serde_json::to_string_pretty(&backup).map_err(|e| crate::error::ServiceError::Serialization(e.to_string()))?;
        Ok(json_str)
    }

    /// Exports all database tables to a JSON file at the specified path.
    pub fn export_to_file<P: AsRef<Path>>(&self, path: P) -> ServiceResult<()> {
        let json_str = self.export_json()?;
        std::fs::write(path, json_str)?;
        Ok(())
    }

    /// Imports all database tables from a structured JSON string.
    pub fn import_json(&self, json_str: &str, overwrite: bool) -> ServiceResult<()> {
        let backup: DatabaseBackup = serde_json::from_str(json_str).map_err(|e| crate::error::ServiceError::Serialization(e.to_string()))?;
        self.storage.import_all(&backup, overwrite)?;
        self.event_bus.publish(DomainEvent::DataImported);
        Ok(())
    }

    /// Imports all database tables from a JSON file at the specified path.
    pub fn import_from_file<P: AsRef<Path>>(&self, path: P, overwrite: bool) -> ServiceResult<()> {
        let json_str = std::fs::read_to_string(path)?;
        self.import_json(&json_str, overwrite)
    }
}
