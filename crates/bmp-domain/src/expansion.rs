use crate::error::DomainError;
use crate::id::{ItemId, ItemOrRecipeId, RecipeId};
use crate::item::PurchaseMode;
use crate::recipe::Recipe;
use crate::units::Quantity;
use rust_decimal::Decimal;
use std::collections::{HashMap, HashSet};

pub fn expand_recipe(
    recipe_id: RecipeId,
    multiplier: Decimal,
    recipes: &HashMap<RecipeId, Recipe>,
    items: &HashMap<ItemId, crate::item::Item>,
    visited: &mut HashSet<RecipeId>,
) -> Result<Vec<(ItemId, Quantity)>, DomainError> {
    if visited.contains(&recipe_id) {
        // Prevent infinite recursion if cycle occurs
        return Ok(Vec::new());
    }

    visited.insert(recipe_id);

    let recipe = match recipes.get(&recipe_id) {
        Some(r) => r,
        None => return Ok(Vec::new()),
    };

    let mut requirements = Vec::new();

    for edge in &recipe.ingredients {
        let scaled_qty = Quantity {
            amount: edge.quantity.amount * multiplier,
            unit: edge.quantity.unit.clone(),
        };

        if edge.cycle_flag {
            // Edge has a cycle flag (e.g. sourdough starter), treat as base item if yield exists
            if let ItemOrRecipeId::Recipe(sub_recipe_id) = edge.target {
                if let Some(sub_recipe) = recipes.get(&sub_recipe_id) {
                    if let Some((yield_item_id, _)) = sub_recipe.yields.first() {
                        requirements.push((*yield_item_id, scaled_qty));
                        continue;
                    }
                }
            }
        }

        match edge.target {
            ItemOrRecipeId::Item(item_id) => {
                requirements.push((item_id, scaled_qty));
            }
            ItemOrRecipeId::Recipe(sub_recipe_id) => {
                let sub_recipe = recipes.get(&sub_recipe_id);
                let yield_item_id = sub_recipe.and_then(|r| r.yields.first().map(|y| y.0));

                let mode = yield_item_id
                    .and_then(|id| items.get(&id))
                    .map(|i| i.preferred_purchase_mode)
                    .unwrap_or(PurchaseMode::BuyFinished);

                if mode == PurchaseMode::BuyFinished && yield_item_id.is_some() {
                    // Buy as finished item
                    requirements.push((yield_item_id.unwrap(), scaled_qty));
                } else {
                    // Expand sub-recipe
                    let sub_results = expand_recipe(
                        sub_recipe_id,
                        scaled_qty.amount,
                        recipes,
                        items,
                        visited,
                    )?;
                    requirements.extend(sub_results);
                }
            }
        }
    }

    visited.remove(&recipe_id);

    Ok(requirements)
}
