use gpui::prelude::*;
use gpui::*;
use gpui_component::ActiveTheme;
use std::sync::Arc;

/// A binary selection Checkbox component with label and custom indicators.
#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    label: Option<SharedString>,
    checked: bool,
    disabled: bool,
    helper_text: Option<SharedString>,
    on_click: Option<Arc<dyn Fn(&bool, &mut Window, &mut App) + 'static>>,
}

impl Checkbox {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            checked: false,
            disabled: false,
            helper_text: None,
            on_click: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn helper_text(mut self, helper: impl Into<SharedString>) -> Self {
        self.helper_text = Some(helper.into());
        self
    }

    pub fn on_click<F>(mut self, callback: F) -> Self
    where
        F: Fn(&bool, &mut Window, &mut App) + 'static,
    {
        self.on_click = Some(Arc::new(callback));
        self
    }
}

impl RenderOnce for Checkbox {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let checked = self.checked;
        let disabled = self.disabled;
        let on_click = self.on_click;

        let bg_color = if checked {
            cx.theme().primary
        } else {
            cx.theme().background
        };

        let border_color = if checked {
            cx.theme().primary
        } else {
            cx.theme().border
        };

        div()
            .id(self.id)
            .flex()
            .items_start()
            .gap_2_5()
            .cursor_pointer()
            .when(!disabled, |this| {
                this.on_click(move |_event, window, cx| {
                    if let Some(ref cb) = on_click {
                        let next_checked = !checked;
                        cb(&next_checked, window, cx);
                    }
                })
            })
            // Checkbox square indicator box
            .child(
                div()
                    .w_4()
                    .h_4()
                    .mt_0p5()
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_sm()
                    .border_1()
                    .border_color(border_color)
                    .bg(bg_color)
                    .child(if checked {
                        div()
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(cx.theme().primary_foreground)
                            .child("✓")
                    } else {
                        div()
                    }),
            )
            // Label & helper text
            .when_some(self.label, |this, label| {
                this.child(
                    div()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(if disabled {
                                    cx.theme().muted_foreground
                                } else {
                                    cx.theme().foreground
                                })
                                .child(label),
                        )
                        .when_some(self.helper_text, |this, helper| {
                            this.child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(helper),
                            )
                        }),
                )
            })
    }
}
