use crate::components::*;
use bmp_domain::*;
use bmp_services::AppServices;
use gpui::prelude::*;
use gpui::*;
use gpui_component::alert::Alert;
use gpui_component::badge::Badge;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::tag::Tag;
use gpui_component::ActiveTheme;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::str::FromStr;

pub struct ItemsView {
    pub services: AppServices,
    pub search_query: String,
    pub status_msg: String,

    // Selected Item for detail view / editing
    pub selected_item_id: Option<ItemId>,

    // Modals visibility state
    pub show_item_modal: bool,
    pub show_package_modal: bool,
    pub show_store_modal: bool,
    pub show_bridge_modal: bool,

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
        Self {
            services,
            search_query: String::new(),
            status_msg: "Items matrix ready".to_string(),
            selected_item_id: None,

            show_item_modal: false,
            show_package_modal: false,
            show_store_modal: false,
            show_bridge_modal: false,

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
        }
    }

    pub fn open_create_item_modal(&mut self, cx: &mut Context<Self>) {
        self.editing_item_id = None;
        self.item_form_name = String::new();
        self.item_form_density = dec!(1.0);
        self.item_form_category = "General".to_string();
        self.item_form_mode = PurchaseMode::BuyFinished;
        self.show_item_modal = true;
        cx.notify();
    }

    pub fn open_edit_item_modal(&mut self, item: &Item, cx: &mut Context<Self>) {
        self.editing_item_id = Some(item.id);
        self.item_form_name = item.name.clone();
        self.item_form_density = item.density.map(|d| d.g_per_ml).unwrap_or(dec!(1.0));
        self.item_form_category = item.category.clone().unwrap_or_else(|| "General".to_string());
        self.item_form_mode = item.preferred_purchase_mode;
        self.show_item_modal = true;
        cx.notify();
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
                            self.show_item_modal = false;
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
                    self.show_item_modal = false;
                }
                Err(e) => {
                    self.status_msg = format!("Error creating item: {}", e);
                }
            }
        }
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
        cx.notify();
    }

    pub fn open_add_package_modal(&mut self, item_id: ItemId, cx: &mut Context<Self>) {
        self.pkg_form_item_id = Some(item_id);
        let stores = self.services.items.list_stores().unwrap_or_default();
        self.pkg_form_store_id = stores.first().map(|s| s.id);
        self.pkg_form_amount = dec!(500);
        self.pkg_form_unit = Unit::Gram;
        self.pkg_form_price = dec!(4.99);
        self.pkg_form_preferred = true;
        self.show_package_modal = true;
        cx.notify();
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
                self.show_package_modal = false;
            }
            Err(e) => {
                self.status_msg = format!("Error adding package: {}", e);
            }
        }
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
        cx.notify();
    }

    pub fn open_add_store_modal(&mut self, cx: &mut Context<Self>) {
        self.store_form_name = String::new();
        self.show_store_modal = true;
        cx.notify();
    }

    pub fn save_store(&mut self, cx: &mut Context<Self>) {
        if self.store_form_name.trim().is_empty() {
            self.status_msg = "Error: Store name cannot be empty".to_string();
            return;
        }

        match self.services.items.add_store(self.store_form_name.trim()) {
            Ok(store) => {
                self.status_msg = format!("Registered store: {}", store.name);
                self.show_store_modal = false;
            }
            Err(e) => {
                self.status_msg = format!("Error adding store: {}", e);
            }
        }
        cx.notify();
    }

    pub fn open_add_bridge_modal(&mut self, item_id: ItemId, cx: &mut Context<Self>) {
        self.bridge_form_item_id = Some(item_id);
        self.bridge_from_amount = dec!(1);
        self.bridge_from_unit = Unit::Each;
        self.bridge_to_amount = dec!(150);
        self.bridge_to_unit = Unit::Gram;
        self.show_bridge_modal = true;
        cx.notify();
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
                        self.show_bridge_modal = false;
                    }
                }
            }
            Err(e) => {
                self.status_msg = format!("Error creating bridge: {}", e);
            }
        }
        cx.notify();
    }
}

impl Render for ItemsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let items = self.services.items.list_items().unwrap_or_default();
        let stores = self.services.items.list_stores().unwrap_or_default();

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

        let mode_options = vec![
            SelectOption::new("BuyFinished", "Buy Finished Package"),
            SelectOption::new("PreferMake", "Prefer Make / Expand"),
            SelectOption::new("AskEveryTime", "Ask Every Time"),
        ];

        let selected_item = self
            .selected_item_id
            .and_then(|id| filtered_items.iter().find(|i| i.id == id).cloned());
        let has_selected_item = selected_item.is_some();

        let selected_item_packages = if let Some(ref item) = selected_item {
            self.services
                .items
                .get_packages_for_item(item.id)
                .unwrap_or_default()
        } else {
            Vec::new()
        };

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
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.open_add_store_modal(cx);
                                    })),
                            )
                            .child(
                                Button::new("btn-create-item")
                                    .primary()
                                    .label("+ New Item")
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.open_create_item_modal(cx);
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
                                                            .on_click(cx.listener(move |this, _event, _window, cx| {
                                                                this.open_edit_item_modal(&item_clone, cx);
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
                                                .on_click(cx.listener(move |this, _event, _window, cx| {
                                                    this.open_add_package_modal(item_id, cx);
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
                                                        .on_click(cx.listener(move |this, _event, _window, cx| {
                                                            this.open_add_bridge_modal(item_id, cx);
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
            // Item Creation / Edit Modal Dialog
            .child(
                Dialog::new(
                    "item-crud-modal",
                    if self.editing_item_id.is_some() {
                        "Edit Domain Item"
                    } else {
                        "Add Domain Item"
                    },
                )
                .subtitle("Configure density (g/ml), category, and default purchase mode")
                .is_open(self.show_item_modal)
                .on_close(cx.listener(|this, _event, _window, cx| {
                    this.show_item_modal = false;
                    cx.notify();
                }))
                .child(
                    FormInput::new("input-item-name")
                        .label("Item Name")
                        .placeholder("e.g. Extra Virgin Olive Oil")
                        .value(self.item_form_name.clone()),
                )
                .child(
                    FormInput::new("input-item-category")
                        .label("Category")
                        .placeholder("e.g. Oils & Fats, Pantry, Dairy")
                        .value(self.item_form_category.clone()),
                )
                .child(
                    NumberInput::new("input-item-density", self.item_form_density)
                        .label("Density (g/ml)")
                        .step(dec!(0.05))
                        .unit("g/ml")
                        .on_increment(cx.listener(|this, val, _window, cx| {
                            this.item_form_density = *val;
                            cx.notify();
                        }))
                        .on_decrement(cx.listener(|this, val, _window, cx| {
                            this.item_form_density = *val;
                            cx.notify();
                        })),
                )
                .child(
                    Select::new("select-item-purchase-mode", mode_options)
                        .label("Preferred Purchase Mode")
                        .selected_id(Some(format!("{:?}", self.item_form_mode)))
                        .on_select(cx.listener(|this, opt: &SelectOption, _window, cx| {
                            this.item_form_mode = match opt.id.as_str() {
                                "PreferMake" => PurchaseMode::PreferMake,
                                "AskEveryTime" => PurchaseMode::AskEveryTime,
                                _ => PurchaseMode::BuyFinished,
                            };
                            cx.notify();
                        })),
                )
                .footer_action(
                    Button::new("btn-cancel-item-modal")
                        .secondary()
                        .label("Cancel")
                        .on_click(cx.listener(|this, _event, _window, cx| {
                            this.show_item_modal = false;
                            cx.notify();
                        })),
                )
                .footer_action(
                    Button::new("btn-save-item-modal")
                        .primary()
                        .label("Save Item")
                        .on_click(cx.listener(|this, _event, _window, cx| {
                            this.save_item(cx);
                        })),
                ),
            )
            // Package Creation Modal Dialog
            .child(
                Dialog::new("package-crud-modal", "Add Store Package")
                    .subtitle("Register a store price and quantity package for this item")
                    .is_open(self.show_package_modal)
                    .on_close(cx.listener(|this, _event, _window, cx| {
                        this.show_package_modal = false;
                        cx.notify();
                    }))
                    .child(
                        Select::new("select-pkg-store", store_options)
                            .label("Store")
                            .selected_id(self.pkg_form_store_id.map(|id| id.0.to_string()))
                            .on_select(cx.listener(|this, opt: &SelectOption, _window, cx| {
                                if let Ok(uuid) = uuid::Uuid::from_str(&opt.id) {
                                    this.pkg_form_store_id = Some(StoreId(uuid));
                                }
                                cx.notify();
                            })),
                    )
                    .child(
                        NumberInput::new("input-pkg-amount", self.pkg_form_amount)
                            .label("Package Size / Amount")
                            .step(dec!(50))
                            .on_increment(cx.listener(|this, val, _window, cx| {
                                this.pkg_form_amount = *val;
                                cx.notify();
                            }))
                            .on_decrement(cx.listener(|this, val, _window, cx| {
                                this.pkg_form_amount = *val;
                                cx.notify();
                            })),
                    )
                    .child(
                        Select::new("select-pkg-unit", unit_options.clone())
                            .label("Package Unit")
                            .selected_id(Some(format!("{:?}", self.pkg_form_unit)))
                            .on_select(cx.listener(|this, opt: &SelectOption, _window, cx| {
                                this.pkg_form_unit = match opt.id.as_str() {
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
                                cx.notify();
                            })),
                    )
                    .child(
                        NumberInput::new("input-pkg-price", self.pkg_form_price)
                            .label("Package Price ($)")
                            .step(dec!(0.50))
                            .unit("$")
                            .on_increment(cx.listener(|this, val, _window, cx| {
                                this.pkg_form_price = *val;
                                cx.notify();
                            }))
                            .on_decrement(cx.listener(|this, val, _window, cx| {
                                this.pkg_form_price = *val;
                                cx.notify();
                            })),
                    )
                    .child(
                        Checkbox::new("cb-pkg-preferred")
                            .label("Preferred store package for shopping calculations")
                            .checked(self.pkg_form_preferred)
                            .on_click(cx.listener(|this, checked, _window, cx| {
                                this.pkg_form_preferred = *checked;
                                cx.notify();
                            })),
                    )
                    .footer_action(
                        Button::new("btn-cancel-pkg")
                            .secondary()
                            .label("Cancel")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.show_package_modal = false;
                                cx.notify();
                            })),
                    )
                    .footer_action(
                        Button::new("btn-save-pkg")
                            .primary()
                            .label("Add Package")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.save_package(cx);
                            })),
                    ),
            )
            // Store Creation Modal Dialog
            .child(
                Dialog::new("store-crud-modal", "Register New Store")
                    .subtitle("Add a supermarket, grocery store, or supplier")
                    .is_open(self.show_store_modal)
                    .on_close(cx.listener(|this, _event, _window, cx| {
                        this.show_store_modal = false;
                        cx.notify();
                    }))
                    .child(
                        FormInput::new("input-store-name")
                            .label("Store Name")
                            .placeholder("e.g. Costco, Trader Joe's, Safeway")
                            .value(self.store_form_name.clone()),
                    )
                    .footer_action(
                        Button::new("btn-cancel-store")
                            .secondary()
                            .label("Cancel")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.show_store_modal = false;
                                cx.notify();
                            })),
                    )
                    .footer_action(
                        Button::new("btn-save-store")
                            .primary()
                            .label("Register Store")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.save_store(cx);
                            })),
                    ),
            )
            // Unit Bridge Modal Dialog
            .child(
                Dialog::new("bridge-crud-modal", "Configure Unit Bridge")
                    .subtitle("Define custom count-to-mass or custom unit conversion for this item")
                    .is_open(self.show_bridge_modal)
                    .on_close(cx.listener(|this, _event, _window, cx| {
                        this.show_bridge_modal = false;
                        cx.notify();
                    }))
                    .child(
                        NumberInput::new("input-bridge-from-qty", self.bridge_from_amount)
                            .label("From Quantity")
                            .step(dec!(1))
                            .on_increment(cx.listener(|this, val, _window, cx| {
                                this.bridge_from_amount = *val;
                                cx.notify();
                            }))
                            .on_decrement(cx.listener(|this, val, _window, cx| {
                                this.bridge_from_amount = *val;
                                cx.notify();
                            })),
                    )
                    .child(
                        Select::new("select-bridge-from-unit", unit_options.clone())
                            .label("From Unit")
                            .selected_id(Some(format!("{:?}", self.bridge_from_unit)))
                            .on_select(cx.listener(|this, opt: &SelectOption, _window, cx| {
                                this.bridge_from_unit = match opt.id.as_str() {
                                    "Gram" => Unit::Gram,
                                    "Each" => Unit::Each,
                                    "Cup" => Unit::Cup,
                                    _ => Unit::Each,
                                };
                                cx.notify();
                            })),
                    )
                    .child(
                        NumberInput::new("input-bridge-to-qty", self.bridge_to_amount)
                            .label("Equals To Quantity")
                            .step(dec!(10))
                            .on_increment(cx.listener(|this, val, _window, cx| {
                                this.bridge_to_amount = *val;
                                cx.notify();
                            }))
                            .on_decrement(cx.listener(|this, val, _window, cx| {
                                this.bridge_to_amount = *val;
                                cx.notify();
                            })),
                    )
                    .child(
                        Select::new("select-bridge-to-unit", unit_options.clone())
                            .label("To Unit")
                            .selected_id(Some(format!("{:?}", self.bridge_to_unit)))
                            .on_select(cx.listener(|this, opt: &SelectOption, _window, cx| {
                                this.bridge_to_unit = match opt.id.as_str() {
                                    "Gram" => Unit::Gram,
                                    "Milliliter" => Unit::Milliliter,
                                    "Ounce" => Unit::Ounce,
                                    _ => Unit::Gram,
                                };
                                cx.notify();
                            })),
                    )
                    .footer_action(
                        Button::new("btn-cancel-bridge")
                            .secondary()
                            .label("Cancel")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.show_bridge_modal = false;
                                cx.notify();
                            })),
                    )
                    .footer_action(
                        Button::new("btn-save-bridge")
                            .primary()
                            .label("Save Unit Bridge")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.save_bridge(cx);
                            })),
                    ),
            )
    }
}
