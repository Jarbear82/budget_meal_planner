use crate::components::*;
use bmp_domain::*;
use bmp_services::AppServices;
use chrono::Utc;
use gpui::prelude::*;
use gpui::*;
use gpui_component::badge::Badge;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::tag::Tag;
use gpui_component::ActiveTheme;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::str::FromStr;

pub struct ShoppingView {
    pub services: AppServices,
    pub status_msg: String,

    // Shopping List State
    pub active_list: Option<ShoppingList>,
    pub selected_store_id: Option<StoreId>,
    pub tax_rate: Decimal,

    // Modals visibility
    pub show_receipt_modal: bool,
    pub show_add_item_modal: bool,

    // Receipt form state
    pub receipt_actual_total: Decimal,
    pub deposit_to_pantry: bool,
    pub receipt_status: String,

    // Custom Item form state
    pub custom_item_id: Option<ItemId>,
    pub custom_amount: Decimal,
    pub custom_unit: Unit,
}

impl ShoppingView {
    pub fn new(services: AppServices) -> Self {
        Self {
            services,
            status_msg: "Shopping list manager ready".to_string(),

            active_list: None,
            selected_store_id: None,
            tax_rate: dec!(0.0),

            show_receipt_modal: false,
            show_add_item_modal: false,

            receipt_actual_total: dec!(0.00),
            deposit_to_pantry: true,
            receipt_status: String::new(),

            custom_item_id: None,
            custom_amount: dec!(1),
            custom_unit: Unit::Each,
        }
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

    pub fn open_receipt_modal(&mut self, cx: &mut Context<Self>) {
        if let Some(ref list) = self.active_list {
            self.receipt_actual_total = list.total;
            self.receipt_status = String::new();
            self.show_receipt_modal = true;
        } else {
            self.status_msg = "Please generate a shopping list first".to_string();
        }
        cx.notify();
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

        let diff = self.receipt_actual_total - list.total;
        let diff_str = if diff > Decimal::ZERO {
            format!("+${} over estimate", diff.normalize())
        } else if diff < Decimal::ZERO {
            format!("-${} under estimate", (-diff).normalize())
        } else {
            "Exact match with estimate".to_string()
        };

        self.status_msg = format!(
            "Recorded receipt #{} (${}). {}! Deposited {} items to Pantry.",
            receipt_id,
            self.receipt_actual_total.normalize(),
            diff_str,
            deposited_count
        );

        self.show_receipt_modal = false;
        self.active_list = None;
        cx.notify();
    }

    pub fn open_add_item_modal(&mut self, cx: &mut Context<Self>) {
        let items = self.services.items.list_items().unwrap_or_default();
        self.custom_item_id = items.first().map(|i| i.id);
        self.custom_amount = dec!(1);
        self.custom_unit = Unit::Each;
        self.show_add_item_modal = true;
        cx.notify();
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
                self.show_add_item_modal = false;
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
        let stores = self.services.items.list_stores().unwrap_or_default();
        let items = self.services.items.list_items().unwrap_or_default();

        let store_options: Vec<SelectOption> = std::iter::once(SelectOption::new("all", "All Stores (Best Price)"))
            .chain(stores.iter().map(|s| SelectOption::new(s.id.0.to_string(), s.name.clone())))
            .collect();

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
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.open_add_item_modal(cx);
                                    })),
                            )
                            .child(
                                Button::new("btn-reconcile-receipt-header")
                                    .primary()
                                    .label("🧾 Finish & Reconcile")
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.open_receipt_modal(cx);
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
                    .child(
                        div()
                            .flex_1()
                            .p_2()
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_md()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("Status: {}", self.status_msg)),
                    ),
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
                                    .child(div().w_1_12().child("Buy"))
                                    .child(div().w_1_4().child("Item Name"))
                                    .child(div().w_1_4().child("Required vs Package Ceiling"))
                                    .child(div().w_1_6().child("Store"))
                                    .child(div().w_1_6().child("Line Total ($)")),
                            )
                            .when(has_list, |this| {
                                let list = self.active_list.as_ref().unwrap();
                                let line_items = list.items.clone();

                                this.child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .overflow_y_scrollbar()
                                        .children(line_items.into_iter().enumerate().map(|(idx, line)| {
                                            let is_checked = line.is_checked;
                                            let store_name = stores
                                                .iter()
                                                .find(|s| s.id == line.store_id)
                                                .map(|s| s.name.clone())
                                                .unwrap_or_else(|| "Store".to_string());

                                            let mode_str = format!("{:?}", line.purchase_mode);

                                            let line_id = format!("shopping-line-{}", idx);
                                            div()
                                                .id(ElementId::from(line_id))
                                                .flex()
                                                .justify_between()
                                                .items_center()
                                                .py_2()
                                                .px_2()
                                                .border_b_1()
                                                .border_color(cx.theme().border)
                                                .rounded_md()
                                                .bg(if is_checked {
                                                    cx.theme().muted
                                                } else {
                                                    cx.theme().background
                                                })
                                                // Checkbox
                                                .child(
                                                    div()
                                                        .w_1_12()
                                                        .child(
                                                            Checkbox::new(format!("cb-line-{}", idx))
                                                                .checked(is_checked)
                                                                .on_click(cx.listener(move |this, _checked, _window, cx| {
                                                                    this.toggle_item_checked(idx, cx);
                                                                })),
                                                        ),
                                                )
                                                // Item Name & Mode
                                                .child(
                                                    div()
                                                        .w_1_4()
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
                                                            div()
                                                                .text_xs()
                                                                .text_color(cx.theme().muted_foreground)
                                                                .child(mode_str),
                                                        ),
                                                )
                                                // Quantities & Package Ceiling Calculation
                                                .child(
                                                    div()
                                                        .w_1_4()
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
                                                )
                                                // Store Name
                                                .child(
                                                    div()
                                                        .w_1_6()
                                                        .child(Tag::new().child(store_name)),
                                                )
                                                // Price Line Total
                                                .child(
                                                    div()
                                                        .w_1_6()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_sm()
                                                        .text_color(cx.theme().foreground)
                                                        .child(format!("${}", line.line_total.normalize())),
                                                )
                                        })),
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
                                        .child("Click '🔄 Generate List' above to calculate consolidated shopping requirements based on your scheduled meals and pantry stock."),
                                )
                            }),
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
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.open_receipt_modal(cx);
                                    })),
                            ),
                    ),
            )
            // Receipt Reconciliation Modal Dialog
            .child(
                Dialog::new("receipt-reconciliation-modal", "Checkout & Reconcile Receipt")
                    .subtitle("Enter your actual receipt total to record spend analytics and update pantry inventory")
                    .is_open(self.show_receipt_modal)
                    .on_close(cx.listener(|this, _event, _window, cx| {
                        this.show_receipt_modal = false;
                        cx.notify();
                    }))
                    .child(
                        NumberInput::new("input-receipt-actual-total", self.receipt_actual_total)
                            .label("Actual Receipt Total ($)")
                            .step(dec!(1.00))
                            .unit("$")
                            .on_increment(cx.listener(|this, val, _window, cx| {
                                this.receipt_actual_total = *val;
                                cx.notify();
                            }))
                            .on_decrement(cx.listener(|this, val, _window, cx| {
                                this.receipt_actual_total = *val;
                                cx.notify();
                            })),
                    )
                    .child(
                        Checkbox::new("cb-deposit-pantry")
                            .label("Deposit checked-off packages directly into Pantry inventory")
                            .checked(self.deposit_to_pantry)
                            .on_click(cx.listener(|this, checked, _window, cx| {
                                this.deposit_to_pantry = *checked;
                                cx.notify();
                            })),
                    )
                    .when(!self.receipt_status.is_empty(), |this| {
                        let msg = self.receipt_status.clone();
                        this.child(
                            div()
                                .p_3()
                                .bg(cx.theme().accent)
                                .rounded_md()
                                .text_xs()
                                .child(msg),
                        )
                    })
                    .footer_action(
                        Button::new("btn-cancel-receipt-modal")
                            .secondary()
                            .label("Cancel")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.show_receipt_modal = false;
                                cx.notify();
                            })),
                    )
                    .footer_action(
                        Button::new("btn-confirm-receipt-modal")
                            .primary()
                            .label("Record Receipt & Clear List")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.finish_and_reconcile_receipt(cx);
                            })),
                    ),
            )
            // Add Custom Requirement Modal Dialog
            .child(
                Dialog::new("add-custom-item-modal", "Add Extra Shopping Requirement")
                    .subtitle("Manually add a package requirement to your shopping list")
                    .is_open(self.show_add_item_modal)
                    .on_close(cx.listener(|this, _event, _window, cx| {
                        this.show_add_item_modal = false;
                        cx.notify();
                    }))
                    .child(
                        Select::new("select-custom-item", item_options)
                            .label("Item")
                            .selected_id(self.custom_item_id.map(|id| id.0.to_string()))
                            .on_select(cx.listener(|this, opt: &SelectOption, _window, cx| {
                                if let Ok(uuid) = uuid::Uuid::from_str(&opt.id) {
                                    this.custom_item_id = Some(ItemId(uuid));
                                }
                                cx.notify();
                            })),
                    )
                    .child(
                        NumberInput::new("input-custom-amount", self.custom_amount)
                            .label("Required Amount")
                            .step(dec!(1))
                            .on_increment(cx.listener(|this, val, _window, cx| {
                                this.custom_amount = *val;
                                cx.notify();
                            }))
                            .on_decrement(cx.listener(|this, val, _window, cx| {
                                this.custom_amount = *val;
                                cx.notify();
                            })),
                    )
                    .child(
                        Select::new("select-custom-unit", unit_options)
                            .label("Unit")
                            .selected_id(Some(format!("{:?}", self.custom_unit)))
                            .on_select(cx.listener(|this, opt: &SelectOption, _window, cx| {
                                this.custom_unit = match opt.id.as_str() {
                                    "Kilogram" => Unit::Kilogram,
                                    "Milliliter" => Unit::Milliliter,
                                    "Liter" => Unit::Liter,
                                    "Cup" => Unit::Cup,
                                    "Pound" => Unit::Pound,
                                    "Each" => Unit::Each,
                                    _ => Unit::Gram,
                                };
                                cx.notify();
                            })),
                    )
                    .footer_action(
                        Button::new("btn-cancel-custom")
                            .secondary()
                            .label("Cancel")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.show_add_item_modal = false;
                                cx.notify();
                            })),
                    )
                    .footer_action(
                        Button::new("btn-save-custom")
                            .primary()
                            .label("Add to Shopping List")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.add_custom_requirement(cx);
                            })),
                    ),
            )
    }
}
