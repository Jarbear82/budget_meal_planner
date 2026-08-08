pub use gpui_component::date_picker::{
    DatePicker as GpuiDatePicker, DatePickerEvent, DatePickerState,
};

use chrono::{Datelike, Local, NaiveDate};
use gpui::prelude::*;
use gpui::*;
use gpui_component::ActiveTheme;
use std::sync::Arc;

/// A clean DatePicker component with calendar view and quick presets.
#[derive(IntoElement)]
pub struct DatePicker {
    id: ElementId,
    label: Option<SharedString>,
    selected_date: NaiveDate,
    current_view_month: NaiveDate,
    is_open: bool,
    on_change: Option<Arc<dyn Fn(&NaiveDate, &mut Window, &mut App) + 'static>>,
    on_toggle: Option<Arc<dyn Fn(&bool, &mut Window, &mut App) + 'static>>,
}

impl DatePicker {
    pub fn new(id: impl Into<ElementId>, selected_date: NaiveDate) -> Self {
        let first_of_month = selected_date.with_day(1).unwrap_or(selected_date);
        Self {
            id: id.into(),
            label: None,
            selected_date,
            current_view_month: first_of_month,
            is_open: false,
            on_change: None,
            on_toggle: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn is_open(mut self, is_open: bool) -> Self {
        self.is_open = is_open;
        self
    }

    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: Fn(&NaiveDate, &mut Window, &mut App) + 'static,
    {
        self.on_change = Some(Arc::new(callback));
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

impl RenderOnce for DatePicker {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let formatted = self.selected_date.format("%Y-%m-%d").to_string();
        let selected_date = self.selected_date;
        let is_open = self.is_open;
        let on_toggle = self.on_toggle;
        let on_change = self.on_change.clone();

        // Calculate days in current_view_month
        let year = self.current_view_month.year();
        let month = self.current_view_month.month();

        // Start day of week (0 = Mon, 6 = Sun)
        let first_day = NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(self.selected_date);
        let start_weekday = first_day.weekday().num_days_from_monday();

        // Days in month
        let days_in_month = if month == 12 {
            NaiveDate::from_ymd_opt(year + 1, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
        }
        .and_then(|d| d.pred_opt())
        .map(|d| d.day())
        .unwrap_or(30);

        let today = Local::now().date_naive();

        div()
            .id(self.id)
            .flex()
            .flex_col()
            .gap_1()
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
                        // Trigger button
                        div()
                            .id("datepicker-trigger")
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
                            .text_color(cx.theme().foreground)
                            .on_click(move |_event, window, cx| {
                                if let Some(ref cb) = on_toggle {
                                    let next_open = !is_open;
                                    cb(&next_open, window, cx);
                                }
                            })
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child("📅")
                                    .child(formatted),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(if is_open { "▲" } else { "▼" }),
                            ),
                    )
                    .when(is_open, |this| {
                        let on_change_preset = on_change.clone();
                        let on_change_preset2 = on_change.clone();
                        let on_change_preset3 = on_change.clone();

                        this.child(
                            div()
                                .id("datepicker-dropdown")
                                .absolute()
                                .top_full()
                                .left_0()
                                .mt_1()
                                .p_3()
                                .w_64()
                                .bg(cx.theme().background)
                                .border_1()
                                .border_color(cx.theme().border)
                                .rounded_md()
                                .shadow_lg()
                                .flex()
                                .flex_col()
                                .gap_2()
                                // Quick Presets Bar
                                .child(
                                    div()
                                        .flex()
                                        .gap_1()
                                        .justify_between()
                                        .child(
                                            div()
                                                .id("preset-today")
                                                .px_2()
                                                .py_1()
                                                .rounded_sm()
                                                .bg(cx.theme().muted)
                                                .cursor_pointer()
                                                .hover(|s| s.bg(cx.theme().accent))
                                                .text_xs()
                                                .on_click(move |_event, window, cx| {
                                                    if let Some(ref cb) = on_change_preset {
                                                        cb(&today, window, cx);
                                                    }
                                                })
                                                .child("Today"),
                                        )
                                        .child(
                                            div()
                                                .id("preset-tomorrow")
                                                .px_2()
                                                .py_1()
                                                .rounded_sm()
                                                .bg(cx.theme().muted)
                                                .cursor_pointer()
                                                .hover(|s| s.bg(cx.theme().accent))
                                                .text_xs()
                                                .on_click(move |_event, window, cx| {
                                                    if let Some(ref cb) = on_change_preset2 {
                                                        let tomorrow = today + chrono::Duration::days(1);
                                                        cb(&tomorrow, window, cx);
                                                    }
                                                })
                                                .child("Tomorrow"),
                                        )
                                        .child(
                                            div()
                                                .id("preset-week")
                                                .px_2()
                                                .py_1()
                                                .rounded_sm()
                                                .bg(cx.theme().muted)
                                                .cursor_pointer()
                                                .hover(|s| s.bg(cx.theme().accent))
                                                .text_xs()
                                                .on_click(move |_event, window, cx| {
                                                    if let Some(ref cb) = on_change_preset3 {
                                                        let next_week = today + chrono::Duration::days(7);
                                                        cb(&next_week, window, cx);
                                                    }
                                                })
                                                .child("+1 Week"),
                                        ),
                                )
                                // Month Header
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .py_1()
                                        .text_xs()
                                        .font_weight(FontWeight::BOLD)
                                        .child(format!("{} {}", first_day.format("%B"), year)),
                                )
                                // Day Names
                                .child(
                                    div()
                                        .flex()
                                        .gap_1()
                                        .justify_between()
                                        .text_center()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("Mo")
                                        .child("Tu")
                                        .child("We")
                                        .child("Th")
                                        .child("Fr")
                                        .child("Sa")
                                        .child("Su"),
                                )
                                // Days Grid
                                .child(
                                    div()
                                        .flex()
                                        .flex_wrap()
                                        .gap_1()
                                        .children((0..start_weekday).map(|_| div().w_7().h_6()))
                                        .children((1..=days_in_month).map(|day_num| {
                                            let date_val =
                                                NaiveDate::from_ymd_opt(year, month, day_num)
                                                    .unwrap_or(selected_date);
                                            let is_selected_day = date_val == selected_date;
                                            let is_today = date_val == today;
                                            let cb_opt = on_change.clone();

                                            let day_id = format!("day-btn-{}", day_num);
                                            div()
                                                .id(ElementId::from(day_id))
                                                .w_7()
                                                .h_6()
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .rounded_sm()
                                                .text_xs()
                                                .cursor_pointer()
                                                .bg(if is_selected_day {
                                                    cx.theme().primary
                                                } else if is_today {
                                                    cx.theme().accent
                                                } else {
                                                    cx.theme().background
                                                })
                                                .text_color(if is_selected_day {
                                                    cx.theme().primary_foreground
                                                } else {
                                                    cx.theme().foreground
                                                })
                                                .hover(|s| s.bg(cx.theme().muted))
                                                .on_click(move |_event, window, cx| {
                                                    if let Some(ref cb) = cb_opt {
                                                        cb(&date_val, window, cx);
                                                    }
                                                })
                                                .child(format!("{}", day_num))
                                        })),
                                ),
                        )
                    }),
            )
    }
}
