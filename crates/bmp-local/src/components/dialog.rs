use gpui::prelude::*;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::dialog::{DialogContent, DialogTitle};
use gpui_component::scroll::ScrollableElement;
use gpui_component::ActiveTheme;
use std::sync::Arc;

/// A reusable Modal Dialog container component built on native gpui_component dialog elements.
#[derive(IntoElement)]
pub struct Dialog {
    id: ElementId,
    title: SharedString,
    subtitle: Option<SharedString>,
    is_open: bool,
    children: Vec<AnyElement>,
    footer_actions: Vec<AnyElement>,
    on_close: Option<Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl Dialog {
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            subtitle: None,
            is_open: false,
            children: Vec::new(),
            footer_actions: Vec::new(),
            on_close: None,
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn is_open(mut self, open: bool) -> Self {
        self.is_open = open;
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub fn footer_action(mut self, action: impl IntoElement) -> Self {
        self.footer_actions.push(action.into_any_element());
        self
    }

    pub fn on_close<F>(mut self, callback: F) -> Self
    where
        F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    {
        self.on_close = Some(Arc::new(callback));
        self
    }
}

impl RenderOnce for Dialog {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        if !self.is_open {
            return div().into_any_element();
        }

        let on_close = self.on_close;
        let on_close_backdrop = on_close.clone();
        let on_close_btn = on_close.clone();

        div()
            .id(self.id)
            .absolute()
            .top_0()
            .left_0()
            .w_full()
            .h_full()
            .flex()
            .items_center()
            .justify_center()
            .child(
                // Backdrop / Dimmed overlay
                div()
                    .id("dialog-backdrop")
                    .absolute()
                    .top_0()
                    .left_0()
                    .w_full()
                    .h_full()
                    .bg(cx.theme().background)
                    .on_click(move |event, window, cx| {
                        if let Some(ref cb) = on_close_backdrop {
                            cb(event, window, cx);
                        }
                    }),
            )
            .child(
                // Modal Window Box built with DialogContent & DialogTitle
                div()
                    .id("dialog-content-box")
                    .relative()
                    .w_full()
                    .max_w_96()
                    .mx_4()
                    .bg(cx.theme().background)
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded_xl()
                    .shadow_lg()
                    .flex()
                    .flex_col()
                    // Header Section with native DialogTitle
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .justify_between()
                            .p_5()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_0p5()
                                    .child(DialogTitle::new().child(self.title))
                                    .when_some(self.subtitle, |this, sub| {
                                        this.child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(sub),
                                        )
                                    }),
                            )
                            .child(
                                Button::new("btn-dialog-close")
                                    .ghost()
                                    .label("✕")
                                    .on_click(move |event, window, cx| {
                                        if let Some(ref cb) = on_close_btn {
                                            cb(event, window, cx);
                                        }
                                    }),
                            ),
                    )
                    // Body Content wrapped in DialogContent
                    .child(
                        DialogContent::new().child(
                            div()
                                .p_5()
                                .flex()
                                .flex_col()
                                .gap_4()
                                .max_h_96()
                                .overflow_y_scrollbar()
                                .children(self.children),
                        ),
                    )
                    // Footer Actions
                    .when(!self.footer_actions.is_empty(), |this| {
                        this.child(
                            div()
                                .flex()
                                .items_center()
                                .justify_end()
                                .gap_2()
                                .p_4()
                                .border_t_1()
                                .border_color(cx.theme().border)
                                .bg(cx.theme().muted)
                                .rounded_b_xl()
                                .children(self.footer_actions),
                        )
                    }),
            )
            .into_any_element()
    }
}
