use super::parse_uuid;
use crate::db::Storage;
use bmp_domain::*;
use rusqlite::{params, Result};
use rust_decimal::Decimal;
use std::collections::HashMap;
use std::str::FromStr;
use uuid::Uuid;

impl Storage {
    // --- RECIPE CRUD ---

    pub fn insert_recipe(&self, recipe: &Recipe) -> Result<()> {
        self.with_transaction(|tx| {
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

            // Delete old yields & edges
            tx.execute("DELETE FROM recipe_yields WHERE recipe_id = ?1", params![recipe.id.0.to_string()])?;
            tx.execute("DELETE FROM ingredient_edges WHERE recipe_id = ?1", params![recipe.id.0.to_string()])?;

            // Insert yields
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

            // Insert edges
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
            Ok(())
        })
    }

    pub fn get_recipe(&self, id: RecipeId) -> Result<Option<Recipe>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT id, name, instructions, servings, meal_type FROM recipes WHERE id = ?1")?;
        let mut rows = stmt.query(params![id.0.to_string()])?;

        if let Some(row) = rows.next()? {
            let id_str: String = row.get(0)?;
            let name: String = row.get(1)?;
            let instructions: String = row.get(2)?;
            let servings_str: String = row.get(3)?;
            let meal_type: Option<String> = row.get(4)?;

            let servings = Decimal::from_str(&servings_str).unwrap_or(Decimal::ONE);
            let mut recipe = Recipe::new(name, servings);
            recipe.id = RecipeId(parse_uuid(&id_str)?);
            recipe.instructions = instructions;
            recipe.meal_type = meal_type;

            // Load yields
            let mut yield_stmt = conn.prepare("SELECT item_id, quantity_amount, quantity_unit FROM recipe_yields WHERE recipe_id = ?1")?;
            let yield_rows = yield_stmt.query_map(params![id_str], |yrow| {
                let item_id_str: String = yrow.get(0)?;
                let amount_str: String = yrow.get(1)?;
                let unit_str: String = yrow.get(2)?;

                let item_id = ItemId(parse_uuid(&item_id_str)?);
                let amount = Decimal::from_str(&amount_str).unwrap_or(Decimal::ONE);
                let unit: Unit = serde_json::from_str(&unit_str).unwrap_or(Unit::Each);

                Ok((item_id, Quantity { amount, unit }))
            })?;

            for y in yield_rows {
                recipe.yields.push(y?);
            }

            // Load ingredient edges
            let mut edge_stmt = conn.prepare(
                "SELECT target_type, target_id, quantity_amount, quantity_unit, required, cycle_flag, per_recipe_substitute
                 FROM ingredient_edges WHERE recipe_id = ?1",
            )?;
            let edge_rows = edge_stmt.query_map(params![id_str], |erow| {
                let target_type: String = erow.get(0)?;
                let target_id_str: String = erow.get(1)?;
                let amount_str: String = erow.get(2)?;
                let unit_str: String = erow.get(3)?;
                let req_int: i32 = erow.get(4)?;
                let cycle_int: i32 = erow.get(5)?;
                let sub_str: Option<String> = erow.get(6)?;

                let target = if target_type == "recipe" {
                    ItemOrRecipeId::Recipe(RecipeId(parse_uuid(&target_id_str)?))
                } else {
                    ItemOrRecipeId::Item(ItemId(parse_uuid(&target_id_str)?))
                };
                let amount = Decimal::from_str(&amount_str).unwrap_or(Decimal::ONE);
                let unit: Unit = serde_json::from_str(&unit_str).unwrap_or(Unit::Gram);
                let required = req_int != 0;
                let cycle_flag = cycle_int != 0;
                let per_recipe_substitute = sub_str.and_then(|s| Uuid::from_str(&s).ok()).map(ItemId);

                Ok(IngredientEdge {
                    target,
                    quantity: Quantity { amount, unit },
                    required,
                    cycle_flag,
                    per_recipe_substitute,
                })
            })?;

            for e in edge_rows {
                recipe.ingredients.push(e?);
            }

            Ok(Some(recipe))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_recipes(&self) -> Result<Vec<Recipe>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT id, name, instructions, servings, meal_type FROM recipes")?;
        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let name: String = row.get(1)?;
            let instructions: String = row.get(2)?;
            let servings_str: String = row.get(3)?;
            let meal_type: Option<String> = row.get(4)?;

            let servings = Decimal::from_str(&servings_str).unwrap_or(Decimal::ONE);
            let mut recipe = Recipe::new(name, servings);
            recipe.id = RecipeId(parse_uuid(&id_str)?);
            recipe.instructions = instructions;
            recipe.meal_type = meal_type;

            Ok(recipe)
        })?;

        let mut recipes = Vec::new();
        for r in rows {
            recipes.push(r?);
        }
        drop(stmt);

        // Load all yields in one query
        let mut yield_stmt = conn.prepare("SELECT recipe_id, item_id, quantity_amount, quantity_unit FROM recipe_yields")?;
        let yield_rows = yield_stmt.query_map([], |yrow| {
            let recipe_id_str: String = yrow.get(0)?;
            let item_id_str: String = yrow.get(1)?;
            let amount_str: String = yrow.get(2)?;
            let unit_str: String = yrow.get(3)?;

            let recipe_id = RecipeId(parse_uuid(&recipe_id_str)?);
            let item_id = ItemId(parse_uuid(&item_id_str)?);
            let amount = Decimal::from_str(&amount_str).unwrap_or(Decimal::ONE);
            let unit: Unit = serde_json::from_str(&unit_str).unwrap_or(Unit::Each);

            Ok((recipe_id, item_id, Quantity { amount, unit }))
        })?;

        let mut yields_map: HashMap<RecipeId, Vec<(ItemId, Quantity)>> = HashMap::new();
        for y in yield_rows {
            let (r_id, i_id, qty) = y?;
            yields_map.entry(r_id).or_default().push((i_id, qty));
        }
        drop(yield_stmt);

        // Load all ingredient edges in one query
        let mut edge_stmt = conn.prepare(
            "SELECT recipe_id, target_type, target_id, quantity_amount, quantity_unit, required, cycle_flag, per_recipe_substitute
             FROM ingredient_edges",
        )?;
        let edge_rows = edge_stmt.query_map([], |erow| {
            let recipe_id_str: String = erow.get(0)?;
            let target_type: String = erow.get(1)?;
            let target_id_str: String = erow.get(2)?;
            let amount_str: String = erow.get(3)?;
            let unit_str: String = erow.get(4)?;
            let req_int: i32 = erow.get(5)?;
            let cycle_int: i32 = erow.get(6)?;
            let sub_str: Option<String> = erow.get(7)?;

            let recipe_id = RecipeId(parse_uuid(&recipe_id_str)?);
            let target = if target_type == "recipe" {
                ItemOrRecipeId::Recipe(RecipeId(parse_uuid(&target_id_str)?))
            } else {
                ItemOrRecipeId::Item(ItemId(parse_uuid(&target_id_str)?))
            };
            let amount = Decimal::from_str(&amount_str).unwrap_or(Decimal::ONE);
            let unit: Unit = serde_json::from_str(&unit_str).unwrap_or(Unit::Gram);
            let required = req_int != 0;
            let cycle_flag = cycle_int != 0;
            let per_sub = sub_str.and_then(|s| Uuid::from_str(&s).ok()).map(ItemId);

            Ok((recipe_id, IngredientEdge {
                target,
                quantity: Quantity { amount, unit },
                required,
                cycle_flag,
                per_recipe_substitute: per_sub,
            }))
        })?;

        let mut edges_map: HashMap<RecipeId, Vec<IngredientEdge>> = HashMap::new();
        for e in edge_rows {
            let (r_id, edge) = e?;
            edges_map.entry(r_id).or_default().push(edge);
        }
        drop(edge_stmt);

        for recipe in &mut recipes {
            if let Some(ys) = yields_map.remove(&recipe.id) {
                recipe.yields = ys;
            }
            if let Some(edges) = edges_map.remove(&recipe.id) {
                recipe.ingredients = edges;
            }
        }

        Ok(recipes)
    }

    pub fn delete_recipe(&self, recipe_id: RecipeId) -> Result<()> {
        let conn = self.conn();
        conn.execute("DELETE FROM recipes WHERE id = ?1", params![recipe_id.0.to_string()])?;
        Ok(())
    }
}
