pub use gpui_component::input::{Input, InputState};

use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::ActiveTheme;
use rust_decimal::Decimal;
use std::sync::Arc;

/// A clean form field container pairing a semantic label with a GPUI input widget.
pub fn form_field(
    label: impl Into<SharedString>,
    input: impl IntoElement,
) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_1()
        .w_full()
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .child(label.into()),
        )
        .child(input)
}

/// A specialized Numeric Input component with step buttons (+/-) built with native gpui_component Button primitives.
#[derive(IntoElement)]
pub struct NumberInput {
    id: ElementId,
    label: Option<SharedString>,
    value: Decimal,
    step: Decimal,
    min: Option<Decimal>,
    max: Option<Decimal>,
    unit: Option<SharedString>,
    disabled: bool,
    on_increment: Option<Arc<dyn Fn(&Decimal, &mut Window, &mut App) + 'static>>,
    on_decrement: Option<Arc<dyn Fn(&Decimal, &mut Window, &mut App) + 'static>>,
}

impl NumberInput {
    pub fn new(id: impl Into<ElementId>, value: Decimal) -> Self {
        Self {
            id: id.into(),
            label: None,
            value,
            step: Decimal::ONE,
            min: None,
            max: None,
            unit: None,
            disabled: false,
            on_increment: None,
            on_decrement: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn step(mut self, step: Decimal) -> Self {
        self.step = step;
        self
    }

    pub fn min(mut self, min: Decimal) -> Self {
        self.min = Some(min);
        self
    }

    pub fn max(mut self, max: Decimal) -> Self {
        self.max = Some(max);
        self
    }

    pub fn unit(mut self, unit: impl Into<SharedString>) -> Self {
        self.unit = Some(unit.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_increment<F>(mut self, callback: F) -> Self
    where
        F: Fn(&Decimal, &mut Window, &mut App) + 'static,
    {
        self.on_increment = Some(Arc::new(callback));
        self
    }

    pub fn on_decrement<F>(mut self, callback: F) -> Self
    where
        F: Fn(&Decimal, &mut Window, &mut App) + 'static,
    {
        self.on_decrement = Some(Arc::new(callback));
        self
    }
}

impl RenderOnce for NumberInput {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let value_str = format!("{}", self.value.normalize());
        let val = self.value;
        let step = self.step;
        let min_val = self.min;
        let max_val = self.max;

        let on_inc = self.on_increment;
        let on_dec = self.on_decrement;

        div()
            .id(self.id)
            .flex()
            .flex_col()
            .gap_1()
            .when_some(self.label, |this, label| {
                this.child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(cx.theme().foreground)
                        .child(label),
                )
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded_md()
                    .bg(cx.theme().background)
                    // Decrement button
                    .child(
                        Button::new("btn-number-dec")
                            .secondary()
                            .label("-")
                            .on_click(move |_event, window, cx| {
                                let next_val = val - step;
                                if let Some(m) = min_val {
                                    if next_val < m {
                                        return;
                                    }
                                }
                                if let Some(ref cb) = on_dec {
                                    cb(&next_val, window, cx);
                                }
                            }),
                    )
                    // Value & Unit display
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .items_center()
                            .justify_center()
                            .gap_1()
                            .px_2()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(cx.theme().foreground)
                            .child(value_str)
                            .when_some(self.unit, |this, unit| {
                                this.child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child(unit),
                                )
                            }),
                    )
                    // Increment button
                    .child(
                        Button::new("btn-number-inc")
                            .secondary()
                            .label("+")
                            .on_click(move |_event, window, cx| {
                                let next_val = val + step;
                                if let Some(m) = max_val {
                                    if next_val > m {
                                        return;
                                    }
                                }
                                if let Some(ref cb) = on_inc {
                                    cb(&next_val, window, cx);
                                }
                            }),
                    ),
            )
    }
}
