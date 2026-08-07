use crate::error::DomainError;
use crate::id::ItemId;
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
}

impl Default for MakeRecipeConfig {
    fn default() -> Self {
        Self {
            batches: Decimal::ONE,
            substitute_overrides: HashMap::new(),
            excluded_optionals: HashSet::new(),
            selected_yield_item: None,
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
    _items: &HashMap<ItemId, crate::item::Item>,
) -> Result<MakeRecipeExecution, DomainError> {
    if config.batches <= Decimal::ZERO {
        return Err(DomainError::NegativeQuantity(config.batches));
    }

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
                    .unwrap_or(item_id);

                ingredients_to_consume.push((resolved_id, scaled_qty));
            }
            crate::id::ItemOrRecipeId::Recipe(_) => {
                // Nested sub-recipes: if evaluated directly here, handle as item or sub-expansion
            }
        }
    }

    let mut yields_produced = Vec::new();
    for (yield_item_id, yield_qty) in &recipe.yields {
        if let Some(selected) = config.selected_yield_item {
            if *yield_item_id != selected {
                continue;
            }
        }
        let scaled_yield = Quantity {
            amount: yield_qty.amount * config.batches,
            unit: yield_qty.unit.clone(),
        };
        yields_produced.push((*yield_item_id, scaled_yield));
    }

    Ok(MakeRecipeExecution {
        ingredients_to_consume,
        yields_produced,
    })
}
