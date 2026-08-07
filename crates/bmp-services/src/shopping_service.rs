use bmp_domain::*;
use bmp_storage::Storage;
use rust_decimal::Decimal;
use std::collections::HashMap;

pub struct ShoppingService {
    storage: Storage,
}

impl ShoppingService {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub fn generate_shopping_list(
        &self,
        scheduled_meal_requirements: Vec<(ItemId, Quantity)>,
        selected_store_id: Option<StoreId>,
        tax_rate: Option<Decimal>,
    ) -> Result<ShoppingList, String> {
        let items_list = self.storage.get_all_items().map_err(|e| e.to_string())?;
        let items_map: HashMap<ItemId, Item> = items_list.into_iter().map(|i| (i.id, i)).collect();

        let mut packages_map = HashMap::new();
        for item_id in items_map.keys() {
            let pkgs = self.storage.get_packages_for_item(*item_id).map_err(|e| e.to_string())?;
            packages_map.insert(*item_id, pkgs);
        }

        let pantry_entries = self.storage.get_all_pantry_entries().map_err(|e| e.to_string())?;

        bmp_domain::shopping::generate_shopping_list(
            scheduled_meal_requirements,
            &items_map,
            &packages_map,
            &pantry_entries,
            selected_store_id,
            tax_rate,
        )
        .map_err(|e| e.to_string())
    }
}
