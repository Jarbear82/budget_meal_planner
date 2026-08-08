use bmp_domain::*;
use bmp_storage::Storage;
use std::collections::HashMap;

pub struct RecipeService {
    storage: Storage,
}

impl RecipeService {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub fn save_recipe(&self, recipe: Recipe) -> Result<Recipe, String> {
        let all_recipes = self.storage.get_all_recipes().map_err(|e| e.to_string())?;
        let mut recipe_map: HashMap<RecipeId, Recipe> = all_recipes.into_iter().map(|r| (r.id, r)).collect();
        recipe_map.insert(recipe.id, recipe.clone());

        // Run cycle detection
        update_cycle_flags(&mut recipe_map);

        let updated_recipe = recipe_map.remove(&recipe.id).unwrap();
        self.storage.insert_recipe(&updated_recipe).map_err(|e| e.to_string())?;
        Ok(updated_recipe)
    }

    pub fn list_recipes(&self) -> Result<Vec<Recipe>, String> {
        self.storage.get_all_recipes().map_err(|e| e.to_string())
    }

    pub fn estimate_cost(&self, recipe_id: RecipeId) -> Result<RecipeCost, String> {
        let recipes = self.list_recipes()?;
        let recipes_map: HashMap<RecipeId, Recipe> = recipes.iter().map(|r| (r.id, r.clone())).collect();
        let recipe = recipes_map.get(&recipe_id)
            .ok_or_else(|| "Recipe not found".to_string())?;

        let items = self.storage.get_all_items().map_err(|e| e.to_string())?;
        let items_map: HashMap<ItemId, Item> = items.iter().map(|i| (i.id, i.clone())).collect();

        let mut pkgs_map = HashMap::new();
        for item in &items {
            let pkgs = self.storage.get_packages_for_item(item.id).map_err(|e| e.to_string())?;
            pkgs_map.insert(item.id, pkgs);
        }

        let mut visited = std::collections::HashSet::new();
        calculate_recipe_cost_full(
            recipe,
            &pkgs_map,
            Some(&items_map),
            Some(&recipes_map),
            &mut visited,
        )
        .map_err(|e| e.to_string())
    }
}
