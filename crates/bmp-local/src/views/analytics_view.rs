use bmp_services::AppServices;
use gpui::*;
use gpui_component::ActiveTheme;
use rust_decimal::Decimal;

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
        let summary = self
            .services
            .analytics
            .get_overall_summary()
            .unwrap_or(bmp_services::AnalyticsSummary {
                projected_cost: Decimal::ZERO,
                actual_expenditure: Decimal::ZERO,
                variance: Decimal::ZERO,
            });

        let proj_str = format!("${:.2}", summary.projected_cost);
        let actual_str = format!("${:.2}", summary.actual_expenditure);

        let pct = if summary.projected_cost > Decimal::ZERO {
            (summary.variance / summary.projected_cost) * Decimal::from(100)
        } else {
            Decimal::ZERO
        };
        let var_str = format!("${:.2} ({:.1}%)", summary.variance, pct);

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
                            .child(div().text_xl().font_weight(FontWeight::BOLD).text_color(rgb(0x10b981)).child(proj_str)),
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
                            .child(div().text_xl().font_weight(FontWeight::BOLD).text_color(rgb(0x38bdf8)).child(actual_str)),
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
                            .child(div().text_xl().font_weight(FontWeight::BOLD).text_color(cx.theme().foreground).child(var_str)),
                    ),
            )
    }
}
