use crate::error::ServiceResult;
use crate::event_bus::EventBus;
use bmp_domain::*;
use bmp_storage::Storage;
use chrono::{DateTime, Utc};

pub struct MealService {
    storage: Storage,
    event_bus: EventBus,
}

impl MealService {
    pub fn new(storage: Storage, event_bus: EventBus) -> Self {
        Self { storage, event_bus }
    }

    pub fn new_with_storage(storage: Storage) -> Self {
        Self::new(storage, EventBus::default())
    }

    pub fn create_pre_planned_meal(&self, name: &str, components: Vec<MealComponent>) -> ServiceResult<PrePlannedMeal> {
        let mut meal = PrePlannedMeal::new(name);
        meal.components = components;
        self.storage.insert_pre_planned_meal(&meal)?;
        self.event_bus.publish(DomainEvent::PrePlannedMealSaved(meal.id));
        Ok(meal)
    }

    pub fn schedule_meal(
        &self,
        source: ScheduledMealSource,
        datetime: DateTime<Utc>,
        people: u32,
    ) -> ServiceResult<ScheduledMeal> {
        let meal = ScheduledMeal::new(source, datetime, people);
        self.storage.insert_scheduled_meal(&meal)?;
        self.event_bus.publish(DomainEvent::MealScheduled(meal.id));
        Ok(meal)
    }

    pub fn list_pre_planned_meals(&self) -> ServiceResult<Vec<PrePlannedMeal>> {
        Ok(self.storage.get_all_pre_planned_meals()?)
    }

    pub fn list_scheduled_meals(&self) -> ServiceResult<Vec<ScheduledMeal>> {
        Ok(self.storage.get_all_scheduled_meals()?)
    }

    pub fn confirm_meal_consumed(&self, meal_id: ScheduledMealId, time: DateTime<Utc>) -> ServiceResult<()> {
        self.storage.mark_scheduled_meal_consumed(meal_id, time)?;
        self.event_bus.publish(DomainEvent::MealConsumed(meal_id));
        Ok(())
    }
}
