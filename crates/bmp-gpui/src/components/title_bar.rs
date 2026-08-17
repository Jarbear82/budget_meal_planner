use gpui::prelude::*;
use gpui::*;
use gpui_component::TitleBar as GpuiTitleBar;
use gpui_component::ActiveTheme;

#[derive(IntoElement)]
pub struct TitleBar {
    pub title: String,
    pub status: String,
}

impl TitleBar {
    pub fn new(title: impl Into<String>, status: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            status: status.into(),
        }
    }
}

impl RenderOnce for TitleBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        GpuiTitleBar::new().child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .w_full()
                .px_4()
                .py_2()
                .bg(cx.theme().muted)
                .border_b_1()
                .border_color(cx.theme().border)
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(div().font_weight(FontWeight::BOLD).text_sm().text_color(cx.theme().foreground).child(self.title)),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("● {}", self.status)),
                ),
        )
    }
}
