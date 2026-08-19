use crate::db::Storage;
use bmp_domain::*;
use rusqlite::{params, Result};

impl Storage {
    // --- FULL DATABASE BACKUP EXPORT & IMPORT ---

    pub fn export_all(&self) -> Result<DatabaseBackup> {
        let items = self.get_all_items()?;
        let stores = self.get_all_stores()?;
        let packages = self.get_all_packages()?;
        let recipes = self.get_all_recipes()?;
        let pre_planned_meals = self.get_all_pre_planned_meals()?;
        let scheduled_meals = self.get_all_scheduled_meals()?;
        let pantry_entries = self.get_all_pantry_entries()?;
        let unit_bridges = self.get_all_unit_bridges()?;
        let global_substitutes = self.get_all_global_substitutes()?;
        let raw_receipts = self.get_all_receipts()?;
        let receipts = raw_receipts
            .into_iter()
            .map(|(id, store_id, total, datetime)| ReceiptRecord {
                id,
                store_id,
                total,
                datetime,
            })
            .collect();

        Ok(DatabaseBackup::new(
            items,
            stores,
            packages,
            recipes,
            pre_planned_meals,
            scheduled_meals,
            pantry_entries,
            unit_bridges,
            global_substitutes,
            receipts,
        ))
    }

    pub fn import_all(&self, backup: &DatabaseBackup, overwrite: bool) -> Result<()> {
        self.with_transaction(|tx| {
            if overwrite {
                // Delete existing in reverse foreign key order
                tx.execute("DELETE FROM receipts", [])?;
                tx.execute("DELETE FROM scheduled_meals", [])?;
                tx.execute("DELETE FROM meal_components", [])?;
                tx.execute("DELETE FROM pre_planned_meals", [])?;
                tx.execute("DELETE FROM ingredient_edges", [])?;
                tx.execute("DELETE FROM recipe_yields", [])?;
                tx.execute("DELETE FROM recipes", [])?;
                tx.execute("DELETE FROM pantry_entries", [])?;
                tx.execute("DELETE FROM packages", [])?;
                tx.execute("DELETE FROM unit_bridges", [])?;
                tx.execute("DELETE FROM global_substitutes", [])?;
                tx.execute("DELETE FROM stores", [])?;
                tx.execute("DELETE FROM items", [])?;
            }

            // Insert items
            for item in &backup.items {
                let density_str = item.density.map(|d| d.g_per_ml.to_string());
                let mode_str = serde_json::to_string(&item.preferred_purchase_mode).unwrap_or_default();
                let nut_str = item.nutrition.as_ref().map(|n| serde_json::to_string(n).unwrap_or_default());
                let flags_str = if item.dietary_flags.is_empty() {
                    None
                } else {
                    Some(serde_json::to_string(&item.dietary_flags).unwrap_or_default())
                };
                tx.execute(
                    "INSERT OR REPLACE INTO items (id, name, density, preferred_purchase_mode, category, nutrition, dietary_flags)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                    params![
                        item.id.0.to_string(),
                        item.name,
                        density_str,
                        mode_str,
                        item.category,
                        nut_str,
                        flags_str
                    ],
                )?;
            }

            // Insert stores
            for store in &backup.stores {
                tx.execute(
                    "INSERT OR REPLACE INTO stores (id, name) VALUES (?1, ?2)",
                    params![store.id.0.to_string(), store.name],
                )?;
            }

            // Insert packages
            for pkg in &backup.packages {
                let unit_str = serde_json::to_string(&pkg.quantity.unit).unwrap_or_default();
                let last_seen_str = pkg.last_seen.map(|dt| dt.to_rfc3339());
                let is_pref = if pkg.is_preferred { 1 } else { 0 };
                tx.execute(
                    "INSERT OR REPLACE INTO packages (id, item_id, store_id, quantity_amount, quantity_unit, price, last_seen, is_preferred)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    params![
                        pkg.id.0.to_string(),
                        pkg.item_id.0.to_string(),
                        pkg.store_id.0.to_string(),
                        pkg.quantity.amount.to_string(),
                        unit_str,
                        pkg.price.to_string(),
                        last_seen_str,
                        is_pref
                    ],
                )?;
            }

            // Insert recipes, yields, edges
            for recipe in &backup.recipes {
                let servings_str = recipe.servings.to_string();
                tx.execute(
                    "INSERT OR REPLACE INTO recipes (id, name, instructions, servings, meal_type)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        recipe.id.0.to_string(),
                        recipe.name,
                        recipe.instructions,
                        servings_str,
                        recipe.meal_type
                    ],
                )?;

                for (y_item_id, y_qty) in &recipe.yields {
                    let unit_str = serde_json::to_string(&y_qty.unit).unwrap_or_default();
                    tx.execute(
                        "INSERT OR REPLACE INTO recipe_yields (recipe_id, item_id, quantity_amount, quantity_unit)
                         VALUES (?1, ?2, ?3, ?4)",
                        params![
                            recipe.id.0.to_string(),
                            y_item_id.0.to_string(),
                            y_qty.amount.to_string(),
                            unit_str
                        ],
                    )?;
                }

                for edge in &recipe.ingredients {
                    let (target_type, target_id_str) = match edge.target {
                        ItemOrRecipeId::Item(id) => ("item", id.0.to_string()),
                        ItemOrRecipeId::Recipe(id) => ("recipe", id.0.to_string()),
                    };
                    let unit_str = serde_json::to_string(&edge.quantity.unit).unwrap_or_default();
                    let req_int = if edge.required { 1 } else { 0 };
                    let cycle_int = if edge.cycle_flag { 1 } else { 0 };
                    let sub_str = edge.per_recipe_substitute.map(|s| s.0.to_string());

                    tx.execute(
                        "INSERT INTO ingredient_edges
                         (recipe_id, target_type, target_id, quantity_amount, quantity_unit, required, cycle_flag, per_recipe_substitute)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                        params![
                            recipe.id.0.to_string(),
                            target_type,
                            target_id_str,
                            edge.quantity.amount.to_string(),
                            unit_str,
                            req_int,
                            cycle_int,
                            sub_str
                        ],
                    )?;
                }
            }

            // Insert pre_planned_meals
            for meal in &backup.pre_planned_meals {
                tx.execute(
                    "INSERT OR REPLACE INTO pre_planned_meals (id, name) VALUES (?1, ?2)",
                    params![meal.id.0.to_string(), meal.name],
                )?;

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
            }

            // Insert scheduled_meals
            for meal in &backup.scheduled_meals {
                let (source_type, payload_str) = match &meal.source {
                    ScheduledMealSource::PrePlanned(_) => ("pre_planned", serde_json::to_string(&meal.source).unwrap_or_default()),
                    ScheduledMealSource::OneOff(_) => ("one_off", serde_json::to_string(&meal.source).unwrap_or_default()),
                    ScheduledMealSource::Restaurant { .. } => ("restaurant", serde_json::to_string(&meal.source).unwrap_or_default()),
                };
                let dt_str = meal.datetime.to_rfc3339();
                let consumed_str = meal.consumed.map(|dt| dt.to_rfc3339());
                tx.execute(
                    "INSERT OR REPLACE INTO scheduled_meals (id, source_type, source_payload, datetime, people, consumed_at)
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
            }

            // Insert pantry_entries
            for entry in &backup.pantry_entries {
                let unit_str = serde_json::to_string(&entry.quantity.unit).unwrap_or_default();
                let exp_str = entry.expiration.map(|d| d.to_string());
                tx.execute(
                    "INSERT OR REPLACE INTO pantry_entries (id, item_id, quantity_amount, quantity_unit, expiration)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        entry.id.0.to_string(),
                        entry.item_id.0.to_string(),
                        entry.quantity.amount.to_string(),
                        unit_str,
                        exp_str
                    ],
                )?;
            }

            // Insert unit_bridges
            for bridge in &backup.unit_bridges {
                let from_unit_str = serde_json::to_string(&bridge.from_qty.unit).unwrap_or_default();
                let to_unit_str = serde_json::to_string(&bridge.to_qty.unit).unwrap_or_default();
                tx.execute(
                    "INSERT OR REPLACE INTO unit_bridges (item_id, from_amount, from_unit, to_amount, to_unit)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        bridge.item_id.0.to_string(),
                        bridge.from_qty.amount.to_string(),
                        from_unit_str,
                        bridge.to_qty.amount.to_string(),
                        to_unit_str
                    ],
                )?;
            }

            // Insert global_substitutes
            for (primary, sub) in &backup.global_substitutes {
                tx.execute(
                    "INSERT OR REPLACE INTO global_substitutes (primary_item_id, substitute_item_id)
                     VALUES (?1, ?2)",
                    params![primary.0.to_string(), sub.0.to_string()],
                )?;
            }

            // Insert receipts
            for r in &backup.receipts {
                let store_str = r.store_id.map(|s| s.0.to_string());
                tx.execute(
                    "INSERT OR REPLACE INTO receipts (id, store_id, total, datetime)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![r.id, store_str, r.total.to_string(), r.datetime.to_rfc3339()],
                )?;
            }

            Ok(())
        })
    }
}
