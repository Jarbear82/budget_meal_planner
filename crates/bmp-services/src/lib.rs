pub mod analytics_service;
pub mod backup_service;
pub mod error;
pub mod event_bus;
pub mod item_service;
pub mod meal_service;
pub mod notification_service;
pub mod pantry_service;
pub mod recipe_service;
pub mod shopping_service;

pub use analytics_service::*;
pub use backup_service::*;
pub use error::*;
pub use event_bus::*;
pub use item_service::*;
pub use meal_service::*;
pub use notification_service::*;
pub use pantry_service::*;
pub use recipe_service::*;
pub use shopping_service::*;

use bmp_storage::Storage;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppServices {
    pub storage: Storage,
    pub event_bus: EventBus,
    pub items: Arc<ItemService>,
    pub recipes: Arc<RecipeService>,
    pub meals: Arc<MealService>,
    pub shopping: Arc<ShoppingService>,
    pub pantry: Arc<PantryService>,
    pub analytics: Arc<AnalyticsService>,
    pub notification: Arc<NotificationService>,
    pub backup: Arc<BackupService>,
}

impl AppServices {
    pub fn new(storage: Storage) -> Self {
        Self::new_with_bus(storage, EventBus::default())
    }

    pub fn new_with_bus(storage: Storage, event_bus: EventBus) -> Self {
        Self {
            storage: storage.clone(),
            event_bus: event_bus.clone(),
            items: Arc::new(ItemService::new(storage.clone(), event_bus.clone())),
            recipes: Arc::new(RecipeService::new(storage.clone(), event_bus.clone())),
            meals: Arc::new(MealService::new(storage.clone(), event_bus.clone())),
            shopping: Arc::new(ShoppingService::new(storage.clone(), event_bus.clone())),
            pantry: Arc::new(PantryService::new(storage.clone(), event_bus.clone())),
            analytics: Arc::new(AnalyticsService::new(storage.clone(), event_bus.clone())),
            notification: Arc::new(NotificationService::new(storage.clone())),
            backup: Arc::new(BackupService::new(storage.clone(), event_bus)),
        }
    }
}
