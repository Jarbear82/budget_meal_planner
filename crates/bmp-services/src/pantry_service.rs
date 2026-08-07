use bmp_domain::*;
use bmp_storage::Storage;
use chrono::NaiveDate;
use rust_decimal::Decimal;

pub struct PantryService {
    storage: Storage,
}

impl PantryService {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub fn add_pantry_entry(
        &self,
        item_id: ItemId,
        amount: Decimal,
        unit: Unit,
        expiration: Option<NaiveDate>,
    ) -> Result<PantryEntry, String> {
        let qty = Quantity::new(amount, unit).map_err(|e| e.to_string())?;
        let entry = PantryEntry::new(item_id, qty, expiration);
        self.storage.insert_pantry_entry(&entry).map_err(|e| e.to_string())?;
        Ok(entry)
    }

    pub fn get_pantry(&self) -> Result<Vec<PantryEntry>, String> {
        self.storage.get_all_pantry_entries().map_err(|e| e.to_string())
    }

    pub fn update_quantity(&self, entry_id: PantryEntryId, new_amount: Decimal) -> Result<(), String> {
        if new_amount < Decimal::ZERO {
            return Err("Quantity cannot be negative".to_string());
        }
        self.storage.update_pantry_quantity(entry_id, new_amount).map_err(|e| e.to_string())
    }
}
