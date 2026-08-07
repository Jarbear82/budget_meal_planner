use crate::id::{ItemId, PrePlannedMealId, RecipeId, ScheduledMealId};
use crate::units::Quantity;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MealComponent {
    Recipe {
        recipe_id: RecipeId,
        servings: Decimal,
    },
    Item {
        item_id: ItemId,
        quantity: Quantity,
    },
    Restaurant {
        name: String,
        cost: Decimal,
        leftover_yield: Option<(ItemId, Quantity)>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrePlannedMeal {
    pub id: PrePlannedMealId,
    pub name: String,
    pub components: Vec<MealComponent>,
}

impl PrePlannedMeal {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: PrePlannedMealId::new(),
            name: name.into(),
            components: Vec::new(),
        }
    }

    pub fn add_component(mut self, component: MealComponent) -> Self {
        self.components.push(component);
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduledMealSource {
    PrePlanned(PrePlannedMealId),
    OneOff(MealComponent),
    Restaurant {
        name: String,
        cost: Decimal,
        leftover_yield: Option<(ItemId, Quantity)>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduledMeal {
    pub id: ScheduledMealId,
    pub source: ScheduledMealSource,
    pub datetime: DateTime<Utc>,
    pub people: u32,
    pub consumed: Option<DateTime<Utc>>,
}

impl ScheduledMeal {
    pub fn new(source: ScheduledMealSource, datetime: DateTime<Utc>, people: u32) -> Self {
        Self {
            id: ScheduledMealId::new(),
            source,
            datetime,
            people,
            consumed: None,
        }
    }
}
