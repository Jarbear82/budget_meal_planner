use gpui::*;
use gpui_component::{ActiveTheme, Theme, ThemeMode};

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
        let current_mode = cx.theme().mode;
        let next_mode_label = if current_mode.is_dark() { "☀️ Light Mode" } else { "🌙 Dark Mode" };

        div()
            .flex()
            .items_center()
            .justify_between()
            .px_4()
            .py_2()
            .bg(cx.theme().background)
            .border_b_1()
            .border_color(cx.theme().border)
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
                    .gap_3()
                    // Theme Switcher Button
                    .child(
                        div()
                            .id("theme-toggle-btn")
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .bg(cx.theme().muted)
                            .hover(|s| s.bg(cx.theme().accent))
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(cx.theme().foreground)
                            .on_click(cx.listener(move |_this, _, _window, cx| {
                                let new_mode = if cx.theme().mode.is_dark() {
                                    ThemeMode::Light
                                } else {
                                    ThemeMode::Dark
                                };
                                Theme::change(new_mode, None, cx);
                            }))
                            .child(next_mode_label),
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
                    )
                    // Custom Title Bar Window Controls (Minimize, Maximize, Close)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_1()
                            .child(
                                div()
                                    .id("titlebar-minimize")
                                    .px_3()
                                    .py_1()
                                    .rounded_md()
                                    .bg(cx.theme().muted)
                                    .hover(|s| s.bg(cx.theme().accent))
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(cx.theme().foreground)
                                    .window_control_area(WindowControlArea::Min)
                                    .on_mouse_down(MouseButton::Left, |_, window, cx| {
                                        window.prevent_default();
                                        cx.stop_propagation();
                                    })
                                    .on_click(|_, window, cx| {
                                        cx.stop_propagation();
                                        window.minimize_window();
                                    })
                                    .child("—"),
                            )
                            .child(
                                div()
                                    .id("titlebar-maximize")
                                    .px_3()
                                    .py_1()
                                    .rounded_md()
                                    .bg(cx.theme().muted)
                                    .hover(|s| s.bg(cx.theme().accent))
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(cx.theme().foreground)
                                    .window_control_area(WindowControlArea::Max)
                                    .on_mouse_down(MouseButton::Left, |_, window, cx| {
                                        window.prevent_default();
                                        cx.stop_propagation();
                                    })
                                    .on_click(|_, window, cx| {
                                        cx.stop_propagation();
                                        window.zoom_window();
                                    })
                                    .child("□"),
                            )
                            .child(
                                div()
                                    .id("titlebar-close")
                                    .px_3()
                                    .py_1()
                                    .rounded_md()
                                    .bg(cx.theme().muted)
                                    .hover(|s| s.bg(cx.theme().danger).text_color(cx.theme().danger_foreground))
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(cx.theme().danger)
                                    .window_control_area(WindowControlArea::Close)
                                    .on_mouse_down(MouseButton::Left, |_, window, cx| {
                                        window.prevent_default();
                                        cx.stop_propagation();
                                    })
                                    .on_click(|_, window, cx| {
                                        cx.stop_propagation();
                                        window.remove_window();
                                    })
                                    .child("✕"),
                            ),
                    ),
            )
    }
}
