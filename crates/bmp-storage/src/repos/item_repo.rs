use super::parse_uuid;
use crate::db::Storage;
use bmp_domain::*;
use chrono::{DateTime, Utc};
use rusqlite::{params, Result};
use rust_decimal::Decimal;
use std::str::FromStr;

impl Storage {
    // --- ITEM CRUD ---

    pub fn insert_item(&self, item: &Item) -> Result<()> {
        let conn = self.conn();
        let density_str = item.density.map(|d| d.g_per_ml.to_string());
        let mode_str = serde_json::to_string(&item.preferred_purchase_mode).unwrap_or_default();
        let nut_str = item.nutrition.as_ref().map(|n| serde_json::to_string(n).unwrap_or_default());
        let flags_str = if item.dietary_flags.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&item.dietary_flags).unwrap_or_default())
        };

        conn.execute(
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

        if let Some(ref bridge) = item.count_bridge {
            let from_unit_str = serde_json::to_string(&bridge.from_qty.unit).unwrap_or_default();
            let to_unit_str = serde_json::to_string(&bridge.to_qty.unit).unwrap_or_default();
            conn.execute(
                "INSERT OR REPLACE INTO unit_bridges (item_id, from_amount, from_unit, to_amount, to_unit)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    item.id.0.to_string(),
                    bridge.from_qty.amount.to_string(),
                    from_unit_str,
                    bridge.to_qty.amount.to_string(),
                    to_unit_str,
                ],
            )?;
        }
        Ok(())
    }

    pub fn get_item(&self, id: ItemId) -> Result<Option<Item>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, name, density, preferred_purchase_mode, category, nutrition, dietary_flags FROM items WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id.0.to_string()])?;

        if let Some(row) = rows.next()? {
            let id_str: String = row.get(0)?;
            let name: String = row.get(1)?;
            let density_str: Option<String> = row.get(2)?;
            let mode_str: String = row.get(3)?;
            let category: Option<String> = row.get(4)?;
            let nut_str: Option<String> = row.get(5)?;
            let flags_str: Option<String> = row.get(6)?;

            let density = density_str
                .and_then(|s| Decimal::from_str(&s).ok())
                .and_then(|d| Density::new(d).ok());

            let mode: PurchaseMode =
                serde_json::from_str(&mode_str).unwrap_or(PurchaseMode::BuyFinished);
            let nutrition: Option<NutritionalInfo> = nut_str.and_then(|s| serde_json::from_str(&s).ok());
            let dietary_flags: Vec<DietaryFlag> = flags_str.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();

            let mut item = Item::new(name).with_purchase_mode(mode);
            item.id = ItemId(parse_uuid(&id_str)?);
            item.density = density;
            item.category = category;
            item.nutrition = nutrition;
            item.dietary_flags = dietary_flags;

            // Re-hydrate count_bridge if present
            let mut bridge_stmt = conn.prepare(
                "SELECT from_amount, from_unit, to_amount, to_unit FROM unit_bridges WHERE item_id = ?1",
            )?;
            let mut bridge_rows = bridge_stmt.query(params![item.id.0.to_string()])?;
            if let Some(brow) = bridge_rows.next()? {
                let from_amt_str: String = brow.get(0)?;
                let from_unit_str: String = brow.get(1)?;
                let to_amt_str: String = brow.get(2)?;
                let to_unit_str: String = brow.get(3)?;

                let from_amount = Decimal::from_str(&from_amt_str).unwrap_or(Decimal::ONE);
                let from_unit: Unit = serde_json::from_str(&from_unit_str).unwrap_or(Unit::Each);
                let to_amount = Decimal::from_str(&to_amt_str).unwrap_or(Decimal::ONE);
                let to_unit: Unit = serde_json::from_str(&to_unit_str).unwrap_or(Unit::Gram);

                let from_qty = Quantity { amount: from_amount, unit: from_unit };
                let to_qty = Quantity { amount: to_amount, unit: to_unit };

                item.count_bridge = Some(UnitBridge {
                    item_id: item.id,
                    from_qty,
                    to_qty,
                });
            }

            Ok(Some(item))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_items(&self) -> Result<Vec<Item>> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, name, density, preferred_purchase_mode, category, nutrition, dietary_flags FROM items",
        )?;
        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let name: String = row.get(1)?;
            let density_str: Option<String> = row.get(2)?;
            let mode_str: String = row.get(3)?;
            let category: Option<String> = row.get(4)?;
            let nut_str: Option<String> = row.get(5)?;
            let flags_str: Option<String> = row.get(6)?;

            let id = ItemId(parse_uuid(&id_str)?);
            let density = density_str
                .and_then(|s| Decimal::from_str(&s).ok())
                .and_then(|d| Density::new(d).ok());
            let mode: PurchaseMode =
                serde_json::from_str(&mode_str).unwrap_or(PurchaseMode::BuyFinished);
            let nutrition: Option<NutritionalInfo> = nut_str.and_then(|s| serde_json::from_str(&s).ok());
            let dietary_flags: Vec<DietaryFlag> = flags_str.and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default();

            let mut item = Item::new(name).with_purchase_mode(mode);
            item.id = id;
            item.density = density;
            item.category = category;
            item.nutrition = nutrition;
            item.dietary_flags = dietary_flags;

            Ok(item)
        })?;

        let mut items = Vec::new();
        for r in rows {
            items.push(r?);
        }
        drop(stmt);

        // Fetch all unit bridges in a single pass
        let mut bridge_stmt = conn.prepare(
            "SELECT item_id, from_amount, from_unit, to_amount, to_unit FROM unit_bridges",
        )?;
        let bridge_rows = bridge_stmt.query_map([], |brow| {
            let item_id_str: String = brow.get(0)?;
            let from_amt_str: String = brow.get(1)?;
            let from_unit_str: String = brow.get(2)?;
            let to_amt_str: String = brow.get(3)?;
            let to_unit_str: String = brow.get(4)?;

            let item_id = ItemId(parse_uuid(&item_id_str)?);
            let from_amount = Decimal::from_str(&from_amt_str).unwrap_or(Decimal::ONE);
            let from_unit: Unit = serde_json::from_str(&from_unit_str).unwrap_or(Unit::Each);
            let to_amount = Decimal::from_str(&to_amt_str).unwrap_or(Decimal::ONE);
            let to_unit: Unit = serde_json::from_str(&to_unit_str).unwrap_or(Unit::Gram);

            Ok((item_id, UnitBridge {
                item_id,
                from_qty: Quantity { amount: from_amount, unit: from_unit },
                to_qty: Quantity { amount: to_amount, unit: to_unit },
            }))
        })?;

        let mut bridge_map = std::collections::HashMap::new();
        for b in bridge_rows {
            let (id, bridge) = b?;
            bridge_map.insert(id, bridge);
        }

        for item in &mut items {
            if let Some(bridge) = bridge_map.remove(&item.id) {
                item.count_bridge = Some(bridge);
            }
        }

        Ok(items)
    }

    pub fn delete_item(&self, item_id: ItemId) -> Result<()> {
        // SRS 5.1 Placeholder Rule: Deleting an Item replaces references in Recipes with a placeholder
        let current_item = self.get_item(item_id)?;
        let item_name = current_item.map(|i| i.name).unwrap_or_else(|| "Unknown".to_string());
        let placeholder_name = format!("[Deleted Item: {}]", item_name);

        let conn = self.conn();
        conn.execute(
            "UPDATE items SET name = ?1 WHERE id = ?2",
            params![placeholder_name, item_id.0.to_string()],
        )?;
        Ok(())
    }

    // --- STORE CRUD ---

    pub fn insert_store(&self, store: &Store) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT OR REPLACE INTO stores (id, name) VALUES (?1, ?2)",
            params![store.id.0.to_string(), store.name],
        )?;
        Ok(())
    }

    pub fn get_all_stores(&self) -> Result<Vec<Store>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT id, name FROM stores")?;
        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let name: String = row.get(1)?;
            let id = StoreId(parse_uuid(&id_str)?);
            Ok(Store { id, name })
        })?;

        let mut stores = Vec::new();
        for r in rows {
            stores.push(r?);
        }
        Ok(stores)
    }

    pub fn delete_store(&self, store_id: StoreId) -> Result<()> {
        let conn = self.conn();
        conn.execute("DELETE FROM stores WHERE id = ?1", params![store_id.0.to_string()])?;
        Ok(())
    }

    // --- PACKAGE CRUD ---

    pub fn insert_package(&self, pkg: &Package) -> Result<()> {
        let conn = self.conn();
        let unit_str = serde_json::to_string(&pkg.quantity.unit).unwrap_or_default();
        let last_seen_str = pkg.last_seen.map(|dt| dt.to_rfc3339());
        let is_pref = if pkg.is_preferred { 1 } else { 0 };

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
                is_pref
            ],
        )?;
        Ok(())
    }

    pub fn insert_packages_batch(&self, packages: &[Package]) -> Result<()> {
        self.with_transaction(|tx| {
            let mut stmt = tx.prepare(
                "INSERT OR REPLACE INTO packages (id, item_id, store_id, quantity_amount, quantity_unit, price, last_seen, is_preferred)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            )?;
            for pkg in packages {
                let unit_str = serde_json::to_string(&pkg.quantity.unit).unwrap_or_default();
                let last_seen_str = pkg.last_seen.map(|dt| dt.to_rfc3339());
                let is_pref = if pkg.is_preferred { 1 } else { 0 };
                stmt.execute(params![
                    pkg.id.0.to_string(),
                    pkg.item_id.0.to_string(),
                    pkg.store_id.0.to_string(),
                    pkg.quantity.amount.to_string(),
                    unit_str,
                    pkg.price.to_string(),
                    last_seen_str,
                    is_pref
                ])?;
            }
            Ok(())
        })
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
            let is_pref_int: i32 = row.get(7)?;

            let id = PackageId(parse_uuid(&id_str)?);
            let item_id = ItemId(parse_uuid(&item_id_str)?);
            let store_id = StoreId(parse_uuid(&store_id_str)?);
            let amount = Decimal::from_str(&amount_str).unwrap_or(Decimal::ONE);
            let unit: Unit = serde_json::from_str(&unit_str).unwrap_or(Unit::Gram);
            let price = Decimal::from_str(&price_str).unwrap_or(Decimal::ZERO);
            let last_seen = last_seen_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc)));
            let is_preferred = is_pref_int != 0;

            Ok(Package {
                id,
                item_id,
                store_id,
                quantity: Quantity { amount, unit },
                price,
                last_seen,
                is_preferred,
            })
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
            let is_pref_int: i32 = row.get(7)?;

            let id = PackageId(parse_uuid(&id_str)?);
            let item_id = ItemId(parse_uuid(&item_id_str)?);
            let store_id = StoreId(parse_uuid(&store_id_str)?);
            let amount = Decimal::from_str(&amount_str).unwrap_or(Decimal::ONE);
            let unit: Unit = serde_json::from_str(&unit_str).unwrap_or(Unit::Gram);
            let price = Decimal::from_str(&price_str).unwrap_or(Decimal::ZERO);
            let last_seen = last_seen_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.with_timezone(&Utc)));
            let is_preferred = is_pref_int != 0;

            Ok(Package {
                id,
                item_id,
                store_id,
                quantity: Quantity { amount, unit },
                price,
                last_seen,
                is_preferred,
            })
        })?;

        let mut packages = Vec::new();
        for r in rows {
            packages.push(r?);
        }
        Ok(packages)
    }

    pub fn delete_package(&self, package_id: PackageId) -> Result<()> {
        let conn = self.conn();
        conn.execute("DELETE FROM packages WHERE id = ?1", params![package_id.0.to_string()])?;
        Ok(())
    }

    // --- GLOBAL SUBSTITUTES & UNIT BRIDGES ---

    pub fn insert_global_substitute(&self, primary: ItemId, substitute: ItemId) -> Result<()> {
        let conn = self.conn();
        conn.execute(
            "INSERT OR REPLACE INTO global_substitutes (primary_item_id, substitute_item_id) VALUES (?1, ?2)",
            params![primary.0.to_string(), substitute.0.to_string()],
        )?;
        Ok(())
    }

    pub fn get_global_substitute(&self, primary: ItemId) -> Result<Option<ItemId>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT substitute_item_id FROM global_substitutes WHERE primary_item_id = ?1")?;
        let mut rows = stmt.query(params![primary.0.to_string()])?;
        if let Some(row) = rows.next()? {
            let id_str: String = row.get(0)?;
            Ok(Some(ItemId(parse_uuid(&id_str)?)))
        } else {
            Ok(None)
        }
    }

    pub fn get_all_global_substitutes(&self) -> Result<Vec<(ItemId, ItemId)>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT primary_item_id, substitute_item_id FROM global_substitutes")?;
        let rows = stmt.query_map([], |row| {
            let p_str: String = row.get(0)?;
            let s_str: String = row.get(1)?;
            let p_id = ItemId(parse_uuid(&p_str)?);
            let s_id = ItemId(parse_uuid(&s_str)?);
            Ok((p_id, s_id))
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn delete_global_substitute(&self, primary: ItemId) -> Result<()> {
        let conn = self.conn();
        conn.execute("DELETE FROM global_substitutes WHERE primary_item_id = ?1", params![primary.0.to_string()])?;
        Ok(())
    }

    pub fn get_all_unit_bridges(&self) -> Result<Vec<UnitBridge>> {
        let conn = self.conn();
        let mut stmt = conn.prepare("SELECT item_id, from_amount, from_unit, to_amount, to_unit FROM unit_bridges")?;
        let rows = stmt.query_map([], |row| {
            let id_str: String = row.get(0)?;
            let from_amt_str: String = row.get(1)?;
            let from_unit_str: String = row.get(2)?;
            let to_amt_str: String = row.get(3)?;
            let to_unit_str: String = row.get(4)?;

            let item_id = ItemId(parse_uuid(&id_str)?);
            let from_amount = Decimal::from_str(&from_amt_str).unwrap_or(Decimal::ONE);
            let from_unit: Unit = serde_json::from_str(&from_unit_str).unwrap_or(Unit::Each);
            let to_amount = Decimal::from_str(&to_amt_str).unwrap_or(Decimal::ONE);
            let to_unit: Unit = serde_json::from_str(&to_unit_str).unwrap_or(Unit::Gram);

            let from_qty = Quantity { amount: from_amount, unit: from_unit };
            let to_qty = Quantity { amount: to_amount, unit: to_unit };

            Ok(UnitBridge {
                item_id,
                from_qty,
                to_qty,
            })
        })?;

        let mut bridges = Vec::new();
        for r in rows {
            bridges.push(r?);
        }
        Ok(bridges)
    }
}
