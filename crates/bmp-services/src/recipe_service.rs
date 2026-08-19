use crate::error::{ServiceError, ServiceResult};
use crate::event_bus::EventBus;
use bmp_domain::*;
use bmp_storage::Storage;
use std::collections::HashMap;

pub struct RecipeService {
    storage: Storage,
    event_bus: EventBus,
}

impl RecipeService {
    pub fn new(storage: Storage, event_bus: EventBus) -> Self {
        Self { storage, event_bus }
    }

    pub fn new_with_storage(storage: Storage) -> Self {
        Self::new(storage, EventBus::default())
    }

    pub fn save_recipe(&self, recipe: Recipe) -> ServiceResult<Recipe> {
        let all_recipes = self.storage.get_all_recipes()?;
        let mut recipe_map: HashMap<RecipeId, Recipe> = all_recipes.into_iter().map(|r| (r.id, r)).collect();
        let recipe_id = recipe.id;
        recipe_map.insert(recipe.id, recipe);

        // Run cycle detection
        update_cycle_flags(&mut recipe_map);

        let updated_recipe = recipe_map.remove(&recipe_id).unwrap();
        self.storage.insert_recipe(&updated_recipe)?;
        self.event_bus.publish(DomainEvent::RecipeSaved(updated_recipe.id));
        Ok(updated_recipe)
    }

    pub fn list_recipes(&self) -> ServiceResult<Vec<Recipe>> {
        Ok(self.storage.get_all_recipes()?)
    }

    pub fn delete_recipe(&self, recipe_id: RecipeId) -> ServiceResult<()> {
        self.storage.delete_recipe(recipe_id)?;
        self.event_bus.publish(DomainEvent::RecipeDeleted(recipe_id));
        Ok(())
    }

    pub fn estimate_cost(&self, recipe_id: RecipeId) -> ServiceResult<RecipeCost> {
        let recipes = self.list_recipes()?;
        let recipes_map: HashMap<RecipeId, Recipe> = recipes.iter().map(|r| (r.id, r.clone())).collect();
        let recipe = recipes_map.get(&recipe_id)
            .ok_or_else(|| ServiceError::NotFound(format!("Recipe {} not found", recipe_id.0)))?;

        let items = self.storage.get_all_items()?;
        let items_map: HashMap<ItemId, Item> = items.iter().map(|i| (i.id, i.clone())).collect();

        let mut pkgs_map = HashMap::new();
        for item in &items {
            let pkgs = self.storage.get_packages_for_item(item.id)?;
            pkgs_map.insert(item.id, pkgs);
        }

        let mut visited = std::collections::HashSet::new();
        let cost = calculate_recipe_cost_full(
            recipe,
            &pkgs_map,
            Some(&items_map),
            Some(&recipes_map),
            &mut visited,
        )?;
        Ok(cost)
    }
}
