use crate::error::ServiceResult;
use crate::event_bus::EventBus;
use bmp_domain::*;
use bmp_storage::Storage;
use rust_decimal::Decimal;

pub struct ItemService {
    storage: Storage,
    event_bus: EventBus,
}

impl ItemService {
    pub fn new(storage: Storage, event_bus: EventBus) -> Self {
        Self { storage, event_bus }
    }

    pub fn new_with_storage(storage: Storage) -> Self {
        Self::new(storage, EventBus::default())
    }

    pub fn create_item(&self, name: &str, density: Option<Decimal>, category: Option<&str>) -> ServiceResult<Item> {
        let mut item = Item::new(name);
        if let Some(d) = density {
            let den = Density::new(d)?;
            item = item.with_density(den);
        }
        if let Some(cat) = category {
            item = item.with_category(cat);
        }
        self.storage.insert_item(&item)?;
        self.event_bus.publish(DomainEvent::ItemCreated(item.id));
        Ok(item)
    }

    pub fn add_package(
        &self,
        item_id: ItemId,
        store_id: StoreId,
        amount: Decimal,
        unit: Unit,
        price: Decimal,
    ) -> ServiceResult<Package> {
        let qty = Quantity::new(amount, unit)?;
        let pkg = Package::new(item_id, store_id, qty, price);
        self.storage.insert_package(&pkg)?;
        self.event_bus.publish(DomainEvent::PackageCreated(pkg.id));
        Ok(pkg)
    }

    pub fn upsert_packages_batch(&self, packages: &[Package]) -> ServiceResult<()> {
        self.storage.insert_packages_batch(packages)?;
        let ids: Vec<PackageId> = packages.iter().map(|p| p.id).collect();
        self.event_bus.publish(DomainEvent::PackagesBulkUpdated(ids));
        Ok(())
    }

    pub fn list_items(&self) -> ServiceResult<Vec<Item>> {
        Ok(self.storage.get_all_items()?)
    }

    pub fn get_item(&self, item_id: ItemId) -> ServiceResult<Option<Item>> {
        Ok(self.storage.get_item(item_id)?)
    }

    pub fn list_stores(&self) -> ServiceResult<Vec<Store>> {
        Ok(self.storage.get_all_stores()?)
    }

    pub fn add_store(&self, name: &str) -> ServiceResult<Store> {
        let store = Store::new(name);
        self.storage.insert_store(&store)?;
        self.event_bus.publish(DomainEvent::StoreCreated(store.id));
        Ok(store)
    }

    pub fn update_item(&self, item: &Item) -> ServiceResult<()> {
        self.storage.insert_item(item)?;
        self.event_bus.publish(DomainEvent::ItemUpdated(item.id));
        Ok(())
    }

    pub fn delete_item(&self, item_id: ItemId) -> ServiceResult<()> {
        self.storage.delete_item(item_id)?;
        self.event_bus.publish(DomainEvent::ItemDeleted(item_id));
        Ok(())
    }

    pub fn update_package(&self, package: &Package) -> ServiceResult<()> {
        self.storage.insert_package(package)?;
        self.event_bus.publish(DomainEvent::PackageUpdated(package.id));
        Ok(())
    }

    pub fn move_package_to_store(&self, package_id: PackageId, new_store_id: StoreId) -> ServiceResult<()> {
        let pkgs = self.storage.get_all_packages()?;
        if let Some(mut pkg) = pkgs.into_iter().find(|p| p.id == package_id) {
            pkg.store_id = new_store_id;
            pkg.last_seen = Some(chrono::Utc::now());
            self.storage.insert_package(&pkg)?;
            self.event_bus.publish(DomainEvent::PackageUpdated(package_id));
        }
        Ok(())
    }

    pub fn update_package_price(&self, package_id: PackageId, new_price: Decimal) -> ServiceResult<()> {
        let pkgs = self.storage.get_all_packages()?;
        if let Some(mut pkg) = pkgs.into_iter().find(|p| p.id == package_id) {
            pkg.price = new_price;
            pkg.last_seen = Some(chrono::Utc::now());
            self.storage.insert_package(&pkg)?;
            self.event_bus.publish(DomainEvent::PackageUpdated(package_id));
        }
        Ok(())
    }

    pub fn get_packages_for_item(&self, item_id: ItemId) -> ServiceResult<Vec<Package>> {
        Ok(self.storage.get_packages_for_item(item_id)?)
    }

    pub fn delete_package(&self, package_id: PackageId) -> ServiceResult<()> {
        self.storage.delete_package(package_id)?;
        self.event_bus.publish(DomainEvent::PackageDeleted(package_id));
        Ok(())
    }

    pub fn set_global_substitute(&self, primary: ItemId, substitute: ItemId) -> ServiceResult<()> {
        self.storage.insert_global_substitute(primary, substitute)?;
        self.event_bus.publish(DomainEvent::ItemUpdated(primary));
        Ok(())
    }

    pub fn get_global_substitute(&self, primary: ItemId) -> ServiceResult<Option<ItemId>> {
        Ok(self.storage.get_global_substitute(primary)?)
    }

    pub fn list_global_substitutes(&self) -> ServiceResult<Vec<(ItemId, ItemId)>> {
        Ok(self.storage.get_all_global_substitutes()?)
    }

    pub fn delete_global_substitute(&self, primary: ItemId) -> ServiceResult<()> {
        self.storage.delete_global_substitute(primary)?;
        self.event_bus.publish(DomainEvent::ItemUpdated(primary));
        Ok(())
    }

    pub fn list_unit_bridges(&self) -> ServiceResult<Vec<UnitBridge>> {
        Ok(self.storage.get_all_unit_bridges()?)
    }
}
