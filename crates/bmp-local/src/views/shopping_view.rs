use crate::components::*;
use bmp_domain::*;
use bmp_services::AppServices;
use chrono::Utc;
use gpui::prelude::*;
use gpui::*;
use gpui_component::alert::Alert;
use gpui_component::badge::Badge;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::dialog::{DialogDescription, DialogFooter, DialogHeader, DialogTitle};
use gpui_component::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};
use gpui_component::tag::Tag;
use gpui_component::WindowExt;
use gpui_component::ActiveTheme;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::str::FromStr;

pub struct ShoppingView {
    pub services: AppServices,
    pub status_msg: String,

    pub cached_list: Option<ShoppingList>,
    pub cached_items: Vec<Item>,
    pub cached_stores: Vec<Store>,

    // Shopping List State
    pub active_list: Option<ShoppingList>,
    pub selected_store_id: Option<StoreId>,
    pub tax_rate: Decimal,

    // Receipt form state
    pub receipt_actual_total: Decimal,
    pub deposit_to_pantry: bool,
    pub update_package_prices: bool,
    pub receipt_status: String,

    // Custom Item form state
    pub custom_item_id: Option<ItemId>,
    pub custom_amount: Decimal,
    pub custom_unit: Unit,
}

impl ShoppingView {
    pub fn new(services: AppServices) -> Self {
        let mut view = Self {
            services,
            status_msg: "Shopping list manager ready".to_string(),

            cached_list: None,
            cached_items: Vec::new(),
            cached_stores: Vec::new(),

            active_list: None,
            selected_store_id: None,
            tax_rate: dec!(0.0),

            receipt_actual_total: dec!(0.00),
            deposit_to_pantry: true,
            update_package_prices: true,
            receipt_status: String::new(),

            custom_item_id: None,
            custom_amount: dec!(1),
            custom_unit: Unit::Each,
        };
        view.reload_data();
        view
    }

    pub fn reload_data(&mut self) {
        self.cached_items = self.services.items.list_items().unwrap_or_default();
        self.cached_stores = self.services.items.list_stores().unwrap_or_default();
    }

    pub fn generate_shopping_list(&mut self, cx: &mut Context<Self>) {
        let tax_opt = if self.tax_rate > Decimal::ZERO {
            Some(self.tax_rate)
        } else {
            None
        };

        match self.services.shopping.generate_shopping_list(
            Vec::new(),
            self.selected_store_id,
            tax_opt,
        ) {
            Ok(list) => {
                self.receipt_actual_total = list.total;
                self.status_msg = format!(
                    "Generated shopping list with {} items. Estimated total: ${}",
                    list.items.len(),
                    list.total.normalize()
                );
                self.active_list = Some(list);
            }
            Err(e) => {
                self.status_msg = format!("Error generating list: {}", e);
            }
        }
        cx.notify();
    }

    pub fn toggle_item_checked(&mut self, idx: usize, cx: &mut Context<Self>) {
        if let Some(ref mut list) = self.active_list {
            if idx < list.items.len() {
                list.items[idx].is_checked = !list.items[idx].is_checked;
                let checked_count = list.items.iter().filter(|i| i.is_checked).count();
                self.status_msg = format!("Checked {}/{} items", checked_count, list.items.len());
            }
        }
        cx.notify();
    }

    pub fn toggle_line_purchase_mode(&mut self, idx: usize, cx: &mut Context<Self>) {
        if let Some(ref list) = self.active_list {
            if idx < list.items.len() {
                let item_id = list.items[idx].item_id;
                let current_mode = list.items[idx].purchase_mode;
                let new_mode = match current_mode {
                    PurchaseMode::BuyFinished => PurchaseMode::PreferMake,
                    _ => PurchaseMode::BuyFinished,
                };
                if let Ok(Some(mut item)) = self.services.items.get_item(item_id) {
                    item.preferred_purchase_mode = new_mode;
                    let _ = self.services.items.update_item(&item);
                    self.status_msg = format!("Updated purchase mode for '{}' to {:?}", item.name, new_mode);
                }
            }
        }
        self.generate_shopping_list(cx);
    }

    pub fn open_receipt_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(ref list) = self.active_list {
            self.receipt_actual_total = list.total;
            self.receipt_status = String::new();
        } else {
            self.status_msg = "Please generate a shopping list first".to_string();
            cx.notify();
            return;
        }

        let view = cx.entity().clone();
        window.open_dialog(cx, move |dialog, _, _| {
            let view = view.clone();
            dialog
                .w(px(500.))
                .content(move |content, _, cx| {
                    let view_read = view.read(cx);
                    let receipt_actual_total = view_read.receipt_actual_total;
                    let deposit_to_pantry = view_read.deposit_to_pantry;
                    let update_package_prices = view_read.update_package_prices;
                    let receipt_status = view_read.receipt_status.clone();

                    let v_num = view.clone();
                    let v_cb = view.clone();
                    let v_pkg = view.clone();
                    let v_confirm = view.clone();

                    content
                        .child(
                            DialogHeader::new()
                                .child(DialogTitle::new().child("Checkout & Reconcile Receipt"))
                                .child(DialogDescription::new().child("Enter your actual receipt total to record spend analytics, update pantry inventory, and optionally update package prices")),
                        )
                        .child(
                            div()
                                .py_4()
                                .flex()
                                .flex_col()
                                .gap_3()
                                .child(
                                    NumberInput::new("input-receipt-actual-total", receipt_actual_total)
                                        .label("Actual Receipt Total ($)")
                                        .step(dec!(1.00))
                                        .unit("$")
                                        .on_increment({
                                            let v = v_num.clone();
                                            move |val, _window, cx| {
                                                v.update(cx, |this, cx| {
                                                    this.receipt_actual_total = *val;
                                                    cx.notify();
                                                });
                                            }
                                        })
                                        .on_decrement({
                                            let v = v_num.clone();
                                            move |val, _window, cx| {
                                                v.update(cx, |this, cx| {
                                                    this.receipt_actual_total = *val;
                                                    cx.notify();
                                                });
                                            }
                                        }),
                                )
                                .child(
                                    Checkbox::new("cb-deposit-pantry")
                                        .label("Deposit checked-off packages directly into Pantry inventory")
                                        .checked(deposit_to_pantry)
                                        .on_click(move |checked, _window, cx| {
                                            v_cb.update(cx, |this, cx| {
                                                this.deposit_to_pantry = *checked;
                                                cx.notify();
                                            });
                                        }),
                                )
                                .child(
                                    Checkbox::new("cb-update-package-prices")
                                        .label("Proportionally update Package prices if actual total differs from estimate (SRS §2.3.3)")
                                        .checked(update_package_prices)
                                        .on_click(move |checked, _window, cx| {
                                            v_pkg.update(cx, |this, cx| {
                                                this.update_package_prices = *checked;
                                                cx.notify();
                                            });
                                        }),
                                )
                                .when(!receipt_status.is_empty(), |this| {
                                    this.child(
                                        div()
                                            .p_3()
                                            .bg(cx.theme().accent)
                                            .rounded_md()
                                            .text_xs()
                                            .child(receipt_status),
                                    )
                                }),
                        )
                        .child(
                            DialogFooter::new()
                                .child(
                                    Button::new("btn-cancel-receipt-modal")
                                        .secondary()
                                        .label("Cancel")
                                        .on_click(|_, window, cx| {
                                            window.close_dialog(cx);
                                        }),
                                )
                                .child(
                                    Button::new("btn-confirm-receipt-modal")
                                        .primary()
                                        .label("Record Receipt & Clear List")
                                        .on_click(move |_, window, cx| {
                                            v_confirm.update(cx, |this, cx| {
                                                this.finish_and_reconcile_receipt(cx);
                                            });
                                            window.close_dialog(cx);
                                        }),
                                ),
                        )
                })
        });
    }

    pub fn finish_and_reconcile_receipt(&mut self, cx: &mut Context<Self>) {
        let list = match &self.active_list {
            Some(l) => l,
            None => return,
        };

        // 1. Record receipt in Analytics
        let receipt_res = self.services.analytics.record_receipt(
            self.selected_store_id,
            self.receipt_actual_total,
            Utc::now(),
        );

        let receipt_id = match receipt_res {
            Ok(id) => id,
            Err(e) => {
                self.receipt_status = format!("Error recording receipt: {}", e);
                cx.notify();
                return;
            }
        };

        // 2. Deposit checked items to Pantry if option selected
        let mut deposited_count = 0;
        if self.deposit_to_pantry {
            for line in &list.items {
                if line.is_checked {
                    let total_qty_purchased = Quantity {
                        amount: line.package_qty.amount * Decimal::from(line.package_count),
                        unit: line.package_qty.unit.clone(),
                    };
                    if self
                        .services
                        .pantry
                        .add_pantry_entry(line.item_id, total_qty_purchased.amount, total_qty_purchased.unit, None)
                        .is_ok()
                    {
                        deposited_count += 1;
                    }
                }
            }
        }

        // 3. Update Package prices if option selected & total differs (SRS §2.3.3)
        let mut updated_packages_count = 0;
        if self.update_package_prices && list.total > Decimal::ZERO && self.receipt_actual_total != list.total {
            let scale_factor = self.receipt_actual_total / list.total;
            for line in &list.items {
                if line.is_checked {
                    let new_price = (line.package_price * scale_factor).round_dp(2);
                    if self.services.items.update_package_price(line.package_id, new_price).is_ok() {
                        updated_packages_count += 1;
                    }
                }
            }
        }

        let diff = self.receipt_actual_total - list.total;
        let diff_str = if diff > Decimal::ZERO {
            format!("+${} over estimate", diff.normalize())
        } else if diff < Decimal::ZERO {
            format!("-${} under estimate", (-diff).normalize())
        } else {
            "Exact match with estimate".to_string()
        };

        let pkg_update_str = if updated_packages_count > 0 {
            format!(" Updated {} package prices.", updated_packages_count)
        } else {
            String::new()
        };

        self.status_msg = format!(
            "Recorded receipt #{} (${}). {}! Deposited {} items to Pantry.{}",
            receipt_id,
            self.receipt_actual_total.normalize(),
            diff_str,
            deposited_count,
            pkg_update_str
        );

        self.active_list = None;
        cx.notify();
    }

    pub fn open_add_item_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let items = &self.cached_items;
        self.custom_item_id = items.first().map(|i| i.id);
        self.custom_amount = dec!(1);
        self.custom_unit = Unit::Each;

        let view = cx.entity().clone();
        window.open_dialog(cx, move |dialog, _, _| {
            let view = view.clone();
            dialog
                .w(px(500.))
                .content(move |content, _, cx| {
                    let view_read = view.read(cx);
                    let items = &view_read.cached_items;
                    let item_options: Vec<SelectOption> = items
                        .iter()
                        .map(|i| SelectOption::new(i.id.0.to_string(), i.name.clone()))
                        .collect();

                    let unit_options = vec![
                        SelectOption::new("Each", "Each (count)"),
                        SelectOption::new("Gram", "Gram (g)"),
                        SelectOption::new("Kilogram", "Kilogram (kg)"),
                        SelectOption::new("Milliliter", "Milliliter (ml)"),
                        SelectOption::new("Liter", "Liter (L)"),
                        SelectOption::new("Cup", "Cup"),
                        SelectOption::new("Pound", "Pound (lb)"),
                    ];

                    let custom_item_id = view_read.custom_item_id;
                    let custom_amount = view_read.custom_amount;
                    let custom_unit = view_read.custom_unit.clone();

                    let v_select_item = view.clone();
                    let v_num_amt = view.clone();
                    let v_select_unit = view.clone();
                    let v_save = view.clone();

                    content
                        .child(
                            DialogHeader::new()
                                .child(DialogTitle::new().child("Add Extra Shopping Requirement"))
                                .child(DialogDescription::new().child("Manually add a package requirement to your shopping list")),
                        )
                        .child(
                            div()
                                .py_4()
                                .flex()
                                .flex_col()
                                .gap_3()
                                .child(
                                    Select::new("select-custom-item", item_options)
                                        .label("Item")
                                        .selected_id(custom_item_id.map(|id| id.0.to_string()))
                                        .on_select(move |opt: &SelectOption, _window, cx| {
                                            if let Ok(uuid) = uuid::Uuid::from_str(&opt.id) {
                                                v_select_item.update(cx, |this, cx| {
                                                    this.custom_item_id = Some(ItemId(uuid));
                                                    cx.notify();
                                                });
                                            }
                                        }),
                                )
                                .child(
                                    NumberInput::new("input-custom-amount", custom_amount)
                                        .label("Required Amount")
                                        .step(dec!(1))
                                        .on_increment({
                                            let v = v_num_amt.clone();
                                            move |val, _window, cx| {
                                                v.update(cx, |this, cx| {
                                                    this.custom_amount = *val;
                                                    cx.notify();
                                                });
                                            }
                                        })
                                        .on_decrement({
                                            let v = v_num_amt.clone();
                                            move |val, _window, cx| {
                                                v.update(cx, |this, cx| {
                                                    this.custom_amount = *val;
                                                    cx.notify();
                                                });
                                            }
                                        }),
                                )
                                .child(
                                    Select::new("select-custom-unit", unit_options)
                                        .label("Unit")
                                        .selected_id(Some(format!("{:?}", custom_unit)))
                                        .on_select(move |opt: &SelectOption, _window, cx| {
                                            let unit = match opt.id.as_str() {
                                                "Kilogram" => Unit::Kilogram,
                                                "Milliliter" => Unit::Milliliter,
                                                "Liter" => Unit::Liter,
                                                "Cup" => Unit::Cup,
                                                "Pound" => Unit::Pound,
                                                "Each" => Unit::Each,
                                                _ => Unit::Gram,
                                            };
                                            v_select_unit.update(cx, |this, cx| {
                                                this.custom_unit = unit;
                                                cx.notify();
                                            });
                                        }),
                                ),
                        )
                        .child(
                            DialogFooter::new()
                                .child(
                                    Button::new("btn-cancel-custom")
                                        .secondary()
                                        .label("Cancel")
                                        .on_click(|_, window, cx| {
                                            window.close_dialog(cx);
                                        }),
                                )
                                .child(
                                    Button::new("btn-save-custom")
                                        .primary()
                                        .label("Add to Shopping List")
                                        .on_click(move |_, window, cx| {
                                            v_save.update(cx, |this, cx| {
                                                this.add_custom_requirement(cx);
                                            });
                                            window.close_dialog(cx);
                                        }),
                                ),
                        )
                })
        });
    }

    pub fn add_custom_requirement(&mut self, cx: &mut Context<Self>) {
        let item_id = match self.custom_item_id {
            Some(id) => id,
            None => {
                self.status_msg = "Error: Select an item to add".to_string();
                return;
            }
        };

        let qty = match Quantity::new(self.custom_amount, self.custom_unit.clone()) {
            Ok(q) => q,
            Err(e) => {
                self.status_msg = format!("Error: {}", e);
                return;
            }
        };

        let reqs = vec![(item_id, qty)];
        let tax_opt = if self.tax_rate > Decimal::ZERO {
            Some(self.tax_rate)
        } else {
            None
        };

        match self.services.shopping.generate_shopping_list(
            reqs,
            self.selected_store_id,
            tax_opt,
        ) {
            Ok(new_list) => {
                if let Some(ref mut existing) = self.active_list {
                    existing.items.extend(new_list.items);
                    existing.subtotal += new_list.subtotal;
                    existing.total += new_list.total;
                } else {
                    self.active_list = Some(new_list);
                }
                self.status_msg = "Added requirement to shopping list".to_string();
            }
            Err(e) => {
                self.status_msg = format!("Error: {}", e);
            }
        }
        cx.notify();
    }
}

impl Render for ShoppingView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let stores = self.cached_stores.clone();

        let store_options: Vec<SelectOption> = std::iter::once(SelectOption::new("all", "All Stores (Best Price)"))
            .chain(stores.iter().map(|s| SelectOption::new(s.id.0.to_string(), s.name.clone())))
            .collect();

        let tax_options = vec![
            SelectOption::new("0", "0% Tax"),
            SelectOption::new("0.05", "5.0% Sales Tax"),
            SelectOption::new("0.0825", "8.25% Sales Tax"),
            SelectOption::new("0.10", "10.0% Sales Tax"),
        ];

        let has_list = self.active_list.is_some();
        let total_items_count = self.active_list.as_ref().map(|l| l.items.len()).unwrap_or(0);
        let checked_items_count = self
            .active_list
            .as_ref()
            .map(|l| l.items.iter().filter(|i| i.is_checked).count())
            .unwrap_or(0);

        let subtotal_str = self
            .active_list
            .as_ref()
            .map(|l| format!("${}", l.subtotal.normalize()))
            .unwrap_or_else(|| "$0.00".to_string());

        let total_str = self
            .active_list
            .as_ref()
            .map(|l| format!("${}", l.total.normalize()))
            .unwrap_or_else(|| "$0.00".to_string());

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
                                    .child("Shopping List & Store Reconciliation"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Consolidate meal requirements, subtract pantry inventory, and reconcile receipt totals"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(Badge::new().child(format!("Checked: {}/{}", checked_items_count, total_items_count)))
                            .child(
                                Button::new("btn-generate-shopping-list")
                                    .secondary()
                                    .label("🔄 Generate List")
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.generate_shopping_list(cx);
                                    })),
                            )
                            .child(
                                Button::new("btn-add-custom-item")
                                    .secondary()
                                    .label("+ Add Extra")
                                    .on_click(cx.listener(|this, _event, window, cx| {
                                        this.open_add_item_modal(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("btn-reconcile-receipt-header")
                                    .primary()
                                    .label("🧾 Finish & Reconcile")
                                    .on_click(cx.listener(|this, _event, window, cx| {
                                        this.open_receipt_modal(window, cx);
                                    })),
                            ),
                    ),
            )
            // Filter Toolbar Bar
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_4()
                    .p_3()
                    .bg(cx.theme().muted)
                    .rounded_lg()
                    .child(
                        div()
                            .w_64()
                            .child(
                                Select::new("select-shopping-store-filter", store_options)
                                    .label("Target Store Filter")
                                    .selected_id(self.selected_store_id.map(|id| id.0.to_string()))
                                    .on_select(cx.listener(|this, opt: &SelectOption, _window, cx| {
                                        if opt.id == "all" {
                                            this.selected_store_id = None;
                                        } else if let Ok(uuid) = uuid::Uuid::from_str(&opt.id) {
                                            this.selected_store_id = Some(StoreId(uuid));
                                        }
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .w_48()
                            .child(
                                Select::new("select-tax-rate", tax_options)
                                    .label("Applied Sales Tax")
                                    .selected_id(Some(format!("{}", self.tax_rate)))
                                    .on_select(cx.listener(|this, opt: &SelectOption, _window, cx| {
                                        if let Ok(rate) = Decimal::from_str(&opt.id) {
                                            this.tax_rate = rate;
                                        }
                                        cx.notify();
                                    })),
                            ),
                    )
                    .child(Alert::new("shopping-status-alert", format!("Status: {}", self.status_msg))),
            )
            // Shopping List Main Pane + Summary Side Card
            .child(
                div()
                    .flex()
                    .gap_4()
                    .flex_1()
                    // Table of Line Items
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
                            .when(has_list, |this| {
                                let list = self.active_list.as_ref().unwrap();
                                let line_items = list.items.clone();

                                this.child(
                                    Table::new()
                                        .child(
                                            TableHeader::new()
                                                .child(TableHead::new().child("Buy"))
                                                .child(TableHead::new().child("Item Name"))
                                                .child(TableHead::new().child("Required vs Package Ceiling"))
                                                .child(TableHead::new().child("Store"))
                                                .child(TableHead::new().child("Line Total ($)")),
                                        )
                                        .child(
                                            TableBody::new().children(line_items.into_iter().enumerate().map(|(idx, line)| {
                                                let is_checked = line.is_checked;
                                                let store_name = stores
                                                    .iter()
                                                    .find(|s| s.id == line.store_id)
                                                    .map(|s| s.name.clone())
                                                    .unwrap_or_else(|| "Store".to_string());

                                                TableRow::new()
                                                    .child(
                                                        TableCell::new().child(
                                                            Checkbox::new(format!("cb-line-{}", idx))
                                                                .checked(is_checked)
                                                                .on_click(cx.listener(move |this, _checked, _window, cx| {
                                                                    this.toggle_item_checked(idx, cx);
                                                                })),
                                                        ),
                                                    )
                                                    .child(
                                                        TableCell::new().child(
                                                            div()
                                                                .flex()
                                                                .flex_col()
                                                                .child(
                                                                    div()
                                                                        .font_weight(FontWeight::BOLD)
                                                                        .text_sm()
                                                                        .text_color(if is_checked {
                                                                            cx.theme().muted_foreground
                                                                        } else {
                                                                            cx.theme().foreground
                                                                        })
                                                                        .child(line.item_name.clone()),
                                                                )
                                                                .child(
                                                                    Button::new(format!("btn-toggle-mode-{}", idx))
                                                                        .ghost()
                                                                        .label(format!("Mode: {:?}", line.purchase_mode))
                                                                        .on_click(cx.listener(move |this, _event, _window, cx| {
                                                                            this.toggle_line_purchase_mode(idx, cx);
                                                                        })),
                                                                ),
                                                        ),
                                                    )
                                                    .child(
                                                        TableCell::new().child(
                                                            div()
                                                                .flex()
                                                                .flex_col()
                                                                .child(
                                                                    div()
                                                                        .text_xs()
                                                                        .font_weight(FontWeight::SEMIBOLD)
                                                                        .child(format!("Req: {} {}", line.required_qty.amount, line.required_qty.unit)),
                                                                )
                                                                .child(
                                                                    div()
                                                                        .text_xs()
                                                                        .text_color(cx.theme().muted_foreground)
                                                                        .child(format!("Buy {}x ({} {})", line.package_count, line.package_qty.amount, line.package_qty.unit)),
                                                                ),
                                                        ),
                                                    )
                                                    .child(TableCell::new().child(Tag::new().child(store_name)))
                                                    .child(
                                                        TableCell::new().child(
                                                            div()
                                                                .font_weight(FontWeight::BOLD)
                                                                .text_sm()
                                                                .text_color(cx.theme().foreground)
                                                                .child(format!("${}", line.line_total.normalize())),
                                                        ),
                                                    )
                                            })),
                                        ),
                                )
                            })
                            .when(!has_list, |this| {
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
                                        .child("Click 'Generate List' above to calculate consolidated shopping requirements based on your scheduled meals and pantry stock."),
                                )
                            })
                    )
                    // Summary Side Panel Card
                    .child(
                        div()
                            .w_72()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .p_5()
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_lg()
                            .child(div().text_base().font_weight(FontWeight::BOLD).child("Order Summary"))
                            .child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .text_sm()
                                    .child(div().text_color(cx.theme().muted_foreground).child("Subtotal"))
                                    .child(div().font_weight(FontWeight::BOLD).child(subtotal_str)),
                            )
                            .child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .text_sm()
                                    .child(div().text_color(cx.theme().muted_foreground).child("Tax Rate"))
                                    .child(div().child(format!("{}%", self.tax_rate * dec!(100)))),
                            )
                            .child(
                                div()
                                    .flex()
                                    .justify_between()
                                    .pt_2()
                                    .border_t_1()
                                    .border_color(cx.theme().border)
                                    .text_base()
                                    .font_weight(FontWeight::BOLD)
                                    .child("Estimated Total")
                                    .child(div().text_color(cx.theme().primary).child(total_str)),
                            )
                            .child(
                                Button::new("btn-reconcile-checkout-side")
                                    .primary()
                                    .label("Checkout & Reconcile")
                                    .on_click(cx.listener(|this, _event, window, cx| {
                                        this.open_receipt_modal(window, cx);
                                    })),
                            )
                    )
            )
    }
}
