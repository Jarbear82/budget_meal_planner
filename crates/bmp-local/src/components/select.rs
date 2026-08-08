pub use gpui_component::select::{Select as GpuiSelect, SelectState};

use gpui::prelude::*;
use gpui::*;
use gpui_component::scroll::ScrollableElement;
use gpui_component::ActiveTheme;
use std::sync::Arc;

/// A single option for the Select / Combobox component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectOption {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
}

impl SelectOption {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            description: None,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// A clean, interactive Select / Dropdown component for picking options.
#[derive(IntoElement)]
pub struct Select {
    id: ElementId,
    label: Option<SharedString>,
    placeholder: SharedString,
    options: Vec<SelectOption>,
    selected_id: Option<String>,
    is_open: bool,
    full_width: bool,
    on_select: Option<Arc<dyn Fn(&SelectOption, &mut Window, &mut App) + 'static>>,
    on_toggle: Option<Arc<dyn Fn(&bool, &mut Window, &mut App) + 'static>>,
}

impl Select {
    pub fn new(id: impl Into<ElementId>, options: Vec<SelectOption>) -> Self {
        Self {
            id: id.into(),
            label: None,
            placeholder: SharedString::from("Select an option..."),
            options,
            selected_id: None,
            is_open: false,
            full_width: true,
            on_select: None,
            on_toggle: None,
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

    pub fn selected_id(mut self, id: Option<impl Into<String>>) -> Self {
        self.selected_id = id.map(|i| i.into());
        self
    }

    pub fn is_open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        self
    }

    pub fn on_select<F>(mut self, callback: F) -> Self
    where
        F: Fn(&SelectOption, &mut Window, &mut App) + 'static,
    {
        self.on_select = Some(Arc::new(callback));
        self
    }

    pub fn on_toggle<F>(mut self, callback: F) -> Self
    where
        F: Fn(&bool, &mut Window, &mut App) + 'static,
    {
        self.on_toggle = Some(Arc::new(callback));
        self
    }
}

impl RenderOnce for Select {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let selected_option = self
            .selected_id
            .as_ref()
            .and_then(|id| self.options.iter().find(|opt| opt.id == *id).cloned());

        let display_label = match &selected_option {
            Some(opt) => opt.label.clone(),
            None => self.placeholder.to_string(),
        };
        let is_selected = selected_option.is_some();

        let is_open = self.is_open;
        let on_toggle = self.on_toggle;
        let on_select = self.on_select;
        let options = self.options.clone();

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
                    .relative()
                    .child(
                        // Main button / trigger
                        div()
                            .id("select-trigger")
                            .flex()
                            .items_center()
                            .justify_between()
                            .px_3()
                            .py_2()
                            .rounded_md()
                            .border_1()
                            .border_color(cx.theme().border)
                            .bg(cx.theme().background)
                            .cursor_pointer()
                            .hover(|s| s.bg(cx.theme().muted))
                            .text_sm()
                            .text_color(if is_selected {
                                cx.theme().foreground
                            } else {
                                cx.theme().muted_foreground
                            })
                            .on_click(move |_event, window, cx| {
                                if let Some(ref cb) = on_toggle {
                                    let next_open = !is_open;
                                    cb(&next_open, window, cx);
                                }
                            })
                            .child(display_label)
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(if is_open { "▲" } else { "▼" }),
                            ),
                    )
                    // Dropdown menu popover when is_open == true
                    .when(is_open, |this| {
                        this.child(
                            div()
                                .id("select-dropdown-menu")
                                .absolute()
                                .top_full()
                                .left_0()
                                .w_full()
                                .mt_1()
                                .bg(cx.theme().background)
                                .border_1()
                                .border_color(cx.theme().border)
                                .rounded_md()
                                .shadow_lg()
                                .max_h_96()
                                .overflow_y_scrollbar()
                                .flex()
                                .flex_col()
                                .py_1()
                                .children(options.into_iter().map(|opt| {
                                    let opt_clone = opt.clone();
                                    let on_select_cb = on_select.clone();
                                    let is_curr_selected = self
                                        .selected_id
                                        .as_ref()
                                        .map(|id| *id == opt.id)
                                        .unwrap_or(false);

                                    let opt_id = format!("select-opt-{}", opt.id);
                                    div()
                                        .id(ElementId::from(opt_id))
                                        .flex()
                                        .flex_col()
                                        .px_3()
                                        .py_2()
                                        .cursor_pointer()
                                        .bg(if is_curr_selected {
                                            cx.theme().accent
                                        } else {
                                            cx.theme().background
                                        })
                                        .hover(|s| s.bg(cx.theme().muted))
                                        .on_click(move |_event, window, cx| {
                                            if let Some(ref cb) = on_select_cb {
                                                cb(&opt_clone, window, cx);
                                            }
                                        })
                                        .child(
                                            div()
                                                .text_sm()
                                                .font_weight(if is_curr_selected {
                                                    FontWeight::BOLD
                                                } else {
                                                    FontWeight::NORMAL
                                                })
                                                .text_color(cx.theme().foreground)
                                                .child(opt.label),
                                        )
                                        .when_some(opt.description, |this, desc| {
                                            this.child(
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child(desc),
                                            )
                                        })
                                })),
                        )
                    }),
            )
    }
}
