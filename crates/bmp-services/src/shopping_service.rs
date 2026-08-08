use bmp_domain::*;
use bmp_storage::Storage;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::collections::{HashMap, HashSet};

pub struct ShoppingService {
    storage: Storage,
}

impl ShoppingService {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub fn collect_scheduled_meal_requirements(
        &self,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<Vec<(ItemId, Quantity)>, String> {
        let scheduled_meals = self.storage.get_all_scheduled_meals().map_err(|e| e.to_string())?;
        let pre_planned_meals = self.storage.get_all_pre_planned_meals().map_err(|e| e.to_string())?;
        let pre_planned_map: HashMap<PrePlannedMealId, PrePlannedMeal> =
            pre_planned_meals.into_iter().map(|m| (m.id, m)).collect();

        let recipes = self.storage.get_all_recipes().map_err(|e| e.to_string())?;
        let recipes_map: HashMap<RecipeId, Recipe> = recipes.into_iter().map(|r| (r.id, r)).collect();

        let items = self.storage.get_all_items().map_err(|e| e.to_string())?;
        let items_map: HashMap<ItemId, Item> = items.into_iter().map(|i| (i.id, i)).collect();

        let mut total_requirements = Vec::new();

        for meal in scheduled_meals {
            if meal.consumed.is_some() {
                continue;
            }
            if let Some(start) = start_date {
                if meal.datetime < start {
                    continue;
                }
            }
            if let Some(end) = end_date {
                if meal.datetime > end {
                    continue;
                }
            }

            let components = match &meal.source {
                ScheduledMealSource::PrePlanned(meal_id) => {
                    if let Some(ppm) = pre_planned_map.get(meal_id) {
                        ppm.components.clone()
                    } else {
                        Vec::new()
                    }
                }
                ScheduledMealSource::OneOff(comp) => vec![comp.clone()],
                ScheduledMealSource::Restaurant { .. } => Vec::new(),
            };

            let people_factor = Decimal::from(meal.people.max(1));

            for comp in components {
                match comp {
                    MealComponent::Recipe { recipe_id, servings } => {
                        if let Some(recipe) = recipes_map.get(&recipe_id) {
                            let base_servings = if recipe.servings > Decimal::ZERO {
                                recipe.servings
                            } else {
                                Decimal::ONE
                            };
                            let multiplier = (servings / base_servings) * people_factor;
                            let mut visited = HashSet::new();
                            if let Ok(reqs) = expand_recipe(
                                recipe_id,
                                multiplier,
                                &recipes_map,
                                &items_map,
                                &mut visited,
                            ) {
                                total_requirements.extend(reqs);
                            }
                        }
                    }
                    MealComponent::Item { item_id, quantity } => {
                        let scaled_qty = Quantity {
                            amount: quantity.amount * people_factor,
                            unit: quantity.unit,
                        };
                        total_requirements.push((item_id, scaled_qty));
                    }
                    MealComponent::Restaurant { .. } => {}
                }
            }
        }

        Ok(total_requirements)
    }

    pub fn generate_shopping_list(
        &self,
        mut scheduled_meal_requirements: Vec<(ItemId, Quantity)>,
        selected_store_id: Option<StoreId>,
        tax_rate: Option<Decimal>,
    ) -> Result<ShoppingList, String> {
        if scheduled_meal_requirements.is_empty() {
            scheduled_meal_requirements = self.collect_scheduled_meal_requirements(None, None)?;
        }

        let items_list = self.storage.get_all_items().map_err(|e| e.to_string())?;
        let items_map: HashMap<ItemId, Item> = items_list.into_iter().map(|i| (i.id, i)).collect();

        let mut packages_map = HashMap::new();
        for item_id in items_map.keys() {
            let pkgs = self.storage.get_packages_for_item(*item_id).map_err(|e| e.to_string())?;
            packages_map.insert(*item_id, pkgs);
        }

        let pantry_entries = self.storage.get_all_pantry_entries().map_err(|e| e.to_string())?;

        bmp_domain::shopping::generate_shopping_list(
            scheduled_meal_requirements,
            &items_map,
            &packages_map,
            &pantry_entries,
            selected_store_id,
            tax_rate,
        )
        .map_err(|e| e.to_string())
    }
}
