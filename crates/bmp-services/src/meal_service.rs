use bmp_domain::*;
use bmp_storage::Storage;
use chrono::{DateTime, Utc};

pub struct MealService {
    storage: Storage,
}

impl MealService {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub fn create_pre_planned_meal(&self, name: &str, components: Vec<MealComponent>) -> Result<PrePlannedMeal, String> {
        let mut meal = PrePlannedMeal::new(name);
        meal.components = components;
        self.storage.insert_pre_planned_meal(&meal).map_err(|e| e.to_string())?;
        Ok(meal)
    }

    pub fn schedule_meal(
        &self,
        source: ScheduledMealSource,
        datetime: DateTime<Utc>,
        people: u32,
    ) -> Result<ScheduledMeal, String> {
        let meal = ScheduledMeal::new(source, datetime, people);
        self.storage.insert_scheduled_meal(&meal).map_err(|e| e.to_string())?;
        Ok(meal)
    }

    pub fn list_scheduled_meals(&self) -> Result<Vec<ScheduledMeal>, String> {
        self.storage.get_all_scheduled_meals().map_err(|e| e.to_string())
    }

    pub fn confirm_meal_consumed(&self, meal_id: ScheduledMealId, time: DateTime<Utc>) -> Result<(), String> {
        self.storage.mark_scheduled_meal_consumed(meal_id, time).map_err(|e| e.to_string())?;
        Ok(())
    }
}
