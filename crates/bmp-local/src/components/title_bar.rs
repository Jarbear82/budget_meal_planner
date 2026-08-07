use gpui::*;
use gpui_component::badge::Badge;
use gpui_component::switch::Switch;
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
                            .child(Badge::new().child(self.status_badge.clone())),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_4()
                            // Standard Switch component for Dark / Light Mode
                            .child(
                                Switch::new("theme-mode-switch")
                                    .checked(is_dark)
                                    .label(if is_dark { "Dark Mode" } else { "Light Mode" })
                                    .on_click(cx.listener(move |_this, checked, _window, cx| {
                                        let new_mode = if *checked {
                                            ThemeMode::Dark
                                        } else {
                                            ThemeMode::Light
                                        };
                                        Theme::change(new_mode, None, cx);
                                    })),
                            )
                            .child(Badge::new().child("Offline Mode")),
                    ),
            )
    }
}
