use crate::db::Storage;
use bmp_domain::*;
use chrono::{DateTime, Utc};
use rusqlite::{params, Result};
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

impl Storage {
    // --- RECEIPTS & ANALYTICS ---

    pub fn insert_receipt(&self, store_id: Option<StoreId>, total: Decimal, datetime: DateTime<Utc>) -> Result<String> {
        let conn = self.conn();
        let id = Uuid::new_v4().to_string();
        let store_str = store_id.map(|s| s.0.to_string());

        conn.execute(
            "INSERT INTO receipts (id, store_id, total, datetime) VALUES (?1, ?2, ?3, ?4)",
            params![id, store_str, total.to_string(), datetime.to_rfc3339()],
        )?;
        Ok(id)
    }

    pub fn get_all_receipts(&self) -> Result<Vec<(String, Option<StoreId>, Decimal, DateTime<Utc>)>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT id, store_id, total, datetime FROM receipts")?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let store_str: Option<String> = row.get(1)?;
            let total_str: String = row.get(2)?;
            let dt_str: String = row.get(3)?;

            let store_id = store_str.and_then(|s| Uuid::from_str(&s).ok()).map(StoreId);
            let total = Decimal::from_str(&total_str).unwrap_or(Decimal::ZERO);
            let dt = DateTime::parse_from_rfc3339(&dt_str).unwrap_or_default().with_timezone(&Utc);

            Ok((id, store_id, total, dt))
        })?;

        let mut receipts = Vec::new();
        for r in rows {
            receipts.push(r?);
        }
        Ok(receipts)
    }
}
