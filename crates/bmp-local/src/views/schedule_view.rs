use bmp_services::AppServices;
use gpui::*;

pub struct ScheduleView {
    pub services: AppServices,
}

impl ScheduleView {
    pub fn new(services: AppServices) -> Self {
        Self { services }
    }
}

impl Render for ScheduleView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let scheduled_meals = self.services.meals.list_scheduled_meals().unwrap_or_default();

        div()
            .flex()
            .flex_col()
            .gap_4()
            .size_full()
            .p_6()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xf4f4f5))
                            .child("Meal Scheduler & Calendar"),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .bg(rgb(0x38bdf8))
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x18181b))
                            .child("+ Schedule Meal"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_4()
                    .bg(rgb(0x18181b))
                    .border_1()
                    .border_color(rgb(0x27272a))
                    .rounded_lg()
                    .child(div().text_xs().font_weight(FontWeight::BOLD).text_color(rgb(0xa1a1aa)).child("SCHEDULED MEALS"))
                    .children(scheduled_meals.iter().map(|m| {
                        let status_str = if m.consumed.is_some() { "Consumed" } else { "Scheduled" };
                        div()
                            .flex()
                            .justify_between()
                            .p_3()
                            .rounded_md()
                            .bg(rgb(0x27272a))
                            .text_sm()
                            .child(div().font_weight(FontWeight::BOLD).child(format!("Meal ID: {}", m.id)))
                            .child(div().text_color(rgb(0x10b981)).child(format!("People: {}", m.people)))
                            .child(div().text_color(rgb(0x38bdf8)).child(status_str))
                    })),
            )
    }
}
