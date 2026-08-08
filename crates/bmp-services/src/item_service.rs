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

    pub fn get_item(&self, item_id: ItemId) -> Result<Option<Item>, String> {
        self.storage.get_item(item_id).map_err(|e| e.to_string())
    }

    pub fn list_stores(&self) -> Result<Vec<Store>, String> {
        self.storage.get_all_stores().map_err(|e| e.to_string())
    }

    pub fn add_store(&self, name: &str) -> Result<Store, String> {
        let store = Store::new(name);
        self.storage.insert_store(&store).map_err(|e| e.to_string())?;
        Ok(store)
    }

    pub fn update_item(&self, item: &Item) -> Result<(), String> {
        self.storage.insert_item(item).map_err(|e| e.to_string())
    }

    pub fn delete_item(&self, item_id: ItemId) -> Result<(), String> {
        self.storage.delete_item(item_id).map_err(|e| e.to_string())
    }

    pub fn update_package(&self, package: &Package) -> Result<(), String> {
        self.storage.insert_package(package).map_err(|e| e.to_string())
    }

    pub fn move_package_to_store(&self, package_id: PackageId, new_store_id: StoreId) -> Result<(), String> {
        let pkgs = self.storage.get_all_packages().map_err(|e| e.to_string())?;
        if let Some(mut pkg) = pkgs.into_iter().find(|p| p.id == package_id) {
            pkg.store_id = new_store_id;
            pkg.last_seen = Some(chrono::Utc::now());
            self.storage.insert_package(&pkg).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn update_package_price(&self, package_id: PackageId, new_price: Decimal) -> Result<(), String> {
        let pkgs = self.storage.get_all_packages().map_err(|e| e.to_string())?;
        if let Some(mut pkg) = pkgs.into_iter().find(|p| p.id == package_id) {
            pkg.price = new_price;
            pkg.last_seen = Some(chrono::Utc::now());
            self.storage.insert_package(&pkg).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    pub fn get_packages_for_item(&self, item_id: ItemId) -> Result<Vec<Package>, String> {
        self.storage.get_packages_for_item(item_id).map_err(|e| e.to_string())
    }

    pub fn delete_package(&self, package_id: PackageId) -> Result<(), String> {
        self.storage.delete_package(package_id).map_err(|e| e.to_string())
    }
}
