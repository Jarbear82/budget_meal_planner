use crate::error::DomainError;
use crate::id::{ItemId, PackageId, StoreId};
use crate::item::{Item, PurchaseMode};
use crate::package::Package;
use crate::pantry::PantryEntry;
use crate::units::Quantity;
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingLineItem {
    pub item_id: ItemId,
    pub item_name: String,
    pub store_id: StoreId,
    pub package_id: PackageId,
    pub required_qty: Quantity,
    pub package_qty: Quantity,
    pub package_count: u32,
    pub package_price: Decimal,
    pub line_total: Decimal,
    pub is_checked: bool,
    pub purchase_mode: PurchaseMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShoppingList {
    pub items: Vec<ShoppingLineItem>,
    pub subtotal: Decimal,
    pub tax_rate: Option<Decimal>,
    pub total: Decimal,
}

pub fn generate_shopping_list(
    item_requirements: Vec<(ItemId, Quantity)>,
    items_map: &HashMap<ItemId, Item>,
    packages_map: &HashMap<ItemId, Vec<Package>>,
    pantry_entries: &[PantryEntry],
    selected_store_id: Option<StoreId>,
    tax_rate: Option<Decimal>,
) -> Result<ShoppingList, DomainError> {
    // 1. Consolidate requirements per ItemId
    let mut consolidated: HashMap<ItemId, Quantity> = HashMap::new();
    for (item_id, qty) in item_requirements {
        let item_opt = items_map.get(&item_id);
        consolidated
            .entry(item_id)
            .and_modify(|existing| {
                if existing.unit == qty.unit {
                    existing.amount += qty.amount;
                } else if let Some(item) = item_opt {
                    if let Ok(converted) = item.convert_quantity(&qty, &existing.unit) {
                        existing.amount += converted.amount;
                    } else {
                        existing.amount += qty.amount;
                    }
                } else {
                    existing.amount += qty.amount;
                }
            })
            .or_insert(qty);
    }

    // 2. Subtract Pantry quantities
    for pantry in pantry_entries {
        if let Some(req) = consolidated.get_mut(&pantry.item_id) {
            let item_opt = items_map.get(&pantry.item_id);
            let pantry_qty_in_req_unit = if req.unit == pantry.quantity.unit {
                Some(pantry.quantity.amount)
            } else if let Some(item) = item_opt {
                item.convert_quantity(&pantry.quantity, &req.unit).ok().map(|q| q.amount)
            } else {
                None
            };

            if let Some(avail) = pantry_qty_in_req_unit {
                if req.amount > avail {
                    req.amount -= avail;
                } else {
                    req.amount = Decimal::ZERO;
                }
            }
        }
    }

    let mut line_items = Vec::new();

    // 3. For each item with remaining requirement, find best matching package
    for (item_id, req_qty) in consolidated {
        if req_qty.amount <= Decimal::ZERO {
            continue;
        }

        let item = match items_map.get(&item_id) {
            Some(i) => i,
            None => continue,
        };

        let available_pkgs = match packages_map.get(&item_id) {
            Some(pkgs) if !pkgs.is_empty() => pkgs,
            _ => continue,
        };

        // Filter by selected store if specified
        let store_pkgs: Vec<&Package> = if let Some(sid) = selected_store_id {
            available_pkgs.iter().filter(|p| p.store_id == sid).collect()
        } else {
            available_pkgs.iter().collect()
        };

        if store_pkgs.is_empty() {
            continue;
        }

        // Helper for normalized price-per-base-unit calculation (g/ml comparison)
        let get_normalized_unit_cost = |pkg: &Package| -> Decimal {
            let base_unit = match pkg.quantity.unit {
                crate::units::Unit::Kilogram | crate::units::Unit::Gram | crate::units::Unit::Ounce | crate::units::Unit::Pound => crate::units::Unit::Gram,
                crate::units::Unit::Liter | crate::units::Unit::Milliliter | crate::units::Unit::Cup | crate::units::Unit::Tablespoon | crate::units::Unit::Teaspoon => crate::units::Unit::Milliliter,
                _ => pkg.quantity.unit.clone(),
            };
            if let Ok(base_qty) = item.convert_quantity(&pkg.quantity, &base_unit) {
                if base_qty.amount > Decimal::ZERO {
                    pkg.price / base_qty.amount
                } else {
                    pkg.price / pkg.quantity.amount
                }
            } else {
                pkg.price / pkg.quantity.amount
            }
        };

        // Pick preferred package or lowest density-normalized unit cost
        let chosen_pkg = store_pkgs
            .iter()
            .find(|p| p.is_preferred)
            .copied()
            .unwrap_or_else(|| {
                store_pkgs
                    .iter()
                    .min_by(|a, b| {
                        let cost_a = get_normalized_unit_cost(a);
                        let cost_b = get_normalized_unit_cost(b);
                        cost_a.cmp(&cost_b)
                    })
                    .copied()
                    .unwrap()
            });

        // Convert required quantity into package unit for correct ceiling rounding ratio
        let req_in_pkg_unit = item
            .convert_quantity(&req_qty, &chosen_pkg.quantity.unit)
            .unwrap_or_else(|_| req_qty.clone());
        let ratio = req_in_pkg_unit.amount / chosen_pkg.quantity.amount;
        let package_count = ratio.ceil().to_u32().unwrap_or(1).max(1);
        let line_total = chosen_pkg.price * Decimal::from(package_count);

        line_items.push(ShoppingLineItem {
            item_id,
            item_name: item.name.clone(),
            store_id: chosen_pkg.store_id,
            package_id: chosen_pkg.id,
            required_qty: req_qty,
            package_qty: chosen_pkg.quantity.clone(),
            package_count,
            package_price: chosen_pkg.price,
            line_total,
            is_checked: false,
            purchase_mode: item.preferred_purchase_mode,
        });
    }

    let subtotal: Decimal = line_items.iter().map(|item| item.line_total).sum();
    let total = if let Some(tax) = tax_rate {
        subtotal * (Decimal::ONE + tax)
    } else {
        subtotal
    };

    Ok(ShoppingList {
        items: line_items,
        subtotal,
        tax_rate,
        total,
    })
}
