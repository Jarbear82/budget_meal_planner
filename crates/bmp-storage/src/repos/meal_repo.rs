use super::parse_uuid;
use crate::db::Storage;
use bmp_domain::*;
use chrono::{DateTime, Utc};
use rusqlite::{params, Result};
use rust_decimal::Decimal;
use std::str::FromStr;

impl Storage {
    // --- MEAL & SCHEDULE CRUD ---

    pub fn insert_pre_planned_meal(&self, meal: &PrePlannedMeal) -> Result<()> {
        self.with_transaction(|tx| {
            tx.execute(
                "INSERT OR REPLACE INTO pre_planned_meals (id, name) VALUES (?1, ?2)",
                params![meal.id.0.to_string(), meal.name],
            )?;

            tx.execute("DELETE FROM meal_components WHERE meal_id = ?1", params![meal.id.0.to_string()])?;

            for comp in &meal.components {
                match comp {
                    MealComponent::Recipe { recipe_id, servings } => {
                        tx.execute(
                            "INSERT INTO meal_components (meal_id, component_type, target_id_or_name, quantity_or_servings)
                             VALUES (?1, 'recipe', ?2, ?3)",
                            params![meal.id.0.to_string(), recipe_id.0.to_string(), servings.to_string()],
                        )?;
                    }
                    MealComponent::Item { item_id, quantity } => {
                        let unit_str = serde_json::to_string(&quantity.unit).unwrap_or_default();
                        tx.execute(
                            "INSERT INTO meal_components (meal_id, component_type, target_id_or_name, quantity_or_servings, unit_or_cost)
                             VALUES (?1, 'item', ?2, ?3, ?4)",
                            params![meal.id.0.to_string(), item_id.0.to_string(), quantity.amount.to_string(), unit_str],
                        )?;
                    }
                    MealComponent::Restaurant { name, cost, leftover_yield } => {
                        let (left_item, left_amt, left_unit) = match leftover_yield {
                            Some((item_id, qty)) => (
                                Some(item_id.0.to_string()),
                                Some(qty.amount.to_string()),
                                Some(serde_json::to_string(&qty.unit).unwrap_or_default()),
                            ),
                            None => (None, None, None),
                        };
                        tx.execute(
                            "INSERT INTO meal_components (meal_id, component_type, target_id_or_name, quantity_or_servings, unit_or_cost, leftover_item_id, leftover_qty_amount, leftover_qty_unit)
                             VALUES (?1, 'restaurant', ?2, '1', ?3, ?4, ?5, ?6)",
                            params![meal.id.0.to_string(), name, cost.to_string(), left_item, left_amt, left_unit],
                        )?;
                    }
                }
            }
            Ok(())
        })
    }

    pub fn get_all_pre_planned_meals(&self) -> Result<Vec<PrePlannedMeal>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT id, name FROM pre_planned_meals")?;
        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let name: String = row.get(1)?;
            let id = PrePlannedMealId(parse_uuid(&id_str)?);
            Ok(PrePlannedMeal { id, name, components: Vec::new() })
        })?;

        let mut meals = Vec::new();
        for r in rows {
            meals.push(r?);
        }
        drop(stmt);

        let mut comp_stmt = conn.prepare(
            "SELECT meal_id, component_type, target_id_or_name, quantity_or_servings, unit_or_cost, leftover_item_id, leftover_qty_amount, leftover_qty_unit
             FROM meal_components",
        )?;
        let comp_rows = comp_stmt.query_map([], |crow| {
            let meal_id_str: String = crow.get(0)?;
            let comp_type: String = crow.get(1)?;
            let target_str: String = crow.get(2)?;
            let qty_str: String = crow.get(3)?;
            let unit_cost_str: Option<String> = crow.get(4)?;
            let left_item_str: Option<String> = crow.get(5)?;
            let left_amt_str: Option<String> = crow.get(6)?;
            let left_unit_str: Option<String> = crow.get(7)?;

            let meal_id = PrePlannedMealId(parse_uuid(&meal_id_str)?);
            let comp = match comp_type.as_str() {
                "recipe" => {
                    let recipe_id = RecipeId(parse_uuid(&target_str)?);
                    let servings = Decimal::from_str(&qty_str).unwrap_or(Decimal::ONE);
                    MealComponent::Recipe { recipe_id, servings }
                }
                "item" => {
                    let item_id = ItemId(parse_uuid(&target_str)?);
                    let amount = Decimal::from_str(&qty_str).unwrap_or(Decimal::ONE);
                    let unit: Unit = unit_cost_str
                        .and_then(|u| serde_json::from_str(&u).ok())
                        .unwrap_or(Unit::Gram);
                    MealComponent::Item { item_id, quantity: Quantity { amount, unit } }
                }
                "restaurant" => {
                    let name = target_str;
                    let cost = unit_cost_str.and_then(|c| Decimal::from_str(&c).ok()).unwrap_or(Decimal::ZERO);
                    let leftover_yield = if let (Some(l_item), Some(l_amt)) = (left_item_str, left_amt_str) {
                        let item_id = ItemId(parse_uuid(&l_item)?);
                        let amount = Decimal::from_str(&l_amt).unwrap_or(Decimal::ONE);
                        let unit: Unit = left_unit_str
                            .and_then(|u| serde_json::from_str(&u).ok())
                            .unwrap_or(Unit::Each);
                        Some((item_id, Quantity { amount, unit }))
                    } else {
                        None
                    };
                    MealComponent::Restaurant { name, cost, leftover_yield }
                }
                _ => return Err(rusqlite::Error::FromSqlConversionFailure(1, rusqlite::types::Type::Text, "Unknown component type".into())),
            };
            Ok((meal_id, comp))
        })?;

        let mut comp_map: std::collections::HashMap<PrePlannedMealId, Vec<MealComponent>> = std::collections::HashMap::new();
        for c in comp_rows {
            let (m_id, comp) = c?;
            comp_map.entry(m_id).or_default().push(comp);
        }
        drop(comp_stmt);

        for meal in &mut meals {
            if let Some(comps) = comp_map.remove(&meal.id) {
                meal.components = comps;
            }
        }
        Ok(meals)
    }

    pub fn insert_scheduled_meal(&self, meal: &ScheduledMeal) -> Result<()> {
        let conn = self.conn();
        let (source_type, payload_str) = match &meal.source {
            ScheduledMealSource::PrePlanned(_) => ("pre_planned", serde_json::to_string(&meal.source).unwrap_or_default()),
            ScheduledMealSource::OneOff(_) => ("one_off", serde_json::to_string(&meal.source).unwrap_or_default()),
            ScheduledMealSource::Restaurant { .. } => ("restaurant", serde_json::to_string(&meal.source).unwrap_or_default()),
        };
        let dt_str = meal.datetime.to_rfc3339();
        let consumed_str = meal.consumed.map(|dt| dt.to_rfc3339());

        conn.execute(
            "INSERT INTO scheduled_meals (id, source_type, source_payload, datetime, people, consumed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                meal.id.0.to_string(),
                source_type,
                payload_str,
                dt_str,
                meal.people,
                consumed_str
            ],
        )?;
        Ok(())
    }

    pub fn mark_scheduled_meal_consumed(&self, id: ScheduledMealId, time: DateTime<Utc>) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "UPDATE scheduled_meals SET consumed_at = ?1 WHERE id = ?2",
            params![time.to_rfc3339(), id.0.to_string()],
        )?;
        Ok(())
    }

    pub fn get_all_scheduled_meals(&self) -> Result<Vec<ScheduledMeal>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT id, source_payload, datetime, people, consumed_at FROM scheduled_meals")?;
        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let payload_str: String = row.get(1)?;
            let dt_str: String = row.get(2)?;
            let people: u32 = row.get(3)?;
            let consumed_str: Option<String> = row.get(4)?;

            let id = ScheduledMealId(parse_uuid(&id_str)?);
            let source: ScheduledMealSource = serde_json::from_str(&payload_str)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(1, rusqlite::types::Type::Text, Box::new(e)))?;
            let datetime = DateTime::parse_from_rfc3339(&dt_str)
                .map_err(|e| rusqlite::Error::FromSqlConversionFailure(2, rusqlite::types::Type::Text, Box::new(e)))?
                .with_timezone(&Utc);
            let consumed = consumed_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc)));

            Ok(ScheduledMeal {
                id,
                source,
                datetime,
                people,
                consumed,
            })
        })?;

        let mut meals = Vec::new();
        for r in rows {
            meals.push(r?);
        }
        Ok(meals)
    }
}
