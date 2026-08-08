use crate::components::*;
use bmp_domain::*;
use bmp_services::AppServices;
use gpui::prelude::*;
use gpui::*;
use gpui_component::alert::Alert;
use gpui_component::badge::Badge;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::dialog::{DialogDescription, DialogFooter, DialogHeader, DialogTitle};
use gpui_component::scroll::ScrollableElement;
use gpui_component::tag::Tag;
use gpui_component::WindowExt;
use gpui_component::ActiveTheme;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::str::FromStr;

pub struct ItemsView {
    pub services: AppServices,
    pub search_query: String,
    pub status_msg: String,

    pub cached_items: Vec<Item>,
    pub cached_stores: Vec<Store>,
    pub cached_packages: Vec<Package>,

    // Selected Item for detail view / editing
    pub selected_item_id: Option<ItemId>,

    // Item modal form state
    pub editing_item_id: Option<ItemId>,
    pub item_form_name: String,
    pub item_form_density: Decimal,
    pub item_form_category: String,
    pub item_form_mode: PurchaseMode,

    // Package modal form state
    pub pkg_form_item_id: Option<ItemId>,
    pub pkg_form_store_id: Option<StoreId>,
    pub pkg_form_amount: Decimal,
    pub pkg_form_unit: Unit,
    pub pkg_form_price: Decimal,
    pub pkg_form_preferred: bool,

    // Store modal form state
    pub store_form_name: String,

    // Bridge modal form state
    pub bridge_form_item_id: Option<ItemId>,
    pub bridge_from_amount: Decimal,
    pub bridge_from_unit: Unit,
    pub bridge_to_amount: Decimal,
    pub bridge_to_unit: Unit,
}

impl ItemsView {
    pub fn new(services: AppServices) -> Self {
        let mut view = Self {
            services,
            search_query: String::new(),
            status_msg: "Items matrix ready".to_string(),
            cached_items: Vec::new(),
            cached_stores: Vec::new(),
            cached_packages: Vec::new(),
            selected_item_id: None,

            editing_item_id: None,
            item_form_name: String::new(),
            item_form_density: dec!(1.0),
            item_form_category: "Pantry".to_string(),
            item_form_mode: PurchaseMode::BuyFinished,

            pkg_form_item_id: None,
            pkg_form_store_id: None,
            pkg_form_amount: dec!(500),
            pkg_form_unit: Unit::Gram,
            pkg_form_price: dec!(4.99),
            pkg_form_preferred: true,

            store_form_name: String::new(),

            bridge_form_item_id: None,
            bridge_from_amount: dec!(1),
            bridge_from_unit: Unit::Each,
            bridge_to_amount: dec!(150),
            bridge_to_unit: Unit::Gram,
        };
        view.reload_data();
        view
    }

    pub fn reload_data(&mut self) {
        self.cached_items = self.services.items.list_items().unwrap_or_default();
        self.cached_stores = self.services.items.list_stores().unwrap_or_default();
        if let Some(id) = self.selected_item_id {
            self.cached_packages = self.services.items.get_packages_for_item(id).unwrap_or_default();
        } else {
            self.cached_packages.clear();
        }
    }

    pub fn open_create_item_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editing_item_id = None;
        self.item_form_name = String::new();
        self.item_form_density = dec!(1.0);
        self.item_form_category = "General".to_string();
        self.item_form_mode = PurchaseMode::BuyFinished;
        self.show_item_dialog(window, cx);
    }

    pub fn open_edit_item_modal(&mut self, item: &Item, window: &mut Window, cx: &mut Context<Self>) {
        self.editing_item_id = Some(item.id);
        self.item_form_name = item.name.clone();
        self.item_form_density = item.density.map(|d| d.g_per_ml).unwrap_or(dec!(1.0));
        self.item_form_category = item.category.clone().unwrap_or_else(|| "General".to_string());
        self.item_form_mode = item.preferred_purchase_mode;
        self.show_item_dialog(window, cx);
    }

    fn show_item_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let view = cx.entity().clone();
        window.open_dialog(cx, move |dialog, _, _| {
            let view = view.clone();
            dialog
                .w(px(500.))
                .content(move |content, _, cx| {
                    let view_read = view.read(cx);
                    let is_edit = view_read.editing_item_id.is_some();
                    let title = if is_edit { "Edit Domain Item" } else { "Add Domain Item" };

                    let mode_options = vec![
                        SelectOption::new("BuyFinished", "Buy Finished Package"),
                        SelectOption::new("PreferMake", "Prefer Make / Expand"),
                        SelectOption::new("AskEveryTime", "Ask Every Time"),
                    ];

                    let form_name = view_read.item_form_name.clone();
                    let form_category = view_read.item_form_category.clone();
                    let form_density = view_read.item_form_density;
                    let form_mode = view_read.item_form_mode;

                    let v_num = view.clone();
                    let v_mode = view.clone();
                    let v_save = view.clone();

                    content
                        .child(
                            DialogHeader::new()
                                .child(DialogTitle::new().child(title))
                                .child(DialogDescription::new().child("Configure density (g/ml), category, and default purchase mode")),
                        )
                        .child(
                            div()
                                .py_4()
                                .flex()
                                .flex_col()
                                .gap_3()
                                .child(
                                    FormInput::new("input-item-name")
                                        .label("Item Name")
                                        .placeholder("e.g. Extra Virgin Olive Oil")
                                        .value(form_name),
                                )
                                .child(
                                    FormInput::new("input-item-category")
                                        .label("Category")
                                        .placeholder("e.g. Oils & Fats, Pantry, Dairy")
                                        .value(form_category),
                                )
                                .child(
                                    NumberInput::new("input-item-density", form_density)
                                        .label("Density (g/ml)")
                                        .step(dec!(0.05))
                                        .unit("g/ml")
                                        .on_increment({
                                            let v = v_num.clone();
                                            move |val, _window, cx| {
                                                v.update(cx, |this, cx| {
                                                    this.item_form_density = *val;
                                                    cx.notify();
                                                });
                                            }
                                        })
                                        .on_decrement({
                                            let v = v_num.clone();
                                            move |val, _window, cx| {
                                                v.update(cx, |this, cx| {
                                                    this.item_form_density = *val;
                                                    cx.notify();
                                                });
                                            }
                                        }),
                                )
                                .child(
                                    Select::new("select-item-purchase-mode", mode_options)
                                        .label("Preferred Purchase Mode")
                                        .selected_id(Some(format!("{:?}", form_mode)))
                                        .on_select(move |opt: &SelectOption, _window, cx| {
                                            let mode = match opt.id.as_str() {
                                                "PreferMake" => PurchaseMode::PreferMake,
                                                "AskEveryTime" => PurchaseMode::AskEveryTime,
                                                _ => PurchaseMode::BuyFinished,
                                            };
                                            v_mode.update(cx, |this, cx| {
                                                this.item_form_mode = mode;
                                                cx.notify();
                                            });
                                        }),
                                ),
                        )
                        .child(
                            DialogFooter::new()
                                .child(
                                    Button::new("btn-cancel-item-modal")
                                        .secondary()
                                        .label("Cancel")
                                        .on_click(|_, window, cx| {
                                            window.close_dialog(cx);
                                        }),
                                )
                                .child(
                                    Button::new("btn-save-item-modal")
                                        .primary()
                                        .label("Save Item")
                                        .on_click(move |_, window, cx| {
                                            v_save.update(cx, |this, cx| {
                                                this.save_item(cx);
                                            });
                                            window.close_dialog(cx);
                                        }),
                                ),
                        )
                })
        });
    }

    pub fn save_item(&mut self, cx: &mut Context<Self>) {
        if self.item_form_name.trim().is_empty() {
            self.status_msg = "Error: Item name cannot be empty".to_string();
            cx.notify();
            return;
        }

        let density_opt = if self.item_form_density > Decimal::ZERO {
            Some(self.item_form_density)
        } else {
            None
        };

        if let Some(item_id) = self.editing_item_id {
            if let Ok(mut items) = self.services.items.list_items() {
                if let Some(item) = items.iter_mut().find(|i| i.id == item_id) {
                    item.name = self.item_form_name.trim().to_string();
                    item.category = Some(self.item_form_category.trim().to_string());
                    item.preferred_purchase_mode = self.item_form_mode;
                    if let Some(d) = density_opt {
                        if let Ok(den) = Density::new(d) {
                            item.density = Some(den);
                        }
                    } else {
                        item.density = None;
                    }

                    match self.services.items.update_item(item) {
                        Ok(_) => {
                            self.status_msg = format!("Updated item: {}", item.name);
                        }
                        Err(e) => {
                            self.status_msg = format!("Error updating item: {}", e);
                        }
                    }
                }
            }
        } else {
            match self.services.items.create_item(
                self.item_form_name.trim(),
                density_opt,
                Some(self.item_form_category.trim()),
            ) {
                Ok(item) => {
                    self.status_msg = format!("Created item: {}", item.name);
                }
                Err(e) => {
                    self.status_msg = format!("Error creating item: {}", e);
                }
            }
        }
        self.reload_data();
        cx.notify();
    }

    pub fn delete_item(&mut self, item_id: ItemId, cx: &mut Context<Self>) {
        match self.services.items.delete_item(item_id) {
            Ok(_) => {
                self.status_msg = "Deleted item successfully".to_string();
                if self.selected_item_id == Some(item_id) {
                    self.selected_item_id = None;
                }
            }
            Err(e) => {
                self.status_msg = format!("Error deleting item: {}", e);
            }
        }
        self.reload_data();
        cx.notify();
    }

    pub fn open_add_package_modal(&mut self, item_id: ItemId, window: &mut Window, cx: &mut Context<Self>) {
        self.pkg_form_item_id = Some(item_id);
        let stores = &self.cached_stores;
        self.pkg_form_store_id = stores.first().map(|s| s.id);
        self.pkg_form_amount = dec!(500);
        self.pkg_form_unit = Unit::Gram;
        self.pkg_form_price = dec!(4.99);
        self.pkg_form_preferred = true;

        let view = cx.entity().clone();
        window.open_dialog(cx, move |dialog, _, _| {
            let view = view.clone();
            dialog
                .w(px(500.))
                .content(move |content, _, cx| {
                    let view_read = view.read(cx);
                    let stores = &view_read.cached_stores;
                    let store_options: Vec<SelectOption> = stores
                        .iter()
                        .map(|s| SelectOption::new(s.id.0.to_string(), s.name.clone()))
                        .collect();

                    let unit_options = vec![
                        SelectOption::new("Gram", "Gram (g)"),
                        SelectOption::new("Kilogram", "Kilogram (kg)"),
                        SelectOption::new("Milliliter", "Milliliter (ml)"),
                        SelectOption::new("Liter", "Liter (L)"),
                        SelectOption::new("Cup", "Cup"),
                        SelectOption::new("Tablespoon", "Tablespoon (tbsp)"),
                        SelectOption::new("Teaspoon", "Teaspoon (tsp)"),
                        SelectOption::new("Ounce", "Ounce (oz)"),
                        SelectOption::new("Pound", "Pound (lb)"),
                        SelectOption::new("Each", "Each (count)"),
                    ];

                    let pkg_store_id = view_read.pkg_form_store_id;
                    let pkg_amount = view_read.pkg_form_amount;
                    let pkg_unit = view_read.pkg_form_unit.clone();
                    let pkg_price = view_read.pkg_form_price;
                    let pkg_preferred = view_read.pkg_form_preferred;

                    let v_store = view.clone();
                    let v_amt = view.clone();
                    let v_unit = view.clone();
                    let v_price = view.clone();
                    let v_pref = view.clone();
                    let v_save = view.clone();

                    content
                        .child(
                            DialogHeader::new()
                                .child(DialogTitle::new().child("Add Store Package"))
                                .child(DialogDescription::new().child("Register a store price and quantity package for this item")),
                        )
                        .child(
                            div()
                                .py_4()
                                .flex()
                                .flex_col()
                                .gap_3()
                                .child(
                                    Select::new("select-pkg-store", store_options)
                                        .label("Store")
                                        .selected_id(pkg_store_id.map(|id| id.0.to_string()))
                                        .on_select(move |opt: &SelectOption, _window, cx| {
                                            if let Ok(uuid) = uuid::Uuid::from_str(&opt.id) {
                                                v_store.update(cx, |this, cx| {
                                                    this.pkg_form_store_id = Some(StoreId(uuid));
                                                    cx.notify();
                                                });
                                            }
                                        }),
                                )
                                .child(
                                    NumberInput::new("input-pkg-amount", pkg_amount)
                                        .label("Package Size / Amount")
                                        .step(dec!(50))
                                        .on_increment({
                                            let v = v_amt.clone();
                                            move |val, _window, cx| {
                                                v.update(cx, |this, cx| {
                                                    this.pkg_form_amount = *val;
                                                    cx.notify();
                                                });
                                            }
                                        })
                                        .on_decrement({
                                            let v = v_amt.clone();
                                            move |val, _window, cx| {
                                                v.update(cx, |this, cx| {
                                                    this.pkg_form_amount = *val;
                                                    cx.notify();
                                                });
                                            }
                                        }),
                                )
                                .child(
                                    Select::new("select-pkg-unit", unit_options)
                                        .label("Package Unit")
                                        .selected_id(Some(format!("{:?}", pkg_unit)))
                                        .on_select(move |opt: &SelectOption, _window, cx| {
                                            let unit = match opt.id.as_str() {
                                                "Kilogram" => Unit::Kilogram,
                                                "Milliliter" => Unit::Milliliter,
                                                "Liter" => Unit::Liter,
                                                "Cup" => Unit::Cup,
                                                "Tablespoon" => Unit::Tablespoon,
                                                "Teaspoon" => Unit::Teaspoon,
                                                "Ounce" => Unit::Ounce,
                                                "Pound" => Unit::Pound,
                                                "Each" => Unit::Each,
                                                _ => Unit::Gram,
                                            };
                                            v_unit.update(cx, |this, cx| {
                                                this.pkg_form_unit = unit;
                                                cx.notify();
                                            });
                                        }),
                                )
                                .child(
                                    NumberInput::new("input-pkg-price", pkg_price)
                                        .label("Package Price ($)")
                                        .step(dec!(0.50))
                                        .unit("$")
                                        .on_increment({
                                            let v = v_price.clone();
                                            move |val, _window, cx| {
                                                v.update(cx, |this, cx| {
                                                    this.pkg_form_price = *val;
                                                    cx.notify();
                                                });
                                            }
                                        })
                                        .on_decrement({
                                            let v = v_price.clone();
                                            move |val, _window, cx| {
                                                v.update(cx, |this, cx| {
                                                    this.pkg_form_price = *val;
                                                    cx.notify();
                                                });
                                            }
                                        }),
                                )
                                .child(
                                    Checkbox::new("cb-pkg-preferred")
                                        .label("Preferred store package for shopping calculations")
                                        .checked(pkg_preferred)
                                        .on_click(move |checked, _window, cx| {
                                            v_pref.update(cx, |this, cx| {
                                                this.pkg_form_preferred = *checked;
                                                cx.notify();
                                            });
                                        }),
                                ),
                        )
                        .child(
                            DialogFooter::new()
                                .child(
                                    Button::new("btn-cancel-pkg")
                                        .secondary()
                                        .label("Cancel")
                                        .on_click(|_, window, cx| {
                                            window.close_dialog(cx);
                                        }),
                                )
                                .child(
                                    Button::new("btn-save-pkg")
                                        .primary()
                                        .label("Add Package")
                                        .on_click(move |_, window, cx| {
                                            v_save.update(cx, |this, cx| {
                                                this.save_package(cx);
                                            });
                                            window.close_dialog(cx);
                                        }),
                                ),
                        )
                })
        });
    }

    pub fn save_package(&mut self, cx: &mut Context<Self>) {
        let item_id = match self.pkg_form_item_id {
            Some(id) => id,
            None => {
                self.status_msg = "Error: No item selected for package".to_string();
                return;
            }
        };

        let store_id = match self.pkg_form_store_id {
            Some(id) => id,
            None => {
                self.status_msg = "Error: Please register or select a store first".to_string();
                return;
            }
        };

        match self.services.items.add_package(
            item_id,
            store_id,
            self.pkg_form_amount,
            self.pkg_form_unit.clone(),
            self.pkg_form_price,
        ) {
            Ok(mut pkg) => {
                pkg.is_preferred = self.pkg_form_preferred;
                let _ = self.services.items.update_package(&pkg);
                self.status_msg = "Added package successfully".to_string();
            }
            Err(e) => {
                self.status_msg = format!("Error adding package: {}", e);
            }
        }
        self.reload_data();
        cx.notify();
    }

    pub fn delete_package(&mut self, pkg_id: PackageId, cx: &mut Context<Self>) {
        match self.services.items.delete_package(pkg_id) {
            Ok(_) => {
                self.status_msg = "Deleted package".to_string();
            }
            Err(e) => {
                self.status_msg = format!("Error deleting package: {}", e);
            }
        }
        self.reload_data();
        cx.notify();
    }

    pub fn toggle_package_preferred(&mut self, pkg_id: PackageId, cx: &mut Context<Self>) {
        if let Some(item_id) = self.selected_item_id {
            if let Ok(pkgs) = self.services.items.get_packages_for_item(item_id) {
                if let Some(mut pkg) = pkgs.into_iter().find(|p| p.id == pkg_id) {
                    pkg.is_preferred = !pkg.is_preferred;
                    let _ = self.services.items.update_package(&pkg);
                    self.status_msg = "Toggled package preferred flag".to_string();
                }
            }
        }
        self.reload_data();
        cx.notify();
    }

    pub fn open_add_store_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.store_form_name = String::new();

        let view = cx.entity().clone();
        window.open_dialog(cx, move |dialog, _, _| {
            let view = view.clone();
            dialog
                .w(px(500.))
                .content(move |content, _, cx| {
                    let view_read = view.read(cx);
                    let store_name = view_read.store_form_name.clone();

                    let v_save = view.clone();

                    content
                        .child(
                            DialogHeader::new()
                                .child(DialogTitle::new().child("Register New Store"))
                                .child(DialogDescription::new().child("Add a supermarket, grocery store, or supplier")),
                        )
                        .child(
                            div()
                                .py_4()
                                .child(
                                    FormInput::new("input-store-name")
                                        .label("Store Name")
                                        .placeholder("e.g. Costco, Trader Joe's, Safeway")
                                        .value(store_name),
                                ),
                        )
                        .child(
                            DialogFooter::new()
                                .child(
                                    Button::new("btn-cancel-store")
                                        .secondary()
                                        .label("Cancel")
                                        .on_click(|_, window, cx| {
                                            window.close_dialog(cx);
                                        }),
                                )
                                .child(
                                    Button::new("btn-save-store")
                                        .primary()
                                        .label("Register Store")
                                        .on_click(move |_, window, cx| {
                                            v_save.update(cx, |this, cx| {
                                                this.save_store(cx);
                                            });
                                            window.close_dialog(cx);
                                        }),
                                ),
                        )
                })
        });
    }

    pub fn save_store(&mut self, cx: &mut Context<Self>) {
        if self.store_form_name.trim().is_empty() {
            self.status_msg = "Error: Store name cannot be empty".to_string();
            return;
        }

        match self.services.items.add_store(self.store_form_name.trim()) {
            Ok(store) => {
                self.status_msg = format!("Registered store: {}", store.name);
            }
            Err(e) => {
                self.status_msg = format!("Error adding store: {}", e);
            }
        }
        self.reload_data();
        cx.notify();
    }

    pub fn open_add_bridge_modal(&mut self, item_id: ItemId, window: &mut Window, cx: &mut Context<Self>) {
        self.bridge_form_item_id = Some(item_id);
        self.bridge_from_amount = dec!(1);
        self.bridge_from_unit = Unit::Each;
        self.bridge_to_amount = dec!(150);
        self.bridge_to_unit = Unit::Gram;

        let view = cx.entity().clone();
        window.open_dialog(cx, move |dialog, _, _| {
            let view = view.clone();
            dialog
                .w(px(500.))
                .content(move |content, _, cx| {
                    let view_read = view.read(cx);
                    let unit_options = vec![
                        SelectOption::new("Gram", "Gram (g)"),
                        SelectOption::new("Kilogram", "Kilogram (kg)"),
                        SelectOption::new("Milliliter", "Milliliter (ml)"),
                        SelectOption::new("Liter", "Liter (L)"),
                        SelectOption::new("Cup", "Cup"),
                        SelectOption::new("Tablespoon", "Tablespoon (tbsp)"),
                        SelectOption::new("Teaspoon", "Teaspoon (tsp)"),
                        SelectOption::new("Ounce", "Ounce (oz)"),
                        SelectOption::new("Pound", "Pound (lb)"),
                        SelectOption::new("Each", "Each (count)"),
                    ];

                    let from_amt = view_read.bridge_from_amount;
                    let from_unit = view_read.bridge_from_unit.clone();
                    let to_amt = view_read.bridge_to_amount;
                    let to_unit = view_read.bridge_to_unit.clone();

                    let v_from_amt = view.clone();
                    let v_from_unit = view.clone();
                    let v_to_amt = view.clone();
                    let v_to_unit = view.clone();
                    let v_save = view.clone();

                    content
                        .child(
                            DialogHeader::new()
                                .child(DialogTitle::new().child("Configure Unit Bridge"))
                                .child(DialogDescription::new().child("Define custom count-to-mass or custom unit conversion for this item")),
                        )
                        .child(
                            div()
                                .py_4()
                                .flex()
                                .flex_col()
                                .gap_3()
                                .child(
                                    NumberInput::new("input-bridge-from-qty", from_amt)
                                        .label("From Quantity")
                                        .step(dec!(1))
                                        .on_increment({
                                            let v = v_from_amt.clone();
                                            move |val, _window, cx| {
                                                v.update(cx, |this, cx| {
                                                    this.bridge_from_amount = *val;
                                                    cx.notify();
                                                });
                                            }
                                        })
                                        .on_decrement({
                                            let v = v_from_amt.clone();
                                            move |val, _window, cx| {
                                                v.update(cx, |this, cx| {
                                                    this.bridge_from_amount = *val;
                                                    cx.notify();
                                                });
                                            }
                                        }),
                                )
                                .child(
                                    Select::new("select-bridge-from-unit", unit_options.clone())
                                        .label("From Unit")
                                        .selected_id(Some(format!("{:?}", from_unit)))
                                        .on_select(move |opt: &SelectOption, _window, cx| {
                                            let unit = match opt.id.as_str() {
                                                "Gram" => Unit::Gram,
                                                "Each" => Unit::Each,
                                                "Cup" => Unit::Cup,
                                                _ => Unit::Each,
                                            };
                                            v_from_unit.update(cx, |this, cx| {
                                                this.bridge_from_unit = unit;
                                                cx.notify();
                                            });
                                        }),
                                )
                                .child(
                                    NumberInput::new("input-bridge-to-qty", to_amt)
                                        .label("Equals To Quantity")
                                        .step(dec!(10))
                                        .on_increment({
                                            let v = v_to_amt.clone();
                                            move |val, _window, cx| {
                                                v.update(cx, |this, cx| {
                                                    this.bridge_to_amount = *val;
                                                    cx.notify();
                                                });
                                            }
                                        })
                                        .on_decrement({
                                            let v = v_to_amt.clone();
                                            move |val, _window, cx| {
                                                v.update(cx, |this, cx| {
                                                    this.bridge_to_amount = *val;
                                                    cx.notify();
                                                });
                                            }
                                        }),
                                )
                                .child(
                                    Select::new("select-bridge-to-unit", unit_options)
                                        .label("To Unit")
                                        .selected_id(Some(format!("{:?}", to_unit)))
                                        .on_select(move |opt: &SelectOption, _window, cx| {
                                            let unit = match opt.id.as_str() {
                                                "Gram" => Unit::Gram,
                                                "Milliliter" => Unit::Milliliter,
                                                "Ounce" => Unit::Ounce,
                                                _ => Unit::Gram,
                                            };
                                            v_to_unit.update(cx, |this, cx| {
                                                this.bridge_to_unit = unit;
                                                cx.notify();
                                            });
                                        }),
                                ),
                        )
                        .child(
                            DialogFooter::new()
                                .child(
                                    Button::new("btn-cancel-bridge")
                                        .secondary()
                                        .label("Cancel")
                                        .on_click(|_, window, cx| {
                                            window.close_dialog(cx);
                                        }),
                                )
                                .child(
                                    Button::new("btn-save-bridge")
                                        .primary()
                                        .label("Save Unit Bridge")
                                        .on_click(move |_, window, cx| {
                                            v_save.update(cx, |this, cx| {
                                                this.save_bridge(cx);
                                            });
                                            window.close_dialog(cx);
                                        }),
                                ),
                        )
                })
        });
    }

    pub fn save_bridge(&mut self, cx: &mut Context<Self>) {
        let item_id = match self.bridge_form_item_id {
            Some(id) => id,
            None => return,
        };

        let from_q = match Quantity::new(self.bridge_from_amount, self.bridge_from_unit.clone()) {
            Ok(q) => q,
            Err(e) => {
                self.status_msg = format!("Error: {}", e);
                return;
            }
        };

        let to_q = match Quantity::new(self.bridge_to_amount, self.bridge_to_unit.clone()) {
            Ok(q) => q,
            Err(e) => {
                self.status_msg = format!("Error: {}", e);
                return;
            }
        };

        match UnitBridge::new(item_id, from_q, to_q) {
            Ok(bridge) => {
                if let Ok(mut items) = self.services.items.list_items() {
                    if let Some(item) = items.iter_mut().find(|i| i.id == item_id) {
                        item.count_bridge = Some(bridge);
                        let _ = self.services.items.update_item(item);
                        self.status_msg = format!("Configured unit bridge for {}", item.name);
                    }
                }
            }
            Err(e) => {
                self.status_msg = format!("Error creating bridge: {}", e);
            }
        }
        self.reload_data();
        cx.notify();
    }
}

impl Render for ItemsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let items = self.cached_items.clone();
        let stores = self.cached_stores.clone();

        let filtered_items: Vec<Item> = items
            .into_iter()
            .filter(|i| {
                if self.search_query.trim().is_empty() {
                    true
                } else {
                    let q = self.search_query.to_lowercase();
                    i.name.to_lowercase().contains(&q)
                        || i.category
                            .as_ref()
                            .map(|c| c.to_lowercase().contains(&q))
                            .unwrap_or(false)
                }
            })
            .collect();

        let selected_item = self
            .selected_item_id
            .and_then(|id| filtered_items.iter().find(|i| i.id == id).cloned());
        let has_selected_item = selected_item.is_some();

        let selected_item_packages = self.cached_packages.clone();

        div()
            .flex()
            .flex_col()
            .gap_4()
            .size_full()
            .p_6()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            // Header Bar
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(cx.theme().foreground)
                                    .child("Items, Stores & Package Matrix"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Manage domain ingredients, densities, package pricing, and unit bridges"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(Badge::new().child(format!("Items: {}", filtered_items.len())))
                            .child(Badge::new().child(format!("Stores: {}", stores.len())))
                            .child(
                                Button::new("btn-register-store")
                                    .secondary()
                                    .label("+ Store")
                                    .on_click(cx.listener(|this, _event, window, cx| {
                                        this.open_add_store_modal(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("btn-create-item")
                                    .primary()
                                    .label("+ New Item")
                                    .on_click(cx.listener(|this, _event, window, cx| {
                                        this.open_create_item_modal(window, cx);
                                    })),
                            ),
                    ),
            )
            // Search & Status Bar
            .child(
                div()
                    .flex()
                    .gap_3()
                    .p_3()
                    .bg(cx.theme().muted)
                    .rounded_lg()
                    .child(Alert::new("items-status-alert", format!("Status: {}", self.status_msg))),
            )
            // Split Matrix View (Items Table + Selected Item Details Drawer)
            .child(
                div()
                    .flex()
                    .gap_4()
                    .flex_1()
                    // Main Items Table
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .p_4()
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_lg()
                            .child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .pb_2()
                                    .border_b_1()
                                    .border_color(cx.theme().border)
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(cx.theme().muted_foreground)
                                    .child(div().w_1_4().child("Item Name"))
                                    .child(div().w_1_4().child("Density (g/ml)"))
                                    .child(div().w_1_4().child("Purchase Mode"))
                                    .child(div().w_1_4().child("Actions")),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .overflow_y_scrollbar()
                                    .children(filtered_items.into_iter().map(|item| {
                                        let item_id = item.id;
                                        let is_sel = self.selected_item_id == Some(item_id);
                                        let density_str = item
                                            .density
                                            .map(|d| format!("{} g/ml", d.g_per_ml.normalize()))
                                            .unwrap_or_else(|| "Missing Density".to_string());
                                        let mode_str = format!("{:?}", item.preferred_purchase_mode);
                                        let item_clone = item.clone();

                                        let item_row_id = format!("item-row-{}", item_id);
                                        div()
                                            .id(ElementId::from(item_row_id))
                                            .flex()
                                            .justify_between()
                                            .items_center()
                                            .py_2()
                                            .px_2()
                                            .rounded_md()
                                            .cursor_pointer()
                                            .bg(if is_sel {
                                                cx.theme().accent
                                            } else {
                                                cx.theme().background
                                            })
                                            .hover(|s| s.bg(cx.theme().muted))
                                            .on_click(cx.listener(move |this, _event, _window, cx| {
                                                this.selected_item_id = Some(item_id);
                                                this.reload_data();
                                                cx.notify();
                                            }))
                                            // Name & Category
                                            .child(
                                                div()
                                                    .w_1_4()
                                                    .flex()
                                                    .flex_col()
                                                    .child(
                                                        div()
                                                            .font_weight(FontWeight::BOLD)
                                                            .text_sm()
                                                            .text_color(cx.theme().foreground)
                                                            .child(item.name.clone()),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(cx.theme().muted_foreground)
                                                            .child(item.category.unwrap_or_else(|| "Uncategorized".to_string())),
                                                    ),
                                            )
                                            // Density
                                            .child(
                                                div()
                                                    .w_1_4()
                                                    .child(Tag::new().child(density_str)),
                                            )
                                            // Purchase Mode
                                            .child(
                                                div().w_1_4().child(Tag::new().child(mode_str)),
                                            )
                                            // Action Buttons
                                            .child(
                                                div()
                                                    .w_1_4()
                                                    .flex()
                                                    .items_center()
                                                    .gap_1()
                                                    .child(
                                                        Button::new(format!("btn-edit-{}", item_id))
                                                            .secondary()
                                                            .label("Edit")
                                                            .on_click(cx.listener(move |this, _event, window, cx| {
                                                                this.open_edit_item_modal(&item_clone, window, cx);
                                                            })),
                                                    )
                                                    .child(
                                                        Button::new(format!("btn-delete-{}", item_id))
                                                            .ghost()
                                                            .label("🗑")
                                                            .on_click(cx.listener(move |this, _event, _window, cx| {
                                                                this.delete_item(item_id, cx);
                                                            })),
                                                    ),
                                            )
                                    })),
                            ),
                    )
                    // Selected Item Drawer (Packages & Unit Bridge)
                    .child(
                        div()
                            .w_80()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .p_4()
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_lg()
                            .when_some(selected_item, |this, item| {
                                let item_id = item.id;
                                let count_bridge_str = item
                                    .count_bridge
                                    .as_ref()
                                    .map(|b| format!("{} {} = {} {}", b.from_qty.amount, b.from_qty.unit, b.to_qty.amount, b.to_qty.unit))
                                    .unwrap_or_else(|| "No Bridge Configured".to_string());

                                this.child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .font_weight(FontWeight::BOLD)
                                                .text_base()
                                                .child(item.name.clone()),
                                        )
                                        .child(
                                            Button::new("btn-add-pkg-item")
                                                .primary()
                                                .label("+ Package")
                                                .on_click(cx.listener(move |this, _event, window, cx| {
                                                    this.open_add_package_modal(item_id, window, cx);
                                                })),
                                        ),
                                )
                                // Unit Bridge Card
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_2()
                                        .p_3()
                                        .bg(cx.theme().muted)
                                        .rounded_md()
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .justify_between()
                                                .child(div().text_xs().font_weight(FontWeight::BOLD).child("Unit Bridge"))
                                                .child(
                                                    Button::new("btn-set-bridge")
                                                        .secondary()
                                                        .label("Configure")
                                                        .on_click(cx.listener(move |this, _event, window, cx| {
                                                            this.open_add_bridge_modal(item_id, window, cx);
                                                        })),
                                                ),
                                        )
                                        .child(div().text_xs().text_color(cx.theme().muted_foreground).child(count_bridge_str)),
                                )
                                // Store Packages List
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_2()
                                        .child(div().text_xs().font_weight(FontWeight::BOLD).child("Registered Store Packages"))
                                        .children(selected_item_packages.into_iter().map(|pkg| {
                                            let pkg_id = pkg.id;
                                            let store_name = stores
                                                .iter()
                                                .find(|s| s.id == pkg.store_id)
                                                .map(|s| s.name.as_str())
                                                .unwrap_or("Unknown Store");

                                            let is_pref = pkg.is_preferred;

                                            let pkg_card_id = format!("pkg-card-{}", pkg_id);
                                            div()
                                                .id(ElementId::from(pkg_card_id))
                                                .flex()
                                                .items_center()
                                                .justify_between()
                                                .p_2_5()
                                                .border_1()
                                                .border_color(cx.theme().border)
                                                .rounded_md()
                                                .child(
                                                    div()
                                                        .flex()
                                                        .flex_col()
                                                        .child(
                                                            div()
                                                                .text_sm()
                                                                .font_weight(FontWeight::BOLD)
                                                                .child(format!("{} - ${}", store_name, pkg.price.normalize())),
                                                        )
                                                        .child(
                                                            div()
                                                                .text_xs()
                                                                .text_color(cx.theme().muted_foreground)
                                                                .child(format!("Size: {} {}", pkg.quantity.amount, pkg.quantity.unit)),
                                                        ),
                                                )
                                                .child(
                                                    div()
                                                        .flex()
                                                        .items_center()
                                                        .gap_1()
                                                        .child(
                                                            Button::new(format!("btn-pref-{}", pkg_id))
                                                                .secondary()
                                                                .label(if is_pref { "★ Preferred" } else { "☆ Preferred" })
                                                                .on_click(cx.listener(move |this, _event, _window, cx| {
                                                                    this.toggle_package_preferred(pkg_id, cx);
                                                                })),
                                                        )
                                                        .child(
                                                            Button::new(format!("btn-del-pkg-{}", pkg_id))
                                                                .ghost()
                                                                .label("🗑")
                                                                .on_click(cx.listener(move |this, _event, _window, cx| {
                                                                    this.delete_package(pkg_id, cx);
                                                                })),
                                                        ),
                                                )
                                        })),
                                )
                            })
                            .when(!has_selected_item, |this| {
                                this.child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .items_center()
                                        .justify_center()
                                        .h_full()
                                        .text_center()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("Select an item row from the matrix to view store packages and unit bridge details."),
                                )
                            }),
                    ),
            )
    }
}
