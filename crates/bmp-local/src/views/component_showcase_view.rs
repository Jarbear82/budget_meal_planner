use crate::components::*;
use chrono::{Local, NaiveDate};
use gpui::prelude::*;
use gpui::*;
use gpui_component::badge::Badge;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::ActiveTheme;
use rust_decimal_macros::dec;

pub struct ComponentShowcaseView {
    pub text_value: String,
    pub number_value: rust_decimal::Decimal,
    pub selected_option_id: Option<String>,
    pub select_is_open: bool,
    pub checkbox_checked: bool,
    pub selected_date: NaiveDate,
    pub datepicker_is_open: bool,
    pub dialog_is_open: bool,
    pub status_msg: String,
}

impl ComponentShowcaseView {
    pub fn new() -> Self {
        Self {
            text_value: "Sample Ingredient Name".to_string(),
            number_value: dec!(1.25),
            selected_option_id: Some("grams".to_string()),
            select_is_open: false,
            checkbox_checked: true,
            selected_date: Local::now().date_naive(),
            datepicker_is_open: false,
            dialog_is_open: false,
            status_msg: "Phase 1 UI Primitives Ready".to_string(),
        }
    }
}

impl Render for ComponentShowcaseView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let options = vec![
            SelectOption::new("grams", "Grams (g)").with_description("Mass unit"),
            SelectOption::new("ml", "Milliliters (ml)").with_description("Volume unit"),
            SelectOption::new("each", "Each (count)").with_description("Discrete count unit"),
            SelectOption::new("lbs", "Pounds (lbs)").with_description("Imperial mass unit"),
        ];

        let text_val = self.text_value.clone();
        let number_val = self.number_value;
        let selected_opt_id = self.selected_option_id.clone();
        let select_open = self.select_is_open;
        let _cb_checked = self.checkbox_checked;
        let sel_date = self.selected_date;
        let date_open = self.datepicker_is_open;
        let dlg_open = self.dialog_is_open;

        div()
            .flex()
            .flex_col()
            .gap_4()
            .size_full()
            .p_6()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(FontWeight::BOLD)
                            .child("Phase 1 UI Component Primitives Showcase"),
                    )
                    .child(Badge::new().child("Foundation Controls")),
            )
            .child(
                div()
                    .p_3()
                    .bg(cx.theme().muted)
                    .rounded_lg()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("State Status: {}", self.status_msg)),
            )
            // Showcase Grid / Cards
            .child(
                div()
                    .grid()
                    .grid_cols(2)
                    .gap_6()
                    // Card 1: Form Inputs & Numeric Controls
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .p_5()
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_xl()
                            .shadow_sm()
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::BOLD)
                                    .child("1. FormInput & NumberInput"),
                            )
                            .child(
                                FormInput::new("showcase-input-1")
                                    .label("Ingredient Name")
                                    .placeholder("e.g. Olive Oil")
                                    .value(text_val)
                                    .helper_text("Enter the primary domain item identifier"),
                            )
                            .child(
                                NumberInput::new("showcase-number-1", number_val)
                                    .label("Density / Quantity")
                                    .step(dec!(0.1))
                                    .unit("g/ml")
                                    .on_increment(cx.listener(|this, val, _window, cx| {
                                        this.number_value = *val;
                                        this.status_msg = format!("Updated quantity: {}", val);
                                        cx.notify();
                                    }))
                                    .on_decrement(cx.listener(|this, val, _window, cx| {
                                        this.number_value = *val;
                                        this.status_msg = format!("Updated quantity: {}", val);
                                        cx.notify();
                                    })),
                            ),
                    )
                    // Card 2: Select & Combobox Controls
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .p_5()
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_xl()
                            .shadow_sm()
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::BOLD)
                                    .child("2. Select / Combobox Picker"),
                            )
                            .child(
                                Select::new("showcase-select-1", options)
                                    .label("Measurement Unit")
                                    .selected_id(selected_opt_id)
                                    .is_open(select_open)
                                    .on_toggle(cx.listener(|this, open, _window, cx| {
                                        this.select_is_open = *open;
                                        cx.notify();
                                    }))
                                    .on_select(cx.listener(|this, opt: &SelectOption, _window, cx| {
                                        this.selected_option_id = Some(opt.id.clone());
                                        this.select_is_open = false;
                                        this.status_msg = format!("Selected unit option: {}", opt.label);
                                        cx.notify();
                                    })),
                            ),
                    )
                    // Card 3: Checkbox & Toggles
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .p_5()
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_xl()
                            .shadow_sm()
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::BOLD)
                                    .child("3. Checkbox & Toggles"),
                            )
                            .child(
                                Checkbox::new("showcase-cb-1")
                                    .label("Optional Recipe Ingredient")
                                    .checked(self.checkbox_checked)
                                    .on_click(cx.listener(|this, checked, _window, cx| {
                                        this.checkbox_checked = *checked;
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Checkbox::new("showcase-cb-2")
                                    .label("Preferred Package Pinning - Force shopping list store selection")
                                    .checked(true),
                            ),
                    )
                    // Card 4: DatePicker & Modal Dialog Trigger
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .p_5()
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_xl()
                            .shadow_sm()
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::BOLD)
                                    .child("4. DatePicker & Dialog Modal"),
                            )
                            .child(
                                DatePicker::new("showcase-dp-1", sel_date)
                                    .label("Scheduled Meal Date")
                                    .is_open(date_open)
                                    .on_toggle(cx.listener(|this, open, _window, cx| {
                                        this.datepicker_is_open = *open;
                                        cx.notify();
                                    }))
                                    .on_change(cx.listener(|this, date, _window, cx| {
                                        this.selected_date = *date;
                                        this.datepicker_is_open = false;
                                        this.status_msg = format!("Selected date: {}", date);
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Button::new("btn-open-dialog-demo")
                                    .primary()
                                    .label("Open Interactive Modal Dialog")
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.dialog_is_open = true;
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            // Interactive Modal Dialog Instance
            .child(
                Dialog::new("demo-modal-dialog", "Make Recipe Configuration")
                    .subtitle("Configure batches, yields, and optional substitutes before batching")
                    .is_open(dlg_open)
                    .on_close(cx.listener(|this, _event, _window, cx| {
                        this.dialog_is_open = false;
                        cx.notify();
                    }))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .child(
                                FormInput::new("modal-input-batches")
                                    .label("Number of Batches")
                                    .value("2.0"),
                            )
                            .child(
                                Checkbox::new("modal-cb-include-optionals")
                                    .label("Include optional ingredients")
                                    .checked(true),
                            ),
                    )
                    .footer_action(
                        Button::new("btn-modal-cancel")
                            .secondary()
                            .label("Cancel")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.dialog_is_open = false;
                                cx.notify();
                            })),
                    )
                    .footer_action(
                        Button::new("btn-modal-confirm")
                            .primary()
                            .label("Produce Batches into Pantry")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.dialog_is_open = false;
                                this.status_msg = "Successfully executed Make Recipe production!".to_string();
                                cx.notify();
                            })),
                    ),
            )
    }
}
