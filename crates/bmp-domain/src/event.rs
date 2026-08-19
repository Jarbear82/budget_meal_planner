use crate::id::{ItemId, PackageId, PantryEntryId, PrePlannedMealId, RecipeId, ScheduledMealId, StoreId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainEvent {
    // Item events
    ItemCreated(ItemId),
    ItemUpdated(ItemId),
    ItemDeleted(ItemId),

    // Store & Package events
    StoreCreated(StoreId),
    StoreDeleted(StoreId),
    PackageCreated(PackageId),
    PackageUpdated(PackageId),
    PackageDeleted(PackageId),
    PackagesBulkUpdated(Vec<PackageId>),

    // Recipe events
    RecipeSaved(RecipeId),
    RecipeDeleted(RecipeId),

    // Meal & Schedule events
    PrePlannedMealSaved(PrePlannedMealId),
    MealScheduled(ScheduledMealId),
    MealConsumed(ScheduledMealId),

    // Pantry events
    PantryEntryAdded(PantryEntryId),
    PantryQuantityUpdated(PantryEntryId),
    PantryEntryDeleted(PantryEntryId),
    PantryBulkAdjusted,

    // Receipt & Shopping events
    ReceiptRecorded(String),
    ShoppingListPurchased,

    // System & Backup events
    DataImported,
    DatabaseReset,
}
