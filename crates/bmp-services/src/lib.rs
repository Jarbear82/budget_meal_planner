pub mod analytics_service;
pub mod item_service;
pub mod meal_service;
pub mod notification_service;
pub mod pantry_service;
pub mod recipe_service;
pub mod shopping_service;

pub use analytics_service::*;
pub use item_service::*;
pub use meal_service::*;
pub use notification_service::*;
pub use pantry_service::*;
pub use recipe_service::*;
pub use shopping_service::*;

use bmp_storage::Storage;

#[derive(Clone)]
pub struct AppServices {
    pub storage: Storage,
    pub items: std::sync::Arc<ItemService>,
    pub recipes: std::sync::Arc<RecipeService>,
    pub meals: std::sync::Arc<MealService>,
    pub shopping: std::sync::Arc<ShoppingService>,
    pub pantry: std::sync::Arc<PantryService>,
    pub analytics: std::sync::Arc<AnalyticsService>,
    pub notification: std::sync::Arc<NotificationService>,
}

impl AppServices {
    pub fn new(storage: Storage) -> Self {
        Self {
            storage: storage.clone(),
            items: std::sync::Arc::new(ItemService::new(storage.clone())),
            recipes: std::sync::Arc::new(RecipeService::new(storage.clone())),
            meals: std::sync::Arc::new(MealService::new(storage.clone())),
            shopping: std::sync::Arc::new(ShoppingService::new(storage.clone())),
            pantry: std::sync::Arc::new(PantryService::new(storage.clone())),
            analytics: std::sync::Arc::new(AnalyticsService::new(storage.clone())),
            notification: std::sync::Arc::new(NotificationService::new(storage.clone())),
        }
    }
}
