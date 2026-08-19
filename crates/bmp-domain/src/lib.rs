pub mod backup;
pub mod bridge;
pub mod cost;
pub mod cycle_detection;
pub mod density;
pub mod error;
pub mod event;
pub mod expansion;
pub mod id;
pub mod item;
pub mod make_recipe;
pub mod meal;
pub mod package;
pub mod pantry;
pub mod recipe;
pub mod shopping;
pub mod store;
pub mod substitute;
pub mod units;

pub use backup::*;
pub use bridge::*;
pub use cost::*;
pub use cycle_detection::*;
pub use density::*;
pub use error::*;
pub use event::*;
pub use expansion::*;
pub use id::*;
pub use item::*;
pub use make_recipe::*;
pub use meal::*;
pub use package::*;
pub use pantry::*;
pub use recipe::*;
pub use shopping::*;
pub use store::*;
pub use substitute::*;
pub use units::*;

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    use std::collections::HashMap;

    #[test]
    fn test_unit_conversions_mass_and_volume() {
        let kg = Quantity::new(dec!(2.5), Unit::Kilogram).unwrap();
        let grams = kg.convert_direct(&Unit::Gram).unwrap();
        assert_eq!(grams.amount, dec!(2500));

        let lb = Quantity::new(dec!(1), Unit::Pound).unwrap();
        let g = lb.convert_direct(&Unit::Gram).unwrap();
        assert_eq!(g.amount, dec!(453.59237));

        let cups = Quantity::new(dec!(2), Unit::Cup).unwrap();
        let ml = cups.convert_direct(&Unit::Milliliter).unwrap();
        assert_eq!(ml.amount, dec!(473.176473));
    }

    #[test]
    fn test_density_cross_conversions() {
        // Water density 1.0 g/ml
        let density = Density::new(dec!(1.0)).unwrap();
        let ml = Quantity::new(dec!(500), Unit::Milliliter).unwrap();
        let mass = density.convert(&ml, &Unit::Gram).unwrap();
        assert_eq!(mass.amount, dec!(500));

        // Flour density ~0.53 g/ml
        let flour_density = Density::new(dec!(0.53)).unwrap();
        let flour_cups = Quantity::new(dec!(1), Unit::Cup).unwrap(); // 236.588 ml
        let flour_g = flour_density.convert(&flour_cups, &Unit::Gram).unwrap();
        assert_eq!(flour_g.amount, dec!(125.391765345));
    }

    #[test]
    fn test_recipe_cycle_detection() {
        let mut recipes = HashMap::new();
        let r1_id = RecipeId::new();
        let r2_id = RecipeId::new();

        let mut r1 = Recipe::new("Sourdough Starter Batch A", dec!(1));
        r1.id = r1_id;
        r1.ingredients.push(IngredientEdge::recipe(r2_id, Quantity::new(dec!(100), Unit::Gram).unwrap()));

        let mut r2 = Recipe::new("Sourdough Starter Batch B", dec!(1));
        r2.id = r2_id;
        r2.ingredients.push(IngredientEdge::recipe(r1_id, Quantity::new(dec!(100), Unit::Gram).unwrap()));

        recipes.insert(r1_id, r1);
        recipes.insert(r2_id, r2);

        update_cycle_flags(&mut recipes);

        let updated_r1 = recipes.get(&r1_id).unwrap();
        assert!(updated_r1.ingredients[0].cycle_flag);
    }

    #[test]
    fn test_mixed_unit_shopping_and_pantry_subtraction() {
        let store_id = StoreId::new();
        let item = Item::new("Flour");
        let item_id = item.id;

        let mut items_map = HashMap::new();
        items_map.insert(item_id, item);

        let pkg = Package::new(item_id, store_id, Quantity::new(dec!(5), Unit::Pound).unwrap(), dec!(3.99));
        let mut packages_map = HashMap::new();
        packages_map.insert(item_id, vec![pkg]);

        // Requirement: 1 Kilogram of Flour (~2.20462 lb)
        let reqs = vec![(item_id, Quantity::new(dec!(1), Unit::Kilogram).unwrap())];

        // Pantry: 453.59237 Grams of Flour (1 lb)
        let pantry_entry = PantryEntry::new(
            item_id,
            Quantity::new(dec!(453.59237), Unit::Gram).unwrap(),
            None,
        );

        let shopping_list = generate_shopping_list(
            reqs,
            &items_map,
            &packages_map,
            &[pantry_entry],
            None,
            None,
        ).unwrap();

        assert_eq!(shopping_list.items.len(), 1);
        // After subtracting ~1 lb from ~2.20 lb requirement, remaining requirement ~1.20 lb.
        // Requires 1 package of 5 lb.
        assert_eq!(shopping_list.items[0].package_count, 1);
    }

    #[test]
    fn test_item_nutrition_and_dietary_flags() {
        let mut item = Item::new("Almond Milk")
            .with_dietary_flag(DietaryFlag::GlutenFree)
            .with_dietary_flag(DietaryFlag::DairyFree)
            .with_dietary_flag(DietaryFlag::Vegan);

        let nutrition = NutritionalInfo {
            calories: Some(dec!(30)),
            protein_g: Some(dec!(1.0)),
            net_carbs_g: Some(dec!(1.0)),
            fat_g: Some(dec!(2.5)),
            fiber_g: Some(dec!(0.5)),
            sodium_mg: Some(dec!(170)),
        };
        item = item.with_nutrition(nutrition);

        assert_eq!(item.dietary_flags.len(), 3);
        assert!(item.dietary_flags.contains(&DietaryFlag::GlutenFree));
        assert!(item.dietary_flags.contains(&DietaryFlag::DairyFree));
        assert!(item.dietary_flags.contains(&DietaryFlag::Vegan));
        assert_eq!(item.nutrition.as_ref().unwrap().calories, Some(dec!(30)));
    }

    #[test]
    fn test_diamond_recipe_cost_calculation() {
        use std::collections::HashSet;

        let tomato = Item::new("Tomato");
        let mut packages_map = HashMap::new();
        let store_id = StoreId::new();
        packages_map.insert(
            tomato.id,
            vec![Package::new(
                tomato.id,
                store_id,
                Quantity::new(dec!(100), Unit::Gram).unwrap(),
                dec!(10),
            )],
        );

        // Sub-recipe: "Tomato Sauce" (uses 100g Tomato, costing $10)
        let mut sauce_recipe = Recipe::new("Tomato Sauce", dec!(1));
        sauce_recipe = sauce_recipe.add_yield(tomato.id, Quantity::new(dec!(1), Unit::Each).unwrap());
        sauce_recipe = sauce_recipe.add_ingredient(IngredientEdge::item(
            tomato.id,
            Quantity::new(dec!(100), Unit::Gram).unwrap(),
        ));

        // Parent recipe: "Pizza Deluxe" (uses Tomato Sauce TWICE: base sauce + dip cup)
        let mut pizza_recipe = Recipe::new("Pizza Deluxe", dec!(1));
        pizza_recipe = pizza_recipe.add_ingredient(IngredientEdge::recipe(
            sauce_recipe.id,
            Quantity::new(dec!(1), Unit::Each).unwrap(),
        ));
        pizza_recipe = pizza_recipe.add_ingredient(IngredientEdge::recipe(
            sauce_recipe.id,
            Quantity::new(dec!(1), Unit::Each).unwrap(),
        ));

        let mut recipes_map = HashMap::new();
        recipes_map.insert(sauce_recipe.id, sauce_recipe.clone());
        recipes_map.insert(pizza_recipe.id, pizza_recipe.clone());

        let mut visited = HashSet::new();
        let cost = calculate_recipe_cost_full(
            &pizza_recipe,
            &packages_map,
            None,
            Some(&recipes_map),
            &mut visited,
        )
        .unwrap();

        // 2 * $10 = $20 total batch cost
        assert_eq!(cost.price_per_batch, dec!(20));
    }
}
