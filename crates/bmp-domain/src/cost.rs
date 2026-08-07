use crate::error::DomainError;
use crate::id::ItemId;
use crate::package::Package;
use crate::recipe::Recipe;
use rust_decimal::Decimal;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeCost {
    pub price_per_batch: Decimal,
    pub price_per_serving: Decimal,
}

pub fn calculate_recipe_cost(
    recipe: &Recipe,
    packages_map: &HashMap<ItemId, Vec<Package>>,
) -> Result<RecipeCost, DomainError> {
    let mut total_batch_cost = Decimal::ZERO;

    for edge in &recipe.ingredients {
        if let crate::id::ItemOrRecipeId::Item(item_id) = edge.target {
            if let Some(pkgs) = packages_map.get(&item_id) {
                if let Some(best_pkg) = pkgs.iter().min_by(|a, b| {
                    let cost_a = a.price / a.quantity.amount;
                    let cost_b = b.price / b.quantity.amount;
                    cost_a.partial_cmp(&cost_b).unwrap()
                }) {
                    let unit_cost = best_pkg.price / best_pkg.quantity.amount;
                    let ingredient_cost = unit_cost * edge.quantity.amount;
                    total_batch_cost += ingredient_cost;
                }
            }
        }
    }

    let price_per_serving = if recipe.servings > Decimal::ZERO {
        total_batch_cost / recipe.servings
    } else {
        total_batch_cost
    };

    Ok(RecipeCost {
        price_per_batch: total_batch_cost,
        price_per_serving,
    })
}
