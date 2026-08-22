use crate::error::DomainError;
use crate::id::{ItemId, RecipeId};
use crate::item::PurchaseMode;
use crate::recipe::Recipe;
use crate::units::Quantity;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MakeRecipeConfig {
    pub batches: Decimal,
    pub substitute_overrides: HashMap<ItemId, ItemId>,
    pub excluded_optionals: HashSet<usize>,
    pub selected_yield_item: Option<ItemId>,
    pub global_substitutes: Option<HashMap<ItemId, ItemId>>,
}

impl Default for MakeRecipeConfig {
    fn default() -> Self {
        Self {
            batches: Decimal::ONE,
            substitute_overrides: HashMap::new(),
            excluded_optionals: HashSet::new(),
            selected_yield_item: None,
            global_substitutes: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MakeRecipeExecution {
    pub ingredients_to_consume: Vec<(ItemId, Quantity)>,
    pub yields_produced: Vec<(ItemId, Quantity)>,
}

pub fn evaluate_make_recipe(
    recipe: &Recipe,
    config: &MakeRecipeConfig,
    items: &HashMap<ItemId, crate::item::Item>,
) -> Result<MakeRecipeExecution, DomainError> {
    evaluate_make_recipe_full(recipe, config, items, &HashMap::new())
}

pub fn evaluate_make_recipe_full(
    recipe: &Recipe,
    config: &MakeRecipeConfig,
    items: &HashMap<ItemId, crate::item::Item>,
    recipes: &HashMap<RecipeId, Recipe>,
) -> Result<MakeRecipeExecution, DomainError> {
    let mut visited = HashSet::new();
    evaluate_make_recipe_internal(recipe, config, items, recipes, &mut visited)
}

fn evaluate_make_recipe_internal(
    recipe: &Recipe,
    config: &MakeRecipeConfig,
    items: &HashMap<ItemId, crate::item::Item>,
    recipes: &HashMap<RecipeId, Recipe>,
    visited: &mut HashSet<RecipeId>,
) -> Result<MakeRecipeExecution, DomainError> {
    if config.batches <= Decimal::ZERO {
        return Err(DomainError::NegativeQuantity(config.batches));
    }

    if visited.contains(&recipe.id) {
        return Ok(MakeRecipeExecution {
            ingredients_to_consume: Vec::new(),
            yields_produced: Vec::new(),
        });
    }
    visited.insert(recipe.id);

    let mut ingredients_to_consume = Vec::new();

    for (idx, edge) in recipe.ingredients.iter().enumerate() {
        if !edge.required && config.excluded_optionals.contains(&idx) {
            continue;
        }

        let scaled_qty = Quantity {
            amount: edge.quantity.amount * config.batches,
            unit: edge.quantity.unit.clone(),
        };

        match edge.target {
            crate::id::ItemOrRecipeId::Item(item_id) => {
                let resolved_id = config
                    .substitute_overrides
                    .get(&item_id)
                    .copied()
                    .or(edge.per_recipe_substitute)
                    .or_else(|| {
                        config
                            .global_substitutes
                            .as_ref()
                            .and_then(|g| g.get(&item_id).copied())
                    })
                    .unwrap_or(item_id);

                ingredients_to_consume.push((resolved_id, scaled_qty));
            }
            crate::id::ItemOrRecipeId::Recipe(sub_recipe_id) => {
                if let Some(sub_recipe) = recipes.get(&sub_recipe_id)
                    && let Some((yield_id, yield_qty)) = sub_recipe.yields.first() {
                        let mode = items
                            .get(yield_id)
                            .map(|i| i.preferred_purchase_mode)
                            .unwrap_or(PurchaseMode::BuyFinished);

                        if mode == PurchaseMode::BuyFinished || edge.cycle_flag || visited.contains(&sub_recipe_id) {
                            let resolved_id = config
                                .substitute_overrides
                                .get(yield_id)
                                .copied()
                                .or_else(|| {
                                    config
                                        .global_substitutes
                                        .as_ref()
                                        .and_then(|g| g.get(yield_id).copied())
                                })
                                .unwrap_or(*yield_id);
                            ingredients_to_consume.push((resolved_id, scaled_qty));
                        } else {
                            let sub_batches = if yield_qty.amount > Decimal::ZERO {
                                (scaled_qty.amount / yield_qty.amount) * config.batches
                            } else {
                                config.batches
                            };
                            let sub_config = MakeRecipeConfig {
                                batches: sub_batches,
                                global_substitutes: config.global_substitutes.clone(),
                                ..Default::default()
                            };
                            if let Ok(sub_exec) = evaluate_make_recipe_internal(
                                sub_recipe,
                                &sub_config,
                                items,
                                recipes,
                                visited,
                            ) {
                                ingredients_to_consume.extend(sub_exec.ingredients_to_consume);
                            }
                        }
                    }
            }
        }
    }

    let mut yields_produced = Vec::new();
    for (yield_item_id, yield_qty) in &recipe.yields {
        if let Some(selected) = config.selected_yield_item
            && *yield_item_id != selected {
                continue;
            }
        let scaled_yield = Quantity {
            amount: yield_qty.amount * config.batches,
            unit: yield_qty.unit.clone(),
        };
        yields_produced.push((*yield_item_id, scaled_yield));
    }

    visited.remove(&recipe.id);

    Ok(MakeRecipeExecution {
        ingredients_to_consume,
        yields_produced,
    })
}
