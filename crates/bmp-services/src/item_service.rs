use bmp_domain::*;
use bmp_storage::Storage;
use rust_decimal::Decimal;

pub struct ItemService {
    storage: Storage,
}

impl ItemService {
    pub fn new(storage: Storage) -> Self {
        Self { storage }
    }

    pub fn create_item(&self, name: &str, density: Option<Decimal>, category: Option<&str>) -> Result<Item, String> {
        let mut item = Item::new(name);
        if let Some(d) = density {
            let den = Density::new(d).map_err(|e| e.to_string())?;
            item = item.with_density(den);
        }
        if let Some(cat) = category {
            item = item.with_category(cat);
        }
        self.storage.insert_item(&item).map_err(|e| e.to_string())?;
        Ok(item)
    }

    pub fn add_package(
        &self,
        item_id: ItemId,
        store_id: StoreId,
        amount: Decimal,
        unit: Unit,
        price: Decimal,
    ) -> Result<Package, String> {
        let qty = Quantity::new(amount, unit).map_err(|e| e.to_string())?;
        let pkg = Package::new(item_id, store_id, qty, price);
        self.storage.insert_package(&pkg).map_err(|e| e.to_string())?;
        Ok(pkg)
    }

    pub fn list_items(&self) -> Result<Vec<Item>, String> {
        self.storage.get_all_items().map_err(|e| e.to_string())
    }

    pub fn list_stores(&self) -> Result<Vec<Store>, String> {
        self.storage.get_all_stores().map_err(|e| e.to_string())
    }

    pub fn add_store(&self, name: &str) -> Result<Store, String> {
        let store = Store::new(name);
        self.storage.insert_store(&store).map_err(|e| e.to_string())?;
        Ok(store)
    }
}
