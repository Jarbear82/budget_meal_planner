use crate::components::*;
use bmp_domain::*;
use bmp_services::AppServices;
use chrono::{Local, NaiveDate};
use gpui::prelude::*;
use gpui::*;
use gpui_component::badge::Badge;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};
use gpui_component::tag::Tag;
use gpui_component::ActiveTheme;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::str::FromStr;

pub struct PantryView {
    pub services: AppServices,
    pub status_msg: String,

    // Add Stock Modal
    pub show_add_modal: bool,
    pub add_item_id: Option<ItemId>,
    pub add_amount: Decimal,
    pub add_unit: Unit,
    pub add_expiration: Option<NaiveDate>,
    pub add_has_expiration: bool,
}

impl PantryView {
    pub fn new(services: AppServices) -> Self {
        Self {
            services,
            status_msg: "Pantry inventory ready".to_string(),

            show_add_modal: false,
            add_item_id: None,
            add_amount: dec!(500),
            add_unit: Unit::Gram,
            add_expiration: Some(Local::now().date_naive() + chrono::Duration::days(14)),
            add_has_expiration: true,
        }
    }

    pub fn open_add_modal(&mut self, cx: &mut Context<Self>) {
        let items = self.services.items.list_items().unwrap_or_default();
        self.add_item_id = items.first().map(|i| i.id);
        self.add_amount = dec!(500);
        self.add_unit = Unit::Gram;
        self.add_expiration = Some(Local::now().date_naive() + chrono::Duration::days(14));
        self.add_has_expiration = true;
        self.show_add_modal = true;
        cx.notify();
    }

    pub fn save_pantry_entry(&mut self, cx: &mut Context<Self>) {
        let item_id = match self.add_item_id {
            Some(id) => id,
            None => {
                self.status_msg = "Error: Select an item to add to pantry".to_string();
                return;
            }
        };

        let exp = if self.add_has_expiration {
            self.add_expiration
        } else {
            None
        };

        match self.services.pantry.add_pantry_entry(
            item_id,
            self.add_amount,
            self.add_unit.clone(),
            exp,
        ) {
            Ok(_) => {
                self.status_msg = "Added pantry stock successfully".to_string();
                self.show_add_modal = false;
            }
            Err(e) => {
                self.status_msg = format!("Error adding stock: {}", e);
            }
        }
        cx.notify();
    }

    pub fn update_quantity(&mut self, entry_id: PantryEntryId, delta: Decimal, cx: &mut Context<Self>) {
        if let Ok(entries) = self.services.pantry.get_pantry() {
            if let Some(entry) = entries.iter().find(|e| e.id == entry_id) {
                let next_amount = entry.quantity.amount + delta;
                if next_amount <= Decimal::ZERO {
                    let _ = self.services.pantry.delete_pantry_entry(entry_id);
                    self.status_msg = "Depleted stock entry removed".to_string();
                } else {
                    let _ = self.services.pantry.update_quantity(entry_id, next_amount);
                    self.status_msg = format!("Updated stock quantity to {}", next_amount.normalize());
                }
            }
        }
        cx.notify();
    }

    pub fn delete_entry(&mut self, entry_id: PantryEntryId, cx: &mut Context<Self>) {
        match self.services.pantry.delete_pantry_entry(entry_id) {
            Ok(_) => {
                self.status_msg = "Deleted pantry stock entry".to_string();
            }
            Err(e) => {
                self.status_msg = format!("Error deleting entry: {}", e);
            }
        }
        cx.notify();
    }
}

impl Render for PantryView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entries = self.services.pantry.get_pantry().unwrap_or_default();
        let items = self.services.items.list_items().unwrap_or_default();
        let today = Local::now().date_naive();

        let item_options: Vec<SelectOption> = items
            .iter()
            .map(|i| SelectOption::new(i.id.0.to_string(), i.name.clone()))
            .collect();

        let unit_options = vec![
            SelectOption::new("Gram", "Gram (g)"),
            SelectOption::new("Kilogram", "Kilogram (kg)"),
            SelectOption::new("Milliliter", "Milliliter (ml)"),
            SelectOption::new("Liter", "Liter (L)"),
            SelectOption::new("Cup", "Cup"),
            SelectOption::new("Ounce", "Ounce (oz)"),
            SelectOption::new("Pound", "Pound (lb)"),
            SelectOption::new("Each", "Each (count)"),
        ];

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
                                    .child("Pantry Inventory & Stock Tracking"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Track available quantities, unit measurements, and expiration dates"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(Badge::new().child(format!("Pantry Entries: {}", entries.len())))
                            .child(
                                Button::new("btn-add-pantry-stock")
                                    .primary()
                                    .label("+ Add Stock")
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.open_add_modal(cx);
                                    })),
                            ),
                    ),
            )
            // Status Bar
            .child(
                div()
                    .p_3()
                    .bg(cx.theme().muted)
                    .rounded_lg()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("Status: {}", self.status_msg)),
            )
            // Inventory Table
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_5()
                    .bg(cx.theme().background)
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded_lg()
                    .flex_1()
                    .overflow_y_scrollbar()
                    .child(
                        Table::new()
                            .child(
                                TableHeader::new()
                                    .child(TableHead::new().child("Item Name"))
                                    .child(TableHead::new().child("Available Stock Quantity"))
                                    .child(TableHead::new().child("Expiration Status"))
                                    .child(TableHead::new().child("Quick Adjust & Delete")),
                            )
                            .child(
                                TableBody::new().children(entries.into_iter().map(|entry| {
                                    let entry_id = entry.id;
                                    let item_name = items
                                        .iter()
                                        .find(|i| i.id == entry.item_id)
                                        .map(|i| i.name.clone())
                                        .unwrap_or_else(|| "Unknown Item".to_string());

                                    let (exp_str, exp_status) = match entry.expiration {
                                        Some(exp) => {
                                            let days = (exp - today).num_days();
                                            if days < 0 {
                                                (format!("Expired ({})", exp), "Expired")
                                            } else if days <= 3 {
                                                (format!("Expires in {}d ({})", days, exp), "Expires Soon")
                                            } else {
                                                (format!("Expires {}", exp), "Fresh")
                                            }
                                        }
                                        None => ("No Expiration Date".to_string(), "Fresh"),
                                    };

                                    TableRow::new()
                                        .child(
                                            TableCell::new().child(
                                                div()
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_sm()
                                                    .text_color(cx.theme().foreground)
                                                    .child(item_name),
                                            ),
                                        )
                                        .child(
                                            TableCell::new().child(
                                                Tag::new().child(format!("{} {}", entry.quantity.amount.normalize(), entry.quantity.unit)),
                                            ),
                                        )
                                        .child(
                                            TableCell::new().child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_2()
                                                    .child(Badge::new().child(exp_status))
                                                    .child(div().text_xs().text_color(cx.theme().muted_foreground).child(exp_str)),
                                            ),
                                        )
                                        .child(
                                            TableCell::new().child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_1()
                                                    .child(
                                                        Button::new(format!("btn-dec-pantry-{}", entry_id))
                                                            .secondary()
                                                            .label("- 100")
                                                            .on_click(cx.listener(move |this, _event, _window, cx| {
                                                                this.update_quantity(entry_id, dec!(-100), cx);
                                                            })),
                                                    )
                                                    .child(
                                                        Button::new(format!("btn-inc-pantry-{}", entry_id))
                                                            .secondary()
                                                            .label("+ 100")
                                                            .on_click(cx.listener(move |this, _event, _window, cx| {
                                                                this.update_quantity(entry_id, dec!(100), cx);
                                                            })),
                                                    )
                                                    .child(
                                                        Button::new(format!("btn-del-pantry-{}", entry_id))
                                                            .ghost()
                                                            .label("🗑")
                                                            .on_click(cx.listener(move |this, _event, _window, cx| {
                                                                this.delete_entry(entry_id, cx);
                                                            })),
                                                    ),
                                            ),
                                        )
                                })),
                            ),
                    ),
            )
            // Add Stock Modal Dialog
            .child(
                Dialog::new("add-pantry-stock-modal", "Add Pantry Inventory Stock")
                    .subtitle("Deposit stock into your pantry inventory")
                    .is_open(self.show_add_modal)
                    .on_close(cx.listener(|this, _event, _window, cx| {
                        this.show_add_modal = false;
                        cx.notify();
                    }))
                    .child(
                        Select::new("select-pantry-item", item_options)
                            .label("Item Name")
                            .selected_id(self.add_item_id.map(|id| id.0.to_string()))
                            .on_select(cx.listener(|this, opt: &SelectOption, _window, cx| {
                                if let Ok(uuid) = uuid::Uuid::from_str(&opt.id) {
                                    this.add_item_id = Some(ItemId(uuid));
                                }
                                cx.notify();
                            })),
                    )
                    .child(
                        NumberInput::new("input-pantry-amount", self.add_amount)
                            .label("Stock Quantity Amount")
                            .step(dec!(50))
                            .on_increment(cx.listener(|this, val, _window, cx| {
                                this.add_amount = *val;
                                cx.notify();
                            }))
                            .on_decrement(cx.listener(|this, val, _window, cx| {
                                this.add_amount = *val;
                                cx.notify();
                            })),
                    )
                    .child(
                        Select::new("select-pantry-unit", unit_options)
                            .label("Unit")
                            .selected_id(Some(format!("{:?}", self.add_unit)))
                            .on_select(cx.listener(|this, opt: &SelectOption, _window, cx| {
                                this.add_unit = match opt.id.as_str() {
                                    "Kilogram" => Unit::Kilogram,
                                    "Milliliter" => Unit::Milliliter,
                                    "Liter" => Unit::Liter,
                                    "Cup" => Unit::Cup,
                                    "Ounce" => Unit::Ounce,
                                    "Pound" => Unit::Pound,
                                    "Each" => Unit::Each,
                                    _ => Unit::Gram,
                                };
                                cx.notify();
                            })),
                    )
                    .child(
                        Checkbox::new("cb-has-expiration")
                            .label("Set Expiration Date")
                            .checked(self.add_has_expiration)
                            .on_click(cx.listener(|this, checked, _window, cx| {
                                this.add_has_expiration = *checked;
                                cx.notify();
                            })),
                    )
                    .when(self.add_has_expiration, |this| {
                        let exp_date = self.add_expiration.unwrap_or(today);
                        this.child(
                            DatePicker::new("dp-pantry-exp", exp_date)
                                .label("Expiration Date")
                                .on_change(cx.listener(|this, date, _window, cx| {
                                    this.add_expiration = Some(*date);
                                    cx.notify();
                                })),
                        )
                    })
                    .footer_action(
                        Button::new("btn-cancel-pantry-modal")
                            .secondary()
                            .label("Cancel")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.show_add_modal = false;
                                cx.notify();
                            })),
                    )
                    .footer_action(
                        Button::new("btn-save-pantry-modal")
                            .primary()
                            .label("Deposit to Pantry")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.save_pantry_entry(cx);
                            })),
                    ),
            )
    }
}
