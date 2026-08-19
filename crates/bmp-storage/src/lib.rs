pub mod db;
pub mod migrations;
pub mod repos;

pub use db::*;
pub use migrations::*;
pub use repos::*;

#[cfg(test)]
mod tests {
    use super::*;
    use bmp_domain::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_storage_items_and_recipes() {
        let storage = Storage::in_memory().unwrap();

        let item = Item::new("Flour")
            .with_density(Density::new(dec!(0.53)).unwrap())
            .with_category("Baking");
        let item_id = item.id;

        storage.insert_item(&item).unwrap();
        let fetched = storage.get_item(item_id).unwrap().unwrap();
        assert_eq!(fetched.name, "Flour");
        assert_eq!(fetched.density.unwrap().g_per_ml, dec!(0.53));

        let store = Store::new("Walmart");
        let store_id = store.id;
        storage.insert_store(&store).unwrap();

        let pkg = Package::new(
            item_id,
            store_id,
            Quantity::new(dec!(5), Unit::Pound).unwrap(),
            dec!(3.48),
        );
        storage.insert_package(&pkg).unwrap();

        let item_pkgs = storage.get_packages_for_item(item_id).unwrap();
        assert_eq!(item_pkgs.len(), 1);
        assert_eq!(item_pkgs[0].price, dec!(3.48));
    }

    #[test]
    fn test_count_bridge_persistence() {
        let storage = Storage::in_memory().unwrap();

        let mut item = Item::new("Apple").with_category("Produce");
        let bridge = UnitBridge::new(
            item.id,
            Quantity::new(dec!(1), Unit::Each).unwrap(),
            Quantity::new(dec!(180), Unit::Gram).unwrap(),
        )
        .unwrap();
        item = item.with_count_bridge(bridge.clone());
        let item_id = item.id;

        storage.insert_item(&item).unwrap();

        let fetched = storage.get_item(item_id).unwrap().unwrap();
        assert!(fetched.count_bridge.is_some());
        let fetched_bridge = fetched.count_bridge.unwrap();
        assert_eq!(fetched_bridge.from_qty.amount, dec!(1));
        assert_eq!(fetched_bridge.from_qty.unit, Unit::Each);
        assert_eq!(fetched_bridge.to_qty.amount, dec!(180));
        assert_eq!(fetched_bridge.to_qty.unit, Unit::Gram);
    }

    #[test]
    fn test_versioned_migrations_and_indexes() {
        let storage = Storage::in_memory().unwrap();
        let conn = storage.conn();

        // Check _schema_migrations has 3 versions applied
        let mut stmt = conn.prepare("SELECT version, description FROM _schema_migrations ORDER BY version ASC").unwrap();
        let versions: Vec<(u32, String)> = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?))).unwrap().filter_map(|r| r.ok()).collect();
        assert!(versions.len() >= 3);
        assert_eq!(versions[0].0, 1);
        assert_eq!(versions[1].0, 2);
        assert_eq!(versions[2].0, 3);

        // Check indexes exist
        let mut idx_stmt = conn.prepare("SELECT name FROM sqlite_master WHERE type = 'index' AND name LIKE 'idx_%'").unwrap();
        let indexes: Vec<String> = idx_stmt.query_map([], |row| row.get(0)).unwrap().filter_map(|r| r.ok()).collect();
        assert!(indexes.contains(&"idx_packages_item_id".to_string()));
        assert!(indexes.contains(&"idx_packages_store_id".to_string()));
        assert!(indexes.contains(&"idx_recipe_yields_recipe_id".to_string()));
        assert!(indexes.contains(&"idx_ingredient_edges_recipe_id".to_string()));
        assert!(indexes.contains(&"idx_pantry_entries_item_id".to_string()));
        assert!(indexes.contains(&"idx_scheduled_meals_datetime".to_string()));
        assert!(indexes.contains(&"idx_receipts_datetime".to_string()));
    }

    #[test]
    fn test_transaction_atomicity_and_rollback() {
        let storage = Storage::in_memory().unwrap();
        let item = Item::new("Butter");
        let item_id = item.id;

        // Transaction that fails should rollback completely
        let res: rusqlite::Result<()> = storage.with_transaction(|tx| {
            tx.execute(
                "INSERT INTO items (id, name, preferred_purchase_mode) VALUES (?1, ?2, 'BuyFinished')",
                rusqlite::params![item_id.0.to_string(), "Butter"],
            )?;
            // Intentional syntax error to trigger rollback
            tx.execute("INVALID SQL STATEMENT", [])?;
            Ok(())
        });

        assert!(res.is_err());
        let fetched = storage.get_item(item_id).unwrap();
        assert!(fetched.is_none(), "Item should not exist after transaction rollback");

        // Transaction that succeeds should persist
        storage.with_transaction(|tx| {
            tx.execute(
                "INSERT INTO items (id, name, preferred_purchase_mode) VALUES (?1, ?2, '\"BuyFinished\"')",
                rusqlite::params![item_id.0.to_string(), "Butter"],
            )?;
            Ok(())
        }).unwrap();

        let fetched_after = storage.get_item(item_id).unwrap();
        assert!(fetched_after.is_some(), "Item should exist after successful transaction commit");
    }
}
