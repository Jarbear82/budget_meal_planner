use bmp_services::AppServices;
use gpui::*;
use gpui_component::ActiveTheme;

pub struct AnalyticsView {
    pub services: AppServices,
}

impl AnalyticsView {
    pub fn new(services: AppServices) -> Self {
        Self { services }
    }
}

impl Render for AnalyticsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
                            .text_color(cx.theme().foreground)
                            .child("Financials & Cost Analytics"),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .bg(cx.theme().muted)
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child("Date Range: Monthly Summary"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_4()
                    .child(
                        div()
                            .flex_1()
                            .p_4()
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_lg()
                            .child(div().text_xs().text_color(cx.theme().muted_foreground).child("PROJECTED EXPENDITURE"))
                            .child(div().text_xl().font_weight(FontWeight::BOLD).text_color(rgb(0x10b981)).child("$0.00")),
                    )
                    .child(
                        div()
                            .flex_1()
                            .p_4()
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_lg()
                            .child(div().text_xs().text_color(cx.theme().muted_foreground).child("ACTUAL EXPENDITURE"))
                            .child(div().text_xl().font_weight(FontWeight::BOLD).text_color(rgb(0x38bdf8)).child("$0.00")),
                    )
                    .child(
                        div()
                            .flex_1()
                            .p_4()
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_lg()
                            .child(div().text_xs().text_color(cx.theme().muted_foreground).child("VARIANCE"))
                            .child(div().text_xl().font_weight(FontWeight::BOLD).text_color(cx.theme().foreground).child("$0.00 (0.0%)")),
                    ),
            )
    }
}
