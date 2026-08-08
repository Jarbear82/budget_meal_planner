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

    pub fn consume_pantry_item(
        &self,
        item_id: ItemId,
        mut amount: Decimal,
        unit: Unit,
    ) -> Result<(), String> {
        let mut entries = self.get_pantry()?;
        entries.retain(|e| e.item_id == item_id);
        entries.sort_by_key(|e| e.expiration);

        let items = self.storage.get_all_items().map_err(|e| e.to_string())?;
        let item = items.into_iter().find(|i| i.id == item_id);

        for entry in entries {
            if amount <= Decimal::ZERO {
                break;
            }

            let entry_amount_in_req_unit = if entry.quantity.unit == unit {
                Some(entry.quantity.amount)
            } else if let Ok(converted) = entry.quantity.convert_direct(&unit) {
                Some(converted.amount)
            } else if let Some(ref it) = item {
                if let (Some(bridge), density) = (&it.count_bridge, it.density.as_ref()) {
                    bridge.convert(&entry.quantity, &unit, density.cloned()).ok().map(|q| q.amount)
                } else if let Some(density) = &it.density {
                    density.convert(&entry.quantity, &unit).ok().map(|q| q.amount)
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(avail_in_req_unit) = entry_amount_in_req_unit {
                if avail_in_req_unit <= amount {
                    amount -= avail_in_req_unit;
                    self.storage.delete_pantry_entry(entry.id).map_err(|e| e.to_string())?;
                } else {
                    let fraction_used = amount / avail_in_req_unit;
                    let remaining_entry_amount = entry.quantity.amount * (Decimal::ONE - fraction_used);
                    amount = Decimal::ZERO;
                    self.storage
                        .update_pantry_quantity(entry.id, remaining_entry_amount)
                        .map_err(|e| e.to_string())?;
                }
            }
        }
        Ok(())
    }

    pub fn delete_pantry_entry(&self, entry_id: PantryEntryId) -> Result<(), String> {
        self.storage.delete_pantry_entry(entry_id).map_err(|e| e.to_string())
    }
}
