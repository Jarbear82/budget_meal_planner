use crate::bridge::UnitBridge;
use crate::id::{ItemId, StoreId};
use crate::item::Item;
use crate::meal::{PrePlannedMeal, ScheduledMeal};
use crate::package::Package;
use crate::pantry::PantryEntry;
use crate::recipe::Recipe;
use crate::store::Store;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReceiptRecord {
    pub id: String,
    pub store_id: Option<StoreId>,
    pub total: Decimal,
    pub datetime: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatabaseBackup {
    pub schema_version: u32,
    pub exported_at: DateTime<Utc>,
    pub items: Vec<Item>,
    pub stores: Vec<Store>,
    pub packages: Vec<Package>,
    pub recipes: Vec<Recipe>,
    pub pre_planned_meals: Vec<PrePlannedMeal>,
    pub scheduled_meals: Vec<ScheduledMeal>,
    pub pantry_entries: Vec<PantryEntry>,
    pub unit_bridges: Vec<UnitBridge>,
    pub global_substitutes: Vec<(ItemId, ItemId)>,
    pub receipts: Vec<ReceiptRecord>,
}

impl DatabaseBackup {
    pub fn new(
        items: Vec<Item>,
        stores: Vec<Store>,
        packages: Vec<Package>,
        recipes: Vec<Recipe>,
        pre_planned_meals: Vec<PrePlannedMeal>,
        scheduled_meals: Vec<ScheduledMeal>,
        pantry_entries: Vec<PantryEntry>,
        unit_bridges: Vec<UnitBridge>,
        global_substitutes: Vec<(ItemId, ItemId)>,
        receipts: Vec<ReceiptRecord>,
    ) -> Self {
        Self {
            schema_version: 1,
            exported_at: Utc::now(),
            items,
            stores,
            packages,
            recipes,
            pre_planned_meals,
            scheduled_meals,
            pantry_entries,
            unit_bridges,
            global_substitutes,
            receipts,
        }
    }
}
