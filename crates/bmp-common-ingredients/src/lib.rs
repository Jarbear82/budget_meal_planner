use bmp_domain::*;
use bmp_storage::Storage;
use rust_decimal_macros::dec;
use std::collections::HashMap;

pub struct CommonIngredient {
    pub name: &'static str,
    pub density_g_per_ml: Option<rust_decimal::Decimal>,
    pub category: &'static str,
}

pub fn common_ingredients_list() -> Vec<CommonIngredient> {
    vec![
        CommonIngredient { name: "All-Purpose Flour", density_g_per_ml: Some(dec!(0.53)), category: "Baking" },
        CommonIngredient { name: "Granulated Sugar", density_g_per_ml: Some(dec!(0.85)), category: "Baking" },
        CommonIngredient { name: "Brown Sugar", density_g_per_ml: Some(dec!(0.82)), category: "Baking" },
        CommonIngredient { name: "Table Salt", density_g_per_ml: Some(dec!(1.20)), category: "Baking" },
        CommonIngredient { name: "Water", density_g_per_ml: Some(dec!(1.00)), category: "Pantry" },
        CommonIngredient { name: "Whole Milk", density_g_per_ml: Some(dec!(1.03)), category: "Dairy" },
        CommonIngredient { name: "Unsalted Butter", density_g_per_ml: Some(dec!(0.911)), category: "Dairy" },
        CommonIngredient { name: "Extra Virgin Olive Oil", density_g_per_ml: Some(dec!(0.92)), category: "Pantry" },
        CommonIngredient { name: "Vegetable Oil", density_g_per_ml: Some(dec!(0.92)), category: "Pantry" },
        CommonIngredient { name: "Large Eggs", density_g_per_ml: None, category: "Dairy" },
        CommonIngredient { name: "White Rice", density_g_per_ml: Some(dec!(0.85)), category: "Grains" },
        CommonIngredient { name: "Rolled Oats", density_g_per_ml: Some(dec!(0.41)), category: "Grains" },
        CommonIngredient { name: "Ground Beef 80/20", density_g_per_ml: None, category: "Meat" },
        CommonIngredient { name: "Boneless Chicken Breast", density_g_per_ml: None, category: "Meat" },
        CommonIngredient { name: "Garlic", density_g_per_ml: None, category: "Produce" },
        CommonIngredient { name: "Yellow Onion", density_g_per_ml: None, category: "Produce" },
        CommonIngredient { name: "Diced Tomatoes (Canned)", density_g_per_ml: Some(dec!(1.02)), category: "Pantry" },
        // Recipe Yield Items
        CommonIngredient { name: "Pancakes", density_g_per_ml: None, category: "Prepared Food" },
        CommonIngredient { name: "Bolognese Sauce", density_g_per_ml: Some(dec!(1.05)), category: "Prepared Food" },
        CommonIngredient { name: "Chicken Rice Bowl", density_g_per_ml: None, category: "Prepared Food" },
    ]
}

pub fn seed_common_ingredients(storage: &Storage) -> Result<usize, String> {
    let (items_count, _) = seed_common_data_if_not_exists(storage)?;
    Ok(items_count)
}

pub fn seed_common_data_if_not_exists(storage: &Storage) -> Result<(usize, usize), String> {
    let existing_items = storage.get_all_items().map_err(|e| e.to_string())?;
    let mut items_map: HashMap<String, Item> = existing_items
        .into_iter()
        .map(|item| (item.name.to_lowercase(), item))
        .collect();

    let mut items_added = 0;
    let list = common_ingredients_list();

    for ing in &list {
        let key = ing.name.to_lowercase();
        if !items_map.contains_key(&key) {
            let mut item = Item::new(ing.name).with_category(ing.category);
            if let Some(d) = ing.density_g_per_ml {
                if let Ok(den) = Density::new(d) {
                    item = item.with_density(den);
                }
            }
            if storage.insert_item(&item).is_ok() {
                items_map.insert(key, item);
                items_added += 1;
            }
        }
    }

    let existing_recipes = storage.get_all_recipes().map_err(|e| e.to_string())?;
    let existing_recipe_names: Vec<String> = existing_recipes
        .into_iter()
        .map(|r| r.name.to_lowercase())
        .collect();

    let mut recipes_added = 0;

    // Helper closure to lookup item id by name
    let get_item_id = |name: &str| -> Option<ItemId> {
        items_map.get(&name.to_lowercase()).map(|i| i.id)
    };

    // 1. Seed "Classic Fluffy Pancakes"
    if !existing_recipe_names.contains(&"classic fluffy pancakes".to_string()) {
        if let (Some(flour_id), Some(milk_id), Some(egg_id), Some(butter_id), Some(sugar_id), Some(pancake_id)) = (
            get_item_id("All-Purpose Flour"),
            get_item_id("Whole Milk"),
            get_item_id("Large Eggs"),
            get_item_id("Unsalted Butter"),
            get_item_id("Granulated Sugar"),
            get_item_id("Pancakes"),
        ) {
            let mut recipe = Recipe::new("Classic Fluffy Pancakes", dec!(4))
                .with_instructions("Whisk dry ingredients. Stir in milk, eggs, and melted butter. Cook on medium-high griddle until golden.")
                .with_meal_type("Breakfast");

            if let Ok(q) = Quantity::new(dec!(1), Unit::Each) {
                recipe = recipe.add_yield(pancake_id, q);
            }
            if let Ok(q) = Quantity::new(dec!(200), Unit::Gram) {
                recipe = recipe.add_ingredient(IngredientEdge::item(flour_id, q));
            }
            if let Ok(q) = Quantity::new(dec!(250), Unit::Milliliter) {
                recipe = recipe.add_ingredient(IngredientEdge::item(milk_id, q));
            }
            if let Ok(q) = Quantity::new(dec!(2), Unit::Each) {
                recipe = recipe.add_ingredient(IngredientEdge::item(egg_id, q));
            }
            if let Ok(q) = Quantity::new(dec!(30), Unit::Gram) {
                recipe = recipe.add_ingredient(IngredientEdge::item(butter_id, q));
            }
            if let Ok(q) = Quantity::new(dec!(25), Unit::Gram) {
                recipe = recipe.add_ingredient(IngredientEdge::item(sugar_id, q));
            }

            if storage.insert_recipe(&recipe).is_ok() {
                recipes_added += 1;
            }
        }
    }

    // 2. Seed "Hearty Spaghetti Bolognese"
    if !existing_recipe_names.contains(&"hearty spaghetti bolognese".to_string()) {
        if let (Some(beef_id), Some(tomato_id), Some(onion_id), Some(garlic_id), Some(oil_id), Some(bolognese_id)) = (
            get_item_id("Ground Beef 80/20"),
            get_item_id("Diced Tomatoes (Canned)"),
            get_item_id("Yellow Onion"),
            get_item_id("Garlic"),
            get_item_id("Extra Virgin Olive Oil"),
            get_item_id("Bolognese Sauce"),
        ) {
            let mut recipe = Recipe::new("Hearty Spaghetti Bolognese", dec!(4))
                .with_instructions("Sauté onions and garlic in olive oil. Brown ground beef thoroughly. Add diced tomatoes and simmer for 45 minutes.")
                .with_meal_type("Dinner");

            if let Ok(q) = Quantity::new(dec!(1), Unit::Each) {
                recipe = recipe.add_yield(bolognese_id, q);
            }
            if let Ok(q) = Quantity::new(dec!(450), Unit::Gram) {
                recipe = recipe.add_ingredient(IngredientEdge::item(beef_id, q));
            }
            if let Ok(q) = Quantity::new(dec!(400), Unit::Gram) {
                recipe = recipe.add_ingredient(IngredientEdge::item(tomato_id, q));
            }
            if let Ok(q) = Quantity::new(dec!(1), Unit::Each) {
                recipe = recipe.add_ingredient(IngredientEdge::item(onion_id, q));
            }
            if let Ok(q) = Quantity::new(dec!(2), Unit::Each) {
                recipe = recipe.add_ingredient(IngredientEdge::item(garlic_id, q));
            }
            if let Ok(q) = Quantity::new(dec!(15), Unit::Milliliter) {
                recipe = recipe.add_ingredient(IngredientEdge::item(oil_id, q));
            }

            if storage.insert_recipe(&recipe).is_ok() {
                recipes_added += 1;
            }
        }
    }

    // 3. Seed "Seared Chicken & Rice Bowl"
    if !existing_recipe_names.contains(&"seared chicken & rice bowl".to_string()) {
        if let (Some(chicken_id), Some(rice_id), Some(oil_id), Some(salt_id), Some(bowl_id)) = (
            get_item_id("Boneless Chicken Breast"),
            get_item_id("White Rice"),
            get_item_id("Vegetable Oil"),
            get_item_id("Table Salt"),
            get_item_id("Chicken Rice Bowl"),
        ) {
            let mut recipe = Recipe::new("Seared Chicken & Rice Bowl", dec!(2))
                .with_instructions("Season chicken breasts with salt and pan-sear in oil. Steam white rice until fluffy. Slice chicken and serve over rice.")
                .with_meal_type("Lunch");

            if let Ok(q) = Quantity::new(dec!(2), Unit::Each) {
                recipe = recipe.add_yield(bowl_id, q);
            }
            if let Ok(q) = Quantity::new(dec!(350), Unit::Gram) {
                recipe = recipe.add_ingredient(IngredientEdge::item(chicken_id, q));
            }
            if let Ok(q) = Quantity::new(dec!(180), Unit::Gram) {
                recipe = recipe.add_ingredient(IngredientEdge::item(rice_id, q));
            }
            if let Ok(q) = Quantity::new(dec!(15), Unit::Milliliter) {
                recipe = recipe.add_ingredient(IngredientEdge::item(oil_id, q));
            }
            if let Ok(q) = Quantity::new(dec!(5), Unit::Gram) {
                recipe = recipe.add_ingredient(IngredientEdge::item(salt_id, q));
            }

            if storage.insert_recipe(&recipe).is_ok() {
                recipes_added += 1;
            }
        }
    }

    Ok((items_added, recipes_added))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seed_common_data_if_not_exists_first_init_and_idempotency() {
        let storage = Storage::in_memory().unwrap();

        // 1. Initial seeding on empty database
        let (items_added, recipes_added) = seed_common_data_if_not_exists(&storage).unwrap();
        assert!(items_added > 0, "Should add common ingredients");
        assert_eq!(recipes_added, 3, "Should add 3 sample starter recipes");

        let items = storage.get_all_items().unwrap();
        let recipes = storage.get_all_recipes().unwrap();
        assert!(items.len() >= 20);
        assert_eq!(recipes.len(), 3);

        // 2. Second seeding attempt (idempotency check)
        let (items_readded, recipes_readded) = seed_common_data_if_not_exists(&storage).unwrap();
        assert_eq!(items_readded, 0, "No duplicate items should be added");
        assert_eq!(recipes_readded, 0, "No duplicate recipes should be added");
    }
}
