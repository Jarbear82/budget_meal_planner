use crate::db::Storage;
use bmp_domain::*;
use chrono::{DateTime, NaiveDate, Utc};
use rusqlite::{params, Result};
use rust_decimal::Decimal;
use std::str::FromStr;
use uuid::Uuid;

fn parse_uuid(s: &str) -> Result<Uuid> {
    Uuid::from_str(s).map_err(|e| rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e)))
}

impl Storage {
    // --- ITEM CRUD ---

    pub fn insert_item(&self, item: &Item) -> Result<()> {
        let conn = self.conn();
        let density_str = item.density.map(|d| d.g_per_ml.to_string());
        let mode_str = serde_json::to_string(&item.preferred_purchase_mode).unwrap_or_default();

        conn.execute(
            "INSERT OR REPLACE INTO items (id, name, density, preferred_purchase_mode, category)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                item.id.0.to_string(),
                item.name,
                density_str,
                mode_str,
                item.category
            ],
        )?;
        Ok(())
    }

    pub fn get_item(&self, id: ItemId) -> Result<Option<Item>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, name, density, preferred_purchase_mode, category FROM items WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id.0.to_string()])?;

        if let Some(row) = rows.next()? {
            let id_str: String = row.get(0)?;
            let name: String = row.get(1)?;
            let density_str: Option<String> = row.get(2)?;
            let mode_str: String = row.get(3)?;
            let category: Option<String> = row.get(4)?;

            let density = density_str
                .and_then(|s| Decimal::from_str(&s).ok())
                .and_then(|d| Density::new(d).ok());

            let mode: PurchaseMode =
                serde_json::from_str(&mode_str).unwrap_or(PurchaseMode::BuyFinished);

            let mut item = Item::new(name).with_purchase_mode(mode);
            item.id = ItemId(parse_uuid(&id_str)?);
            item.density = density;
            item.category = category;

            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_items(&self) -> Result<Vec<Item>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, name, density, preferred_purchase_mode, category FROM items ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let name: String = row.get(1)?;
            let density_str: Option<String> = row.get(2)?;
            let mode_str: String = row.get(3)?;
            let category: Option<String> = row.get(4)?;

            let density = density_str
                .and_then(|s| Decimal::from_str(&s).ok())
                .and_then(|d| Density::new(d).ok());

            let mode: PurchaseMode =
                serde_json::from_str(&mode_str).unwrap_or(PurchaseMode::BuyFinished);

            let mut item = Item::new(name).with_purchase_mode(mode);
            item.id = ItemId(parse_uuid(&id_str)?);
            item.density = density;
            item.category = category;

            Ok(item)
        })?;

        let mut items = Vec::new();
        for r in rows {
            items.push(r?);
        }
        Ok(items)
    }

    pub fn delete_item(&self, id: ItemId) -> Result<()> {
        let mut conn = self.conn();
        let tx = conn.transaction()?;

        let item_name: Option<String> = tx
            .query_row(
                "SELECT name FROM items WHERE id = ?1",
                params![id.0.to_string()],
                |row| row.get(0),
            )
            .ok();

        if let Some(name) = item_name {
            let ref_count: i64 = tx
                .query_row(
                    "SELECT COUNT(*) FROM ingredient_edges WHERE target_type = 'item' AND target_id = ?1",
                    params![id.0.to_string()],
                    |row| row.get(0),
                )
                .unwrap_or(0);

            if ref_count > 0 {
                // SRS §5.1: Flag/convert referenced items into a placeholder rather than orphan references
                let placeholder_name = format!("[Deleted Item: {}]", name);
                let mode_str = serde_json::to_string(&PurchaseMode::BuyFinished).unwrap_or_default();
                tx.execute(
                    "UPDATE items SET name = ?1, density = NULL, preferred_purchase_mode = ?2, category = 'Placeholder' WHERE id = ?3",
                    params![placeholder_name, mode_str, id.0.to_string()],
                )?;
            } else {
                tx.execute("DELETE FROM items WHERE id = ?1", params![id.0.to_string()])?;
                tx.execute("DELETE FROM packages WHERE item_id = ?1", params![id.0.to_string()])?;
                tx.execute("DELETE FROM pantry_entries WHERE item_id = ?1", params![id.0.to_string()])?;
            }
        }

        tx.commit()?;
        Ok(())
    }

    // --- STORE CRUD ---

    pub fn insert_store(&self, store: &Store) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO stores (id, name) VALUES (?1, ?2)",
            params![store.id.0.to_string(), store.name],
        )?;
        Ok(())
    }

    pub fn get_all_stores(&self) -> Result<Vec<Store>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT id, name FROM stores ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let name: String = row.get(1)?;
            let store = Store {
                id: StoreId(parse_uuid(&id_str)?),
                name,
            };
            Ok(store)
        })?;

        let mut stores = Vec::new();
        for r in rows {
            stores.push(r?);
        }
        Ok(stores)
    }

    pub fn delete_store(&self, id: StoreId) -> Result<()> {
        let conn = self.conn();
        conn.execute("DELETE FROM stores WHERE id = ?1", params![id.0.to_string()])?;
        Ok(())
    }

    // --- PACKAGE CRUD ---

    pub fn insert_package(&self, pkg: &Package) -> Result<()> {
        let conn = self.conn();
        let unit_str = serde_json::to_string(&pkg.quantity.unit).unwrap_or_default();
        let last_seen_str = pkg.last_seen.map(|dt| dt.to_rfc3339());

        conn.execute(
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
                if pkg.is_preferred { 1 } else { 0 }
            ],
        )?;
        Ok(())
    }

    pub fn get_packages_for_item(&self, item_id: ItemId) -> Result<Vec<Package>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, item_id, store_id, quantity_amount, quantity_unit, price, last_seen, is_preferred
             FROM packages WHERE item_id = ?1",
        )?;

        let rows = stmt.query_map(params![item_id.0.to_string()], |row| {
            let id_str: String = row.get(0)?;
            let item_id_str: String = row.get(1)?;
            let store_id_str: String = row.get(2)?;
            let amount_str: String = row.get(3)?;
            let unit_str: String = row.get(4)?;
            let price_str: String = row.get(5)?;
            let last_seen_str: Option<String> = row.get(6)?;
            let is_pref: i32 = row.get(7)?;

            let amount = Decimal::from_str(&amount_str).unwrap_or(Decimal::ZERO);
            let unit: Unit = serde_json::from_str(&unit_str).unwrap_or(Unit::Each);
            let price = Decimal::from_str(&price_str).unwrap_or(Decimal::ZERO);
            let last_seen = last_seen_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc)));

            let pkg = Package {
                id: PackageId(parse_uuid(&id_str)?),
                item_id: ItemId(parse_uuid(&item_id_str)?),
                store_id: StoreId(parse_uuid(&store_id_str)?),
                quantity: Quantity { amount, unit },
                price,
                last_seen,
                is_preferred: is_pref != 0,
            };
            Ok(pkg)
        })?;

        let mut packages = Vec::new();
        for r in rows {
            packages.push(r?);
        }
        Ok(packages)
    }

    pub fn get_all_packages(&self) -> Result<Vec<Package>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, item_id, store_id, quantity_amount, quantity_unit, price, last_seen, is_preferred FROM packages",
        )?;

        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let item_id_str: String = row.get(1)?;
            let store_id_str: String = row.get(2)?;
            let amount_str: String = row.get(3)?;
            let unit_str: String = row.get(4)?;
            let price_str: String = row.get(5)?;
            let last_seen_str: Option<String> = row.get(6)?;
            let is_pref: i32 = row.get(7)?;

            let amount = Decimal::from_str(&amount_str).unwrap_or(Decimal::ZERO);
            let unit: Unit = serde_json::from_str(&unit_str).unwrap_or(Unit::Each);
            let price = Decimal::from_str(&price_str).unwrap_or(Decimal::ZERO);
            let last_seen = last_seen_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc)));

            let pkg = Package {
                id: PackageId(parse_uuid(&id_str)?),
                item_id: ItemId(parse_uuid(&item_id_str)?),
                store_id: StoreId(parse_uuid(&store_id_str)?),
                quantity: Quantity { amount, unit },
                price,
                last_seen,
                is_preferred: is_pref != 0,
            };
            Ok(pkg)
        })?;

        let mut packages = Vec::new();
        for r in rows {
            packages.push(r?);
        }
        Ok(packages)
    }

    // --- PANTRY CRUD ---

    pub fn insert_pantry_entry(&self, entry: &PantryEntry) -> Result<()> {
        let conn = self.conn();
        let unit_str = serde_json::to_string(&entry.quantity.unit).unwrap_or_default();
        let exp_str = entry.expiration.map(|e| e.to_string());

        conn.execute(
            "INSERT INTO pantry_entries (id, item_id, quantity_amount, quantity_unit, expiration)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                entry.id.0.to_string(),
                entry.item_id.0.to_string(),
                entry.quantity.amount.to_string(),
                unit_str,
                exp_str
            ],
        )?;
        Ok(())
    }

    pub fn get_all_pantry_entries(&self) -> Result<Vec<PantryEntry>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, item_id, quantity_amount, quantity_unit, expiration FROM pantry_entries",
        )?;
        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let item_id_str: String = row.get(1)?;
            let amount_str: String = row.get(2)?;
            let unit_str: String = row.get(3)?;
            let exp_str: Option<String> = row.get(4)?;

            let amount = Decimal::from_str(&amount_str).unwrap_or(Decimal::ZERO);
            let unit: Unit = serde_json::from_str(&unit_str).unwrap_or(Unit::Each);
            let expiration = exp_str.and_then(|s| NaiveDate::from_str(&s).ok());

            let entry = PantryEntry {
                id: PantryEntryId(parse_uuid(&id_str)?),
                item_id: ItemId(parse_uuid(&item_id_str)?),
                quantity: Quantity { amount, unit },
                expiration,
            };
            Ok(entry)
        })?;

        let mut entries = Vec::new();
        for r in rows {
            entries.push(r?);
        }
        Ok(entries)
    }

    pub fn update_pantry_quantity(&self, entry_id: PantryEntryId, new_amount: Decimal) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "UPDATE pantry_entries SET quantity_amount = ?1 WHERE id = ?2",
            params![new_amount.to_string(), entry_id.0.to_string()],
        )?;
        Ok(())
    }

    pub fn delete_pantry_entry(&self, entry_id: PantryEntryId) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "DELETE FROM pantry_entries WHERE id = ?1",
            params![entry_id.0.to_string()],
        )?;
        Ok(())
    }

    // --- RECIPE CRUD ---

    pub fn insert_recipe(&self, recipe: &Recipe) -> Result<()> {
        let mut conn = self.conn();
        let tx = conn.transaction()?;

        tx.execute(
            "INSERT OR REPLACE INTO recipes (id, name, instructions, servings) VALUES (?1, ?2, ?3, ?4)",
            params![
                recipe.id.0.to_string(),
                recipe.name,
                recipe.instructions,
                recipe.servings.to_string()
            ],
        )?;

        tx.execute(
            "DELETE FROM recipe_yields WHERE recipe_id = ?1",
            params![recipe.id.0.to_string()],
        )?;

        tx.execute(
            "DELETE FROM ingredient_edges WHERE recipe_id = ?1",
            params![recipe.id.0.to_string()],
        )?;

        for (item_id, qty) in &recipe.yields {
            let unit_str = serde_json::to_string(&qty.unit).unwrap_or_default();
            tx.execute(
                "INSERT INTO recipe_yields (recipe_id, item_id, quantity_amount, quantity_unit)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    recipe.id.0.to_string(),
                    item_id.0.to_string(),
                    qty.amount.to_string(),
                    unit_str
                ],
            )?;
        }

        for edge in &recipe.ingredients {
            let (target_type, target_id) = match edge.target {
                ItemOrRecipeId::Item(id) => ("item", id.0.to_string()),
                ItemOrRecipeId::Recipe(id) => ("recipe", id.0.to_string()),
            };
            let unit_str = serde_json::to_string(&edge.quantity.unit).unwrap_or_default();
            let sub_str = edge.per_recipe_substitute.map(|s| s.0.to_string());

            tx.execute(
                "INSERT INTO ingredient_edges
                 (recipe_id, target_type, target_id, quantity_amount, quantity_unit, required, cycle_flag, per_recipe_substitute)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    recipe.id.0.to_string(),
                    target_type,
                    target_id,
                    edge.quantity.amount.to_string(),
                    unit_str,
                    if edge.required { 1 } else { 0 },
                    if edge.cycle_flag { 1 } else { 0 },
                    sub_str
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn get_all_recipes(&self) -> Result<Vec<Recipe>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT id, name, instructions, servings FROM recipes ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let name: String = row.get(1)?;
            let instructions: String = row.get(2)?;
            let servings_str: String = row.get(3)?;

            let recipe_id = RecipeId(parse_uuid(&id_str)?);
            let servings = Decimal::from_str(&servings_str).unwrap_or(Decimal::ONE);

            let mut recipe = Recipe::new(name, servings).with_instructions(instructions);
            recipe.id = recipe_id;
            Ok(recipe)
        })?;

        let mut recipes = Vec::new();
        for r in rows {
            let mut recipe = r?;
            let mut yield_stmt = conn.prepare("SELECT item_id, quantity_amount, quantity_unit FROM recipe_yields WHERE recipe_id = ?1")?;
            let yield_rows = yield_stmt.query_map(params![recipe.id.0.to_string()], |yrow| {
                let item_id_str: String = yrow.get(0)?;
                let amount_str: String = yrow.get(1)?;
                let unit_str: String = yrow.get(2)?;
                let item_id = ItemId(parse_uuid(&item_id_str)?);
                let amount = Decimal::from_str(&amount_str).unwrap_or(Decimal::ONE);
                let unit: Unit = serde_json::from_str(&unit_str).unwrap_or(Unit::Each);
                Ok((item_id, Quantity { amount, unit }))
            })?;
            for yr in yield_rows {
                recipe.yields.push(yr?);
            }

            let mut edge_stmt = conn.prepare(
                "SELECT target_type, target_id, quantity_amount, quantity_unit, required, cycle_flag, per_recipe_substitute
                 FROM ingredient_edges WHERE recipe_id = ?1",
            )?;
            let edge_rows = edge_stmt.query_map(params![recipe.id.0.to_string()], |erow| {
                let target_type: String = erow.get(0)?;
                let target_id_str: String = erow.get(1)?;
                let amount_str: String = erow.get(2)?;
                let unit_str: String = erow.get(3)?;
                let required: i32 = erow.get(4)?;
                let cycle_flag: i32 = erow.get(5)?;
                let sub_str: Option<String> = erow.get(6)?;

                let target = match target_type.as_str() {
                    "recipe" => ItemOrRecipeId::Recipe(RecipeId(parse_uuid(&target_id_str)?)),
                    _ => ItemOrRecipeId::Item(ItemId(parse_uuid(&target_id_str)?)),
                };
                let amount = Decimal::from_str(&amount_str).unwrap_or(Decimal::ONE);
                let unit: Unit = serde_json::from_str(&unit_str).unwrap_or(Unit::Each);
                let per_sub = sub_str.and_then(|s| Uuid::from_str(&s).ok()).map(ItemId);

                Ok(IngredientEdge {
                    target,
                    quantity: Quantity { amount, unit },
                    required: required != 0,
                    cycle_flag: cycle_flag != 0,
                    per_recipe_substitute: per_sub,
                })
            })?;
            for er in edge_rows {
                recipe.ingredients.push(er?);
            }

            recipes.push(recipe);
        }

        Ok(recipes)
    }

    // --- PRE-PLANNED MEALS & SCHEDULED MEALS ---

    pub fn insert_pre_planned_meal(&self, meal: &PrePlannedMeal) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT INTO pre_planned_meals (id, name) VALUES (?1, ?2)",
            params![meal.id.0.to_string(), meal.name],
        )?;

        for component in &meal.components {
            let (c_type, target, qty, unit, leftover_id, leftover_amt, leftover_unit) = match component {
                MealComponent::Recipe { recipe_id, servings } => (
                    "recipe",
                    recipe_id.0.to_string(),
                    servings.to_string(),
                    None,
                    None,
                    None,
                    None,
                ),
                MealComponent::Item { item_id, quantity } => (
                    "item",
                    item_id.0.to_string(),
                    quantity.amount.to_string(),
                    Some(serde_json::to_string(&quantity.unit).unwrap_or_default()),
                    None,
                    None,
                    None,
                ),
                MealComponent::Restaurant { name, cost, leftover_yield } => {
                    let (lid, lamt, lunit) = if let Some((id, q)) = leftover_yield {
                        (
                            Some(id.0.to_string()),
                            Some(q.amount.to_string()),
                            Some(serde_json::to_string(&q.unit).unwrap_or_default()),
                        )
                    } else {
                        (None, None, None)
                    };
                    ("restaurant", name.clone(), cost.to_string(), None, lid, lamt, lunit)
                }
            };

            conn.execute(
                "INSERT INTO meal_components
                 (meal_id, component_type, target_id_or_name, quantity_or_servings, unit_or_cost, leftover_item_id, leftover_qty_amount, leftover_qty_unit)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![meal.id.0.to_string(), c_type, target, qty, unit, leftover_id, leftover_amt, leftover_unit],
            )?;
        }
        Ok(())
    }

    pub fn get_all_pre_planned_meals(&self) -> Result<Vec<PrePlannedMeal>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT id, name FROM pre_planned_meals")?;
        let meal_rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let name: String = row.get(1)?;
            let id = PrePlannedMealId(parse_uuid(&id_str)?);
            Ok((id, name))
        })?;

        let mut meals = Vec::new();
        for m in meal_rows {
            let (id, name) = m?;
            let mut comp_stmt = conn.prepare(
                "SELECT component_type, target_id_or_name, quantity_or_servings, unit_or_cost, leftover_item_id, leftover_qty_amount, leftover_qty_unit
                 FROM meal_components WHERE meal_id = ?1",
            )?;
            let comp_rows = comp_stmt.query_map(params![id.0.to_string()], |crow| {
                let c_type: String = crow.get(0)?;
                let target: String = crow.get(1)?;
                let qty_serv: String = crow.get(2)?;
                let unit_cost: Option<String> = crow.get(3)?;
                let leftover_id: Option<String> = crow.get(4)?;
                let leftover_amt: Option<String> = crow.get(5)?;
                let leftover_unit: Option<String> = crow.get(6)?;

                let component = match c_type.as_str() {
                    "recipe" => {
                        let recipe_id = RecipeId(parse_uuid(&target)?);
                        let servings = Decimal::from_str(&qty_serv).unwrap_or(Decimal::ONE);
                        MealComponent::Recipe { recipe_id, servings }
                    }
                    "item" => {
                        let item_id = ItemId(parse_uuid(&target)?);
                        let amount = Decimal::from_str(&qty_serv).unwrap_or(Decimal::ONE);
                        let unit: Unit = unit_cost
                            .and_then(|u| serde_json::from_str(&u).ok())
                            .unwrap_or(Unit::Each);
                        MealComponent::Item {
                            item_id,
                            quantity: Quantity { amount, unit },
                        }
                    }
                    "restaurant" => {
                        let cost = Decimal::from_str(&qty_serv).unwrap_or(Decimal::ZERO);
                        let leftover_yield = if let (Some(lid), Some(lamt), Some(lunit)) =
                            (leftover_id, leftover_amt, leftover_unit)
                        {
                            let item_id = ItemId(parse_uuid(&lid)?);
                            let amount = Decimal::from_str(&lamt).unwrap_or(Decimal::ZERO);
                            let unit: Unit = serde_json::from_str(&lunit).unwrap_or(Unit::Each);
                            Some((item_id, Quantity { amount, unit }))
                        } else {
                            None
                        };
                        MealComponent::Restaurant {
                            name: target,
                            cost,
                            leftover_yield,
                        }
                    }
                    _ => MealComponent::Restaurant {
                        name: target,
                        cost: Decimal::ZERO,
                        leftover_yield: None,
                    },
                };
                Ok(component)
            })?;

            let mut components = Vec::new();
            for c in comp_rows {
                components.push(c?);
            }

            meals.push(PrePlannedMeal { id, name, components });
        }
        Ok(meals)
    }

    pub fn insert_scheduled_meal(&self, meal: &ScheduledMeal) -> Result<()> {
        let conn = self.conn();
        let source_type = match &meal.source {
            ScheduledMealSource::PrePlanned(_) => "pre_planned",
            ScheduledMealSource::OneOff(_) => "one_off",
            ScheduledMealSource::Restaurant { .. } => "restaurant",
        };
        let payload_str = serde_json::to_string(&meal.source).unwrap_or_default();
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

    // --- RECEIPTS & ANALYTICS ---

    pub fn insert_receipt(&self, store_id: Option<StoreId>, total: Decimal, datetime: DateTime<Utc>) -> Result<String> {
        let conn = self.conn();
        let id = Uuid::new_v4().to_string();
        let store_str = store_id.map(|s| s.0.to_string());

        conn.execute(
            "INSERT INTO receipts (id, store_id, total, datetime) VALUES (?1, ?2, ?3, ?4)",
            params![id, store_str, total.to_string(), datetime.to_rfc3339()],
        )?;
        Ok(id)
    }

    pub fn get_all_receipts(&self) -> Result<Vec<(String, Option<StoreId>, Decimal, DateTime<Utc>)>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT id, store_id, total, datetime FROM receipts")?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let store_str: Option<String> = row.get(1)?;
            let total_str: String = row.get(2)?;
            let dt_str: String = row.get(3)?;

            let store_id = store_str.and_then(|s| Uuid::from_str(&s).ok()).map(StoreId);
            let total = Decimal::from_str(&total_str).unwrap_or(Decimal::ZERO);
            let dt = DateTime::parse_from_rfc3339(&dt_str).unwrap_or_default().with_timezone(&Utc);

            Ok((id, store_id, total, dt))
        })?;

        let mut receipts = Vec::new();
        for r in rows {
            receipts.push(r?);
        }
        Ok(receipts)
    }
}
