use crate::components::*;
use chrono::Local;
use gpui::prelude::*;
use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::WindowExt;
use gpui_component::badge::Badge;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::dialog::{DialogDescription, DialogFooter, DialogHeader, DialogTitle};
use rust_decimal_macros::dec;

pub struct ComponentShowcaseView {
    pub demo_input: Entity<InputState>,
    pub demo_select: Entity<SelectState<Vec<SelectOption>>>,
    pub demo_date_picker: Entity<DatePickerState>,
    pub number_value: rust_decimal::Decimal,
    pub checkbox_checked: bool,
    pub status_msg: String,
}

impl ComponentShowcaseView {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let demo_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("e.g. Olive Oil")
                .default_value("Sample Ingredient Name")
        });

        let options = vec![
            SelectOption::new("grams", "Grams (g)").with_description("Mass unit"),
            SelectOption::new("ml", "Milliliters (ml)").with_description("Volume unit"),
            SelectOption::new("each", "Each (count)").with_description("Discrete count unit"),
            SelectOption::new("lbs", "Pounds (lbs)").with_description("Imperial mass unit"),
        ];

        let demo_select = cx.new(|cx| {
            SelectState::new(options, Some(IndexPath::default().row(0)), window, cx)
        });

        let demo_date_picker = cx.new(|cx| {
            let mut picker = DatePickerState::new(window, cx);
            picker.set_date(Local::now().date_naive(), window, cx);
            picker
        });

        cx.subscribe_in(
            &demo_select,
            window,
            |this, _, ev: &SelectEvent<_>, _window, cx| {
                if let SelectEvent::Confirm(Some(id)) = ev {
                    this.status_msg = format!("Selected unit option: {}", id);
                    cx.notify();
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &demo_date_picker,
            window,
            |this, _, ev: &DatePickerEvent, _window, cx| {
                if let DatePickerEvent::Change(Date::Single(Some(date))) = ev {
                    this.status_msg = format!("Selected date: {}", date);
                    cx.notify();
                }
            },
        )
        .detach();

        Self {
            demo_input,
            demo_select,
            demo_date_picker,
            number_value: dec!(1.25),
            checkbox_checked: true,
            status_msg: "UI Primitives Ready".to_string(),
        }
    }
}

impl Render for ComponentShowcaseView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let number_val = self.number_value;

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
                                    .child("1. Interactive Text Input & Number Controls"),
                            )
                            .child(
                                form_field(
                                    "Ingredient Name",
                                    Input::new(&self.demo_input),
                                ),
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
                                select_field(
                                    "Measurement Unit",
                                    Select::new(&self.demo_select),
                                ),
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
                                date_picker_field(
                                    "Scheduled Meal Date",
                                    DatePicker::new(&self.demo_date_picker),
                                ),
                            )
                            .child(
                                Button::new("btn-open-dialog-demo")
                                    .primary()
                                    .label("Open Interactive Modal Dialog")
                                    .on_click(cx.listener(|_this, _event, window, cx| {
                                        let batch_input = cx.new(|cx| {
                                            InputState::new(window, cx)
                                                .placeholder("2.0")
                                                .default_value("2.0")
                                        });
                                        let view = cx.entity().clone();
                                        window.open_dialog(cx, move |dialog, _, _| {
                                            let view_confirm = view.clone();
                                            let b_in = batch_input.clone();
                                            dialog
                                                .w(px(500.))
                                                .content(move |content, _, _cx| {
                                                    let v_confirm = view_confirm.clone();
                                                    let b_save = b_in.clone();
                                                    content
                                                        .child(
                                                            DialogHeader::new()
                                                                .child(DialogTitle::new().child("Make Recipe Configuration"))
                                                                .child(DialogDescription::new().child("Configure batches, yields, and optional substitutes before batching")),
                                                        )
                                                        .child(
                                                            div()
                                                                .py_4()
                                                                .flex()
                                                                .flex_col()
                                                                .gap_3()
                                                                .child(
                                                                    form_field(
                                                                        "Number of Batches",
                                                                        Input::new(&b_in),
                                                                    ),
                                                                )
                                                                .child(
                                                                    Checkbox::new("modal-cb-include-optionals")
                                                                        .label("Include optional ingredients")
                                                                        .checked(true),
                                                                ),
                                                        )
                                                        .child(
                                                            DialogFooter::new()
                                                                .child(
                                                                    Button::new("btn-modal-cancel")
                                                                        .secondary()
                                                                        .label("Cancel")
                                                                        .on_click(|_, window, cx| {
                                                                            window.close_dialog(cx);
                                                                        }),
                                                                )
                                                                .child(
                                                                    Button::new("btn-modal-confirm")
                                                                        .primary()
                                                                        .label("Produce Batches into Pantry")
                                                                        .on_click(move |_, window, cx| {
                                                                            let batches = b_save.read(cx).value().to_string();
                                                                            v_confirm.update(cx, |this, cx| {
                                                                                this.status_msg = format!("Successfully executed Make Recipe production for {} batches!", batches);
                                                                                cx.notify();
                                                                            });
                                                                            window.close_dialog(cx);
                                                                        }),
                                                                ),
                                                        )
                                                })
                                        });
                                    })),
                            ),
                    ),
            )
    }
}
