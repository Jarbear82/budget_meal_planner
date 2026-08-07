use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::{ActiveTheme, Theme, ThemeMode, TitleBar as ComponentTitleBar};

pub struct TitleBar {
    pub title: String,
    pub status_badge: String,
}

impl TitleBar {
    pub fn new(title: impl Into<String>, status_badge: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            status_badge: status_badge.into(),
        }
    }
}

impl Render for TitleBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let is_dark = cx.theme().mode.is_dark();

        ComponentTitleBar::new()
            .on_close_window(|_, _window, cx| {
                cx.quit();
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .w_full()
                    .px_2()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_3()
                            .child(
                                div()
                                    .text_base()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(cx.theme().foreground)
                                    .child(self.title.clone()),
                            )
                            .child(
                                div()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_md()
                                    .bg(cx.theme().muted)
                                    .text_xs()
                                    .text_color(rgb(0x10b981))
                                    .child(self.status_badge.clone()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            // Interactive Switch Toggle for Light / Dark Mode
                            .child(
                                div()
                                    .id("theme-switch-toggle")
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .px_2p5()
                                    .py_1()
                                    .rounded_full()
                                    .bg(cx.theme().muted)
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .on_click(cx.listener(move |_this, _, _window, cx| {
                                        let new_mode = if cx.theme().mode.is_dark() {
                                            ThemeMode::Light
                                        } else {
                                            ThemeMode::Dark
                                        };
                                        Theme::change(new_mode, None, cx);
                                    }))
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(if is_dark { cx.theme().foreground } else { cx.theme().muted_foreground })
                                            .child("🌙"),
                                    )
                                    // Pill Track & Sliding Knob
                                    .child(
                                        div()
                                            .w_8()
                                            .h_4()
                                            .rounded_full()
                                            .bg(if is_dark { cx.theme().primary } else { cx.theme().accent })
                                            .flex()
                                            .items_center()
                                            .px_0p5()
                                            .child(
                                                div()
                                                    .w_3()
                                                    .h_3()
                                                    .rounded_full()
                                                    .bg(cx.theme().background)
                                                    .when(is_dark, |s| s.ml_auto()),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(if !is_dark { cx.theme().foreground } else { cx.theme().muted_foreground })
                                            .child("☀️"),
                                    ),
                            )
                            .child(
                                div()
                                    .px_2p5()
                                    .py_1()
                                    .rounded_md()
                                    .bg(cx.theme().muted)
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Offline Mode"),
                            ),
                    ),
            )
    }
}
