use bmp_domain::ScheduledMeal;
use bmp_storage::Storage;
use chrono::{DateTime, Duration, Utc};

pub struct NotificationService {
    storage: Storage,
}

impl NotificationService {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    /// Checks scheduled meals and returns any that are due for consumption confirmation (e.g. 30 min after meal time).
    pub fn check_pending_notifications(&self, current_time: DateTime<Utc>) -> Result<Vec<ScheduledMeal>, String> {
        let meals = self.storage.get_all_scheduled_meals().map_err(|e| e.to_string())?;
        let due_meals = meals
            .into_iter()
            .filter(|m| {
                m.consumed.is_none() && (current_time >= m.datetime + Duration::minutes(30))
            })
            .collect();
        Ok(due_meals)
    }
}
