use crate::components::*;
use bmp_services::AppServices;
use gpui::prelude::*;
use gpui::*;
use gpui_component::scroll::ScrollableElement;
use gpui_component::table::{Table, TableBody, TableCell, TableHead, TableHeader, TableRow};
use gpui_component::tag::Tag;
use gpui_component::ActiveTheme;
use rust_decimal::Decimal;

pub struct AnalyticsView {
    pub services: AppServices,
    pub date_filter: String, // "all", "7d", "30d"
}

impl AnalyticsView {
    pub fn new(services: AppServices) -> Self {
        Self {
            services,
            date_filter: "all".to_string(),
        }
    }
}

impl Render for AnalyticsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let summary = self.services.analytics.get_overall_summary().ok();
        let receipts = self.services.storage.get_all_receipts().unwrap_or_default();
        let stores = self.services.items.list_stores().unwrap_or_default();

        let projected_str = summary
            .as_ref()
            .map(|s| format!("${}", s.projected_cost.normalize()))
            .unwrap_or_else(|| "$0.00".to_string());

        let actual_str = summary
            .as_ref()
            .map(|s| format!("${}", s.actual_expenditure.normalize()))
            .unwrap_or_else(|| "$0.00".to_string());

        let variance_str = summary
            .as_ref()
            .map(|s| {
                let v = s.variance;
                if v > Decimal::ZERO {
                    format!("+${} Over Budget", v.normalize())
                } else if v < Decimal::ZERO {
                    format!("-${} Under Budget", (-v).normalize())
                } else {
                    "$0.00 On Target".to_string()
                }
            })
            .unwrap_or_else(|| "$0.00".to_string());

        let date_options = vec![
            SelectOption::new("all", "All Time"),
            SelectOption::new("7d", "Past 7 Days"),
            SelectOption::new("30d", "Past 30 Days"),
        ];

        div()
            .flex()
            .flex_col()
            .gap_4()
            .size_full()
            .p_6()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            // Header Bar
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(cx.theme().foreground)
                                    .child("Analytics & Financial Overview"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Projected vs. actual expenditure analytics, price breakdowns, and receipt logs"),
                            ),
                    )
                    .child(
                        div()
                            .w_48()
                            .child(
                                Select::new("select-analytics-date-filter", date_options)
                                    .selected_id(Some(self.date_filter.clone()))
                                    .on_select(cx.listener(|this, opt: &SelectOption, _window, cx| {
                                        this.date_filter = opt.id.clone();
                                        cx.notify();
                                    })),
                            ),
                    ),
            )
            // Metric Cards Grid
            .child(
                div()
                    .grid()
                    .grid_cols(3)
                    .gap_4()
                    // Card 1: Projected Cost
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .p_5()
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_xl()
                            .shadow_sm()
                            .child(div().text_xs().text_color(cx.theme().muted_foreground).child("Projected Shopping Cost"))
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(cx.theme().foreground)
                                    .child(projected_str),
                            ),
                    )
                    // Card 2: Actual Expenditure
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .p_5()
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_xl()
                            .shadow_sm()
                            .child(div().text_xs().text_color(cx.theme().muted_foreground).child("Actual Spend (Receipts)"))
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(cx.theme().foreground)
                                    .child(actual_str),
                            ),
                    )
                    // Card 3: Expenditure Variance
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .p_5()
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_xl()
                            .shadow_sm()
                            .child(div().text_xs().text_color(cx.theme().muted_foreground).child("Budget Variance"))
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(cx.theme().foreground)
                                    .child(variance_str),
                            ),
                    ),
            )
            // Receipts History Breakdown Table
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_5()
                    .bg(cx.theme().background)
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded_lg()
                    .flex_1()
                    .overflow_y_scrollbar()
                    .child(
                        Table::new()
                            .child(
                                TableHeader::new()
                                    .child(TableHead::new().child("Receipt Details"))
                                    .child(TableHead::new().child("Store"))
                                    .child(TableHead::new().child("Actual Amount Spent ($)")),
                            )
                            .child(
                                TableBody::new().children(receipts.into_iter().map(|(id_str, store_id_opt, total, dt)| {
                                    let store_name = store_id_opt
                                        .and_then(|sid| stores.iter().find(|s| s.id == sid).map(|s| s.name.clone()))
                                        .unwrap_or_else(|| "General Store / Unspecified".to_string());

                                    let date_fmt = dt.format("%Y-%m-%d %H:%M").to_string();

                                    TableRow::new()
                                        .child(
                                            TableCell::new().child(
                                                div()
                                                    .flex()
                                                    .flex_col()
                                                    .child(
                                                        div()
                                                            .font_weight(FontWeight::BOLD)
                                                            .text_sm()
                                                            .text_color(cx.theme().foreground)
                                                            .child(format!("Receipt #{}", id_str)),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .text_color(cx.theme().muted_foreground)
                                                            .child(format!("Recorded on: {}", date_fmt)),
                                                    ),
                                            ),
                                        )
                                        .child(TableCell::new().child(Tag::new().child(store_name)))
                                        .child(
                                            TableCell::new().child(
                                                div()
                                                    .font_weight(FontWeight::BOLD)
                                                    .text_base()
                                                    .text_color(cx.theme().foreground)
                                                    .child(format!("${}", total.normalize())),
                                            ),
                                        )
                                })),
                            ),
                    ),
            )
    }
}
