use crate::id::{ItemId, PantryEntryId};
use crate::units::Quantity;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PantryEntry {
    pub id: PantryEntryId,
    pub item_id: ItemId,
    pub quantity: Quantity,
    pub expiration: Option<NaiveDate>,
}

impl PantryEntry {
    pub fn new(item_id: ItemId, quantity: Quantity, expiration: Option<NaiveDate>) -> Self {
        Self {
            id: PantryEntryId::new(),
            item_id,
            quantity,
            expiration,
        }
    }
}
