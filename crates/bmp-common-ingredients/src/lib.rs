use bmp_domain::*;
use bmp_storage::Storage;
use rust_decimal_macros::dec;

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
    ]
}

pub fn seed_common_ingredients(storage: &Storage) -> Result<usize, String> {
    let list = common_ingredients_list();
    let mut count = 0;
    for ing in &list {
        let mut item = Item::new(ing.name).with_category(ing.category);
        if let Some(d) = ing.density_g_per_ml {
            if let Ok(den) = Density::new(d) {
                item = item.with_density(den);
            }
        }
        if storage.insert_item(&item).is_ok() {
            count += 1;
        }
    }
    Ok(count)
}
