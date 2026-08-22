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
    if !visited.insert(recipe_id) {
        return Ok(Vec::new());
    }

    let result = expand_recipe_inner(recipe_id, multiplier, recipes, items, visited);
    visited.remove(&recipe_id);
    result
}

fn expand_recipe_inner(
    recipe_id: RecipeId,
    multiplier: Decimal,
    recipes: &HashMap<RecipeId, Recipe>,
    items: &HashMap<ItemId, crate::item::Item>,
    visited: &mut HashSet<RecipeId>,
) -> Result<Vec<(ItemId, Quantity)>, DomainError> {
    let Some(recipe) = recipes.get(&recipe_id) else {
        return Ok(Vec::new());
    };

    let mut requirements = Vec::new();

    for edge in &recipe.ingredients {
        let scaled_qty = Quantity {
            amount: edge.quantity.amount * multiplier,
            unit: edge.quantity.unit.clone(),
        };

        match edge.target {
            ItemOrRecipeId::Item(item_id) => {
                requirements.push((item_id, scaled_qty));
            }
            ItemOrRecipeId::Recipe(sub_recipe_id) => {
                let sub_recipe = recipes.get(&sub_recipe_id);
                let yield_item_id = sub_recipe.and_then(|r| r.yields.first().map(|y| y.0));

                let purchase_mode = yield_item_id
                    .and_then(|id| items.get(&id))
                    .map(|i| i.preferred_purchase_mode)
                    .unwrap_or_default();

                let buy_as_finished = edge.cycle_flag || purchase_mode == PurchaseMode::BuyFinished;

                if buy_as_finished && let Some(yield_id) = yield_item_id {
                    requirements.push((yield_id, scaled_qty));
                } else if let Some(sub_r) = sub_recipe {
                    let sub_multiplier = sub_r.scale_multiplier(&scaled_qty);
                    let sub_results =
                        expand_recipe(sub_recipe_id, sub_multiplier, recipes, items, visited)?;
                    requirements.extend(sub_results);
                }
            }
        }
    }

    Ok(requirements)
}
