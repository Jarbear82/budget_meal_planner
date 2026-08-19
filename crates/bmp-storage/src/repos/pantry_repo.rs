use super::parse_uuid;
use crate::db::Storage;
use bmp_domain::*;
use chrono::NaiveDate;
use rusqlite::{params, Result};
use rust_decimal::Decimal;
use std::str::FromStr;

impl Storage {
    // --- PANTRY CRUD & BATCH OPERATIONS ---

    pub fn insert_pantry_entry(&self, entry: &PantryEntry) -> Result<()> {
        let conn = self.conn();
        let unit_str = serde_json::to_string(&entry.quantity.unit).unwrap_or_default();
        let exp_str = entry.expiration.map(|d| d.to_string());

        conn.execute(
            "INSERT OR REPLACE INTO pantry_entries (id, item_id, quantity_amount, quantity_unit, expiration)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                entry.id.0.to_string(),
                entry.item_id.0.to_string(),
                entry.quantity.amount.to_string(),
                unit_str,
                exp_str
            ],
        )?;
        Ok(())
    }

    pub fn get_all_pantry_entries(&self) -> Result<Vec<PantryEntry>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT id, item_id, quantity_amount, quantity_unit, expiration FROM pantry_entries")?;
        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let item_id_str: String = row.get(1)?;
            let amount_str: String = row.get(2)?;
            let unit_str: String = row.get(3)?;
            let exp_str: Option<String> = row.get(4)?;

            let id = PantryEntryId(parse_uuid(&id_str)?);
            let item_id = ItemId(parse_uuid(&item_id_str)?);
            let amount = Decimal::from_str(&amount_str).unwrap_or(Decimal::ZERO);
            let unit: Unit = serde_json::from_str(&unit_str).unwrap_or(Unit::Gram);
            let expiration = exp_str.and_then(|s| NaiveDate::from_str(&s).ok());

            Ok(PantryEntry {
                id,
                item_id,
                quantity: Quantity { amount, unit },
                expiration,
            })
        })?;

        let mut entries = Vec::new();
        for r in rows {
            entries.push(r?);
        }
        Ok(entries)
    }

    pub fn update_pantry_quantity(&self, id: PantryEntryId, amount: Decimal) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "UPDATE pantry_entries SET quantity_amount = ?1 WHERE id = ?2",
            params![amount.to_string(), id.0.to_string()],
        )?;
        Ok(())
    }

    pub fn bulk_adjust_pantry(&self, adjustments: &[(PantryEntryId, Decimal)]) -> Result<()> {
        self.with_transaction(|tx| {
            let mut stmt = tx.prepare("UPDATE pantry_entries SET quantity_amount = ?1 WHERE id = ?2")?;
            for (id, amount) in adjustments {
                stmt.execute(params![amount.to_string(), id.0.to_string()])?;
            }
            Ok(())
        })
    }

    pub fn delete_pantry_entry(&self, id: PantryEntryId) -> Result<()> {
        let conn = self.conn();
        conn.execute("DELETE FROM pantry_entries WHERE id = ?1", params![id.0.to_string()])?;
        Ok(())
    }
}
