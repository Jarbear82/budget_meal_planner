pub mod db;
pub mod migrations;
pub mod repo;

pub use db::*;
pub use migrations::*;

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
}
