use bmp_domain::*;
use bmp_services::AppServices;
use chrono::Utc;
use gpui::*;
use gpui_component::badge::Badge;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::ActiveTheme;
use rust_decimal_macros::dec;

pub struct ScheduleView {
    pub services: AppServices,
    pub status_msg: String,
}

impl ScheduleView {
    pub fn new(services: AppServices) -> Self {
        Self {
            services,
            status_msg: "Ready".to_string(),
        }
    }

    pub fn schedule_sample_meal(&mut self, cx: &mut Context<Self>) {
        if let Ok(recipes) = self.services.recipes.list_recipes() {
            if let Some(r) = recipes.first() {
                let source = ScheduledMealSource::OneOff(MealComponent::Recipe {
                    recipe_id: r.id,
                    servings: dec!(4),
                });
                match self.services.meals.schedule_meal(source, Utc::now(), 2) {
                    Ok(meal) => {
                        self.status_msg = format!("Scheduled meal ID: {}", meal.id);
                    }
                    Err(e) => {
                        self.status_msg = format!("Error: {}", e);
                    }
                }
            } else {
                self.status_msg = "No recipes available to schedule. Create a recipe first.".to_string();
            }
        }
        cx.notify();
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
                            .label("+ Schedule Meal")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.schedule_sample_meal(cx);
                            })),
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
                    .child(div().text_xs().font_weight(FontWeight::BOLD).text_color(cx.theme().muted_foreground).child(format!("SCHEDULED MEALS ({})", self.status_msg)))
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
