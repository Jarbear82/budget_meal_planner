use crate::error::DomainError;
use crate::id::{ItemId, ItemOrRecipeId, RecipeId};
use crate::item::Item;
use crate::package::Package;
use crate::recipe::Recipe;
use rust_decimal::Decimal;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecipeCost {
    pub price_per_batch: Decimal,
    pub price_per_serving: Decimal,
}

pub fn calculate_recipe_cost(
    recipe: &Recipe,
    packages_map: &HashMap<ItemId, Vec<Package>>,
) -> Result<RecipeCost, DomainError> {
    let mut visited = HashSet::new();
    calculate_recipe_cost_full(recipe, packages_map, None, None, &mut visited)
}

pub fn calculate_recipe_cost_with_items(
    recipe: &Recipe,
    packages_map: &HashMap<ItemId, Vec<Package>>,
    items_map: Option<&HashMap<ItemId, Item>>,
) -> Result<RecipeCost, DomainError> {
    let mut visited = HashSet::new();
    calculate_recipe_cost_full(recipe, packages_map, items_map, None, &mut visited)
}

pub fn calculate_recipe_cost_full(
    recipe: &Recipe,
    packages_map: &HashMap<ItemId, Vec<Package>>,
    items_map: Option<&HashMap<ItemId, Item>>,
    recipes_map: Option<&HashMap<RecipeId, Recipe>>,
    visited: &mut HashSet<RecipeId>,
) -> Result<RecipeCost, DomainError> {
    if visited.contains(&recipe.id) {
        return Ok(RecipeCost {
            price_per_batch: Decimal::ZERO,
            price_per_serving: Decimal::ZERO,
        });
    }
    visited.insert(recipe.id);

    let mut total_batch_cost = Decimal::ZERO;

    for edge in &recipe.ingredients {
        match edge.target {
            ItemOrRecipeId::Item(item_id) => {
                if let Some(pkgs) = packages_map.get(&item_id) {
                    if let Some(best_pkg) = pkgs.iter().min_by(|a, b| {
                        let cost_a = a.price / a.quantity.amount;
                        let cost_b = b.price / b.quantity.amount;
                        cost_a.cmp(&cost_b)
                    }) {
                        let item_opt = items_map.and_then(|m| m.get(&item_id));
                        let converted_qty = if edge.quantity.unit == best_pkg.quantity.unit {
                            Ok(edge.quantity.clone())
                        } else if let Some(item) = item_opt {
                            item.convert_quantity(&edge.quantity, &best_pkg.quantity.unit)
                        } else {
                            edge.quantity.convert_direct(&best_pkg.quantity.unit)
                        };

                        let req_amount = converted_qty
                            .map(|q| q.amount)
                            .unwrap_or(edge.quantity.amount);

                        let unit_cost = best_pkg.price / best_pkg.quantity.amount;
                        let ingredient_cost = unit_cost * req_amount;
                        total_batch_cost += ingredient_cost;
                    }
                }
            }
            ItemOrRecipeId::Recipe(sub_recipe_id) => {
                if let Some(sub_recipes) = recipes_map {
                    if let Some(sub_recipe) = sub_recipes.get(&sub_recipe_id) {
                        if let Ok(sub_cost) = calculate_recipe_cost_full(
                            sub_recipe,
                            packages_map,
                            items_map,
                            recipes_map,
                            visited,
                        ) {
                            let yield_qty = sub_recipe
                                .yields
                                .first()
                                .map(|y| y.1.amount)
                                .unwrap_or(Decimal::ONE);
                            let multiplier = if yield_qty > Decimal::ZERO {
                                edge.quantity.amount / yield_qty
                            } else {
                                Decimal::ONE
                            };
                            total_batch_cost += sub_cost.price_per_batch * multiplier;
                        }
                    }
                }
            }
        }
    }

    let price_per_serving = if recipe.servings > Decimal::ZERO {
        total_batch_cost / recipe.servings
    } else {
        total_batch_cost
    };

    visited.remove(&recipe.id);

    Ok(RecipeCost {
        price_per_batch: total_batch_cost,
        price_per_serving,
    })
}
