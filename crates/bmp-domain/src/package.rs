use crate::id::{ItemId, PackageId, StoreId};
use crate::units::Quantity;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Package {
    pub id: PackageId,
    pub item_id: ItemId,
    pub store_id: StoreId,
    pub quantity: Quantity,
    pub price: Decimal,
    pub last_seen: Option<DateTime<Utc>>,
    pub is_preferred: bool,
}

impl Package {
    pub fn new(
        item_id: ItemId,
        store_id: StoreId,
        quantity: Quantity,
        price: Decimal,
    ) -> Self {
        Self {
            id: PackageId::new(),
            item_id,
            store_id,
            quantity,
            price,
            last_seen: Some(Utc::now()),
            is_preferred: false,
        }
    }

    pub fn with_preferred(mut self, preferred: bool) -> Self {
        self.is_preferred = preferred;
        self
    }
}
