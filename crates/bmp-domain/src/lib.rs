pub mod bridge;
pub mod cost;
pub mod cycle_detection;
pub mod density;
pub mod error;
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

pub use bridge::*;
pub use cost::*;
pub use cycle_detection::*;
pub use density::*;
pub use error::*;
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
}
