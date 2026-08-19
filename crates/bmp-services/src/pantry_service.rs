use crate::error::{ServiceError, ServiceResult};
use crate::event_bus::EventBus;
use bmp_domain::*;
use bmp_storage::Storage;
use chrono::NaiveDate;
use rust_decimal::Decimal;

pub struct PantryService {
    storage: Storage,
    event_bus: EventBus,
}

impl PantryService {
    pub fn new(storage: Storage, event_bus: EventBus) -> Self {
        Self { storage, event_bus }
    }

    pub fn new_with_storage(storage: Storage) -> Self {
        Self::new(storage, EventBus::default())
    }

    pub fn add_pantry_entry(
        &self,
        item_id: ItemId,
        amount: Decimal,
        unit: Unit,
        expiration: Option<NaiveDate>,
    ) -> ServiceResult<PantryEntry> {
        let qty = Quantity::new(amount, unit)?;
        let entry = PantryEntry::new(item_id, qty, expiration);
        self.storage.insert_pantry_entry(&entry)?;
        self.event_bus.publish(DomainEvent::PantryEntryAdded(entry.id));
        Ok(entry)
    }

    pub fn get_pantry(&self) -> ServiceResult<Vec<PantryEntry>> {
        Ok(self.storage.get_all_pantry_entries()?)
    }

    pub fn update_quantity(&self, entry_id: PantryEntryId, new_amount: Decimal) -> ServiceResult<()> {
        if new_amount < Decimal::ZERO {
            return Err(ServiceError::Validation("Quantity cannot be negative".to_string()));
        }
        self.storage.update_pantry_quantity(entry_id, new_amount)?;
        self.event_bus.publish(DomainEvent::PantryQuantityUpdated(entry_id));
        Ok(())
    }

    pub fn bulk_pantry_adjust(&self, adjustments: &[(PantryEntryId, Decimal)]) -> ServiceResult<()> {
        for (_, amt) in adjustments {
            if *amt < Decimal::ZERO {
                return Err(ServiceError::Validation("Quantity cannot be negative".to_string()));
            }
        }
        self.storage.bulk_adjust_pantry(adjustments)?;
        self.event_bus.publish(DomainEvent::PantryBulkAdjusted);
        Ok(())
    }

    pub fn consume_pantry_item(
        &self,
        item_id: ItemId,
        mut amount: Decimal,
        unit: Unit,
    ) -> ServiceResult<()> {
        let mut entries = self.get_pantry()?;
        entries.retain(|e| e.item_id == item_id);
        entries.sort_by_key(|e| e.expiration);

        let items = self.storage.get_all_items()?;
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
                    self.storage.delete_pantry_entry(entry.id)?;
                    self.event_bus.publish(DomainEvent::PantryEntryDeleted(entry.id));
                } else {
                    let fraction_used = amount / avail_in_req_unit;
                    let remaining_entry_amount = entry.quantity.amount * (Decimal::ONE - fraction_used);
                    amount = Decimal::ZERO;
                    self.storage.update_pantry_quantity(entry.id, remaining_entry_amount)?;
                    self.event_bus.publish(DomainEvent::PantryQuantityUpdated(entry.id));
                }
            }
        }
        Ok(())
    }

    pub fn delete_pantry_entry(&self, entry_id: PantryEntryId) -> ServiceResult<()> {
        self.storage.delete_pantry_entry(entry_id)?;
        self.event_bus.publish(DomainEvent::PantryEntryDeleted(entry_id));
        Ok(())
    }
}
