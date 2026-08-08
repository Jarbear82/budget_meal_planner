use gpui::prelude::*;
use gpui::*;
use gpui_component::ActiveTheme;
use rust_decimal::Decimal;
use std::sync::Arc;

/// A clean, styled Form Input component for text entry.
#[derive(IntoElement)]
pub struct FormInput {
    id: ElementId,
    label: Option<SharedString>,
    placeholder: SharedString,
    value: SharedString,
    helper_text: Option<SharedString>,
    error_text: Option<SharedString>,
    disabled: bool,
    full_width: bool,
    on_change: Option<Arc<dyn Fn(&str, &mut Window, &mut App) + 'static>>,
}

impl FormInput {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            placeholder: SharedString::from("Enter text..."),
            value: SharedString::from(""),
            helper_text: None,
            error_text: None,
            disabled: false,
            full_width: true,
            on_change: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = value.into();
        self
    }

    pub fn helper_text(mut self, helper: impl Into<SharedString>) -> Self {
        self.helper_text = Some(helper.into());
        self
    }

    pub fn error_text(mut self, error: impl Into<SharedString>) -> Self {
        self.error_text = Some(error.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&str, &mut Window, &mut App) + 'static,
    {
        self.on_change = Some(Arc::new(callback));
        self
    }
}

impl RenderOnce for FormInput {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let has_error = self.error_text.is_some();
        let border_col = if has_error {
            cx.theme().accent
        } else {
            cx.theme().border
        };

        let is_empty = self.value.is_empty();
        let display_text = if is_empty {
            self.placeholder.clone()
        } else {
            self.value.clone()
        };

        div()
            .id(self.id)
            .flex()
            .flex_col()
            .gap_1()
            .when(self.full_width, |this| this.w_full())
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
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .border_1()
                    .border_color(border_col)
                    .bg(if self.disabled {
                        cx.theme().muted
                    } else {
                        cx.theme().background
                    })
                    .text_sm()
                    .text_color(if is_empty {
                        cx.theme().muted_foreground
                    } else {
                        cx.theme().foreground
                    })
                    .child(display_text),
            )
            .when(has_error, |this| {
                let err = self.error_text.unwrap();
                this.child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().accent)
                        .child(err),
                )
            })
            .when(!has_error, |this| {
                this.when_some(self.helper_text, |this, helper| {
                    this.child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(helper),
                    )
                })
            })
    }
}

/// A specialized Numeric Input component with step buttons (+/-) and optional unit suffix.
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
                        div()
                            .id("btn-dec")
                            .px_3()
                            .py_1_5()
                            .cursor_pointer()
                            .hover(|s| s.bg(cx.theme().muted))
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(cx.theme().foreground)
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
                            })
                            .child("-"),
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
                        div()
                            .id("btn-inc")
                            .px_3()
                            .py_1_5()
                            .cursor_pointer()
                            .hover(|s| s.bg(cx.theme().muted))
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(cx.theme().foreground)
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
                            })
                            .child("+"),
                    ),
            )
    }
}

