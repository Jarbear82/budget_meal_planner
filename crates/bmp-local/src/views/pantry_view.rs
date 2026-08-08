use crate::components::*;
use bmp_domain::*;
use bmp_services::AppServices;
use chrono::{Local, NaiveDate};
use gpui::prelude::*;
use gpui::*;
use gpui_component::badge::Badge;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::dialog::{DialogDescription, DialogFooter, DialogHeader, DialogTitle};
use gpui_component::scroll::ScrollableElement;
use gpui_component::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};
use gpui_component::tag::Tag;
use gpui_component::WindowExt;
use gpui_component::ActiveTheme;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::str::FromStr;

pub struct PantryView {
    pub services: AppServices,
    pub status_msg: String,

    pub cached_entries: Vec<PantryEntry>,
    pub cached_items: Vec<Item>,

    // Add Stock Form State
    pub add_item_id: Option<ItemId>,
    pub add_amount: Decimal,
    pub add_unit: Unit,
    pub add_expiration: Option<NaiveDate>,
    pub add_has_expiration: bool,
}

impl PantryView {
    pub fn new(services: AppServices) -> Self {
        let mut view = Self {
            services,
            status_msg: "Pantry inventory ready".to_string(),

            cached_entries: Vec::new(),
            cached_items: Vec::new(),

            add_item_id: None,
            add_amount: dec!(500),
            add_unit: Unit::Gram,
            add_expiration: Some(Local::now().date_naive() + chrono::Duration::days(14)),
            add_has_expiration: true,
        };
        view.reload_data();
        view
    }

    pub fn reload_data(&mut self) {
        self.cached_entries = self.services.pantry.get_pantry().unwrap_or_default();
        self.cached_items = self.services.items.list_items().unwrap_or_default();
    }

    pub fn open_add_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let items = &self.cached_items;
        self.add_item_id = items.first().map(|i| i.id);
        self.add_amount = dec!(500);
        self.add_unit = Unit::Gram;
        self.add_expiration = Some(Local::now().date_naive() + chrono::Duration::days(14));
        self.add_has_expiration = true;

        let view = cx.entity().clone();
        window.open_dialog(cx, move |dialog, _, _| {
            let view = view.clone();
            dialog
                .w(px(500.))
                .content(move |content, _, cx| {
                    let view_read = view.read(cx);
                    let items = &view_read.cached_items;
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

                    let add_item_id = view_read.add_item_id;
                    let add_amount = view_read.add_amount;
                    let add_unit = view_read.add_unit.clone();
                    let add_has_exp = view_read.add_has_expiration;
                    let add_exp = view_read.add_expiration.unwrap_or(today);

                    let v_select_item = view.clone();
                    let v_num_amt = view.clone();
                    let v_select_unit = view.clone();
                    let v_cb_exp = view.clone();
                    let v_dp_exp = view.clone();
                    let v_save = view.clone();

                    content
                        .child(
                            DialogHeader::new()
                                .child(DialogTitle::new().child("Add Pantry Inventory Stock"))
                                .child(DialogDescription::new().child("Deposit stock into your pantry inventory")),
                        )
                        .child(
                            div()
                                .py_4()
                                .flex()
                                .flex_col()
                                .gap_3()
                                .child(
                                    Select::new("select-pantry-item", item_options)
                                        .label("Item Name")
                                        .selected_id(add_item_id.map(|id| id.0.to_string()))
                                        .on_select(move |opt: &SelectOption, _window, cx| {
                                            if let Ok(uuid) = uuid::Uuid::from_str(&opt.id) {
                                                v_select_item.update(cx, |this, cx| {
                                                    this.add_item_id = Some(ItemId(uuid));
                                                    cx.notify();
                                                });
                                            }
                                        }),
                                )
                                .child(
                                    NumberInput::new("input-pantry-amount", add_amount)
                                        .label("Stock Quantity Amount")
                                        .step(dec!(50))
                                        .on_increment({
                                            let v = v_num_amt.clone();
                                            move |val, _window, cx| {
                                                v.update(cx, |this, cx| {
                                                    this.add_amount = *val;
                                                    cx.notify();
                                                });
                                            }
                                        })
                                        .on_decrement({
                                            let v = v_num_amt.clone();
                                            move |val, _window, cx| {
                                                v.update(cx, |this, cx| {
                                                    this.add_amount = *val;
                                                    cx.notify();
                                                });
                                            }
                                        }),
                                )
                                .child(
                                    Select::new("select-pantry-unit", unit_options)
                                        .label("Unit")
                                        .selected_id(Some(format!("{:?}", add_unit)))
                                        .on_select(move |opt: &SelectOption, _window, cx| {
                                            let unit = match opt.id.as_str() {
                                                "Kilogram" => Unit::Kilogram,
                                                "Milliliter" => Unit::Milliliter,
                                                "Liter" => Unit::Liter,
                                                "Cup" => Unit::Cup,
                                                "Ounce" => Unit::Ounce,
                                                "Pound" => Unit::Pound,
                                                "Each" => Unit::Each,
                                                _ => Unit::Gram,
                                            };
                                            v_select_unit.update(cx, |this, cx| {
                                                this.add_unit = unit;
                                                cx.notify();
                                            });
                                        }),
                                )
                                .child(
                                    Checkbox::new("cb-has-expiration")
                                        .label("Set Expiration Date")
                                        .checked(add_has_exp)
                                        .on_click(move |checked, _window, cx| {
                                            v_cb_exp.update(cx, |this, cx| {
                                                this.add_has_expiration = *checked;
                                                cx.notify();
                                            });
                                        }),
                                )
                                .when(add_has_exp, |this| {
                                    this.child(
                                        DatePicker::new("dp-pantry-exp", add_exp)
                                            .label("Expiration Date")
                                            .on_change(move |date, _window, cx| {
                                                v_dp_exp.update(cx, |this, cx| {
                                                    this.add_expiration = Some(*date);
                                                    cx.notify();
                                                });
                                            }),
                                    )
                                }),
                        )
                        .child(
                            DialogFooter::new()
                                .child(
                                    Button::new("btn-cancel-pantry-modal")
                                        .secondary()
                                        .label("Cancel")
                                        .on_click(|_, window, cx| {
                                            window.close_dialog(cx);
                                        }),
                                )
                                .child(
                                    Button::new("btn-save-pantry-modal")
                                        .primary()
                                        .label("Deposit to Pantry")
                                        .on_click(move |_, window, cx| {
                                            v_save.update(cx, |this, cx| {
                                                this.save_pantry_entry(cx);
                                            });
                                            window.close_dialog(cx);
                                        }),
                                ),
                        )
                })
        });
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
            }
            Err(e) => {
                self.status_msg = format!("Error adding stock: {}", e);
            }
        }
        self.reload_data();
        cx.notify();
    }

    pub fn update_quantity(&mut self, entry_id: PantryEntryId, delta: Decimal, cx: &mut Context<Self>) {
        let entries = &self.cached_entries;
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
        self.reload_data();
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
        self.reload_data();
        cx.notify();
    }
}

impl Render for PantryView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let entries = self.cached_entries.clone();
        let items = self.cached_items.clone();
        let today = Local::now().date_naive();

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
                                    .on_click(cx.listener(|this, _event, window, cx| {
                                        this.open_add_modal(window, cx);
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
    }
}
