use crate::id::{ItemId, ItemOrRecipeId, RecipeId};
use crate::units::Quantity;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IngredientEdge {
    pub target: ItemOrRecipeId,
    pub quantity: Quantity,
    pub required: bool,
    pub cycle_flag: bool,
    pub per_recipe_substitute: Option<ItemId>,
}

impl IngredientEdge {
    pub fn item(item_id: ItemId, quantity: Quantity) -> Self {
        Self {
            target: ItemOrRecipeId::Item(item_id),
            quantity,
            required: true,
            cycle_flag: false,
            per_recipe_substitute: None,
        }
    }

    pub fn recipe(recipe_id: RecipeId, quantity: Quantity) -> Self {
        Self {
            target: ItemOrRecipeId::Recipe(recipe_id),
            quantity,
            required: true,
            cycle_flag: false,
            per_recipe_substitute: None,
        }
    }

    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn with_cycle_flag(mut self, cycle_flag: bool) -> Self {
        self.cycle_flag = cycle_flag;
        self
    }

    pub fn with_substitute(mut self, substitute: ItemId) -> Self {
        self.per_recipe_substitute = Some(substitute);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recipe {
    pub id: RecipeId,
    pub name: String,
    pub yields: Vec<(ItemId, Quantity)>,
    pub ingredients: Vec<IngredientEdge>,
    pub instructions: String,
    pub servings: Decimal,
    pub meal_type: Option<String>,
}

impl Recipe {
    pub fn new(name: impl Into<String>, servings: Decimal) -> Self {
        Self {
            id: RecipeId::new(),
            name: name.into(),
            yields: Vec::new(),
            ingredients: Vec::new(),
            instructions: String::new(),
            servings,
            meal_type: None,
        }
    }

    pub fn add_yield(mut self, item_id: ItemId, quantity: Quantity) -> Self {
        self.yields.push((item_id, quantity));
        self
    }

    pub fn add_ingredient(mut self, edge: IngredientEdge) -> Self {
        self.ingredients.push(edge);
        self
    }

    pub fn with_instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = instructions.into();
        self
    }

    pub fn with_meal_type(mut self, meal_type: impl Into<String>) -> Self {
        self.meal_type = Some(meal_type.into());
        self
    }

    /// Calculates the batch multiplier needed to satisfy a target quantity.
    pub fn scale_multiplier(&self, target_qty: &Quantity) -> Decimal {
        if let Some((_, yield_qty)) = self.yields.first() {
            if yield_qty.amount > Decimal::ZERO {
                let converted_target = target_qty
                    .convert_direct(&yield_qty.unit)
                    .map(|q| q.amount)
                    .unwrap_or(target_qty.amount);
                return converted_target / yield_qty.amount;
            }
        }

        if self.servings > Decimal::ZERO {
            return target_qty.amount / self.servings;
        }

        target_qty.amount
    }
}
