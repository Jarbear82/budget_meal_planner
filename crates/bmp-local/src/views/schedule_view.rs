use bmp_services::AppServices;
use gpui::*;
use gpui_component::badge::Badge;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::ActiveTheme;

pub struct ScheduleView {
    pub services: AppServices,
}

impl ScheduleView {
    pub fn new(services: AppServices) -> Self {
        Self { services }
    }
}

impl Render for ScheduleView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let scheduled_meals = self.services.meals.list_scheduled_meals().unwrap_or_default();

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
                            .child("Meal Scheduler & Calendar"),
                    )
                    .child(
                        Button::new("btn-schedule-meal")
                            .primary()
                            .label("+ Schedule Meal"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_4()
                    .bg(cx.theme().background)
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded_lg()
                    .child(div().text_xs().font_weight(FontWeight::BOLD).text_color(cx.theme().muted_foreground).child("SCHEDULED MEALS"))
                    .children(scheduled_meals.iter().map(|m| {
                        let status_str = if m.consumed.is_some() { "Consumed" } else { "Scheduled" };
                        div()
                            .flex()
                            .justify_between()
                            .items_center()
                            .p_3()
                            .rounded_md()
                            .bg(cx.theme().muted)
                            .text_sm()
                            .text_color(cx.theme().foreground)
                            .child(div().font_weight(FontWeight::BOLD).child(format!("Meal ID: {}", m.id)))
                            .child(div().child(Badge::new().child(format!("People: {}", m.people))))
                            .child(div().child(Badge::new().child(status_str)))
                    })),
            )
    }
}
