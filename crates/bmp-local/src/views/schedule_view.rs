use crate::components::*;
use bmp_domain::*;
use bmp_services::AppServices;
use chrono::{DateTime, Local, NaiveDate, Utc};
use gpui::prelude::*;
use gpui::*;
use gpui_component::alert::Alert;
use gpui_component::badge::Badge;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::tag::Tag;
use gpui_component::ActiveTheme;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::str::FromStr;

pub struct ScheduleView {
    pub services: AppServices,
    pub status_msg: String,

    // Modals
    pub show_schedule_modal: bool,
    pub show_consumed_modal: bool,

    // Schedule form state
    pub form_meal_type: String, // "recipe", "pre_planned", "restaurant"
    pub form_recipe_id: Option<RecipeId>,
    pub form_pre_planned_name: String,
    pub form_restaurant_name: String,
    pub form_restaurant_cost: Decimal,
    pub form_people: u32,
    pub form_date: NaiveDate,

    // Consumed modal state
    pub target_meal_id: Option<ScheduledMealId>,
}

impl ScheduleView {
    pub fn new(services: AppServices) -> Self {
        Self {
            services,
            status_msg: "Schedule manager ready".to_string(),

            show_schedule_modal: false,
            show_consumed_modal: false,

            form_meal_type: "recipe".to_string(),
            form_recipe_id: None,
            form_pre_planned_name: String::new(),
            form_restaurant_name: String::new(),
            form_restaurant_cost: dec!(25.00),
            form_people: 2,
            form_date: Local::now().date_naive(),

            target_meal_id: None,
        }
    }

    pub fn open_schedule_modal(&mut self, cx: &mut Context<Self>) {
        let recipes = self.services.recipes.list_recipes().unwrap_or_default();
        self.form_recipe_id = recipes.first().map(|r| r.id);
        self.form_meal_type = "recipe".to_string();
        self.form_people = 2;
        self.form_date = Local::now().date_naive();
        self.show_schedule_modal = true;
        cx.notify();
    }

    pub fn save_scheduled_meal(&mut self, cx: &mut Context<Self>) {
        let dt = match self.form_date.and_hms_opt(18, 0, 0) {
            Some(naive_dt) => DateTime::<Utc>::from_naive_utc_and_offset(naive_dt, Utc),
            None => Utc::now(),
        };

        let source = match self.form_meal_type.as_str() {
            "restaurant" => ScheduledMealSource::Restaurant {
                name: if self.form_restaurant_name.trim().is_empty() {
                    "Local Restaurant".to_string()
                } else {
                    self.form_restaurant_name.trim().to_string()
                },
                cost: self.form_restaurant_cost,
                leftover_yield: None,
            },
            "pre_planned" => {
                let name = if self.form_pre_planned_name.trim().is_empty() {
                    "Standard Family Dinner".to_string()
                } else {
                    self.form_pre_planned_name.trim().to_string()
                };
                let ppm = match self.services.meals.create_pre_planned_meal(&name, Vec::new()) {
                    Ok(m) => m,
                    Err(e) => {
                        self.status_msg = format!("Error: {}", e);
                        return;
                    }
                };
                ScheduledMealSource::PrePlanned(ppm.id)
            }
            _ => {
                let recipe_id = match self.form_recipe_id {
                    Some(id) => id,
                    None => {
                        self.status_msg = "Error: Select a recipe first".to_string();
                        return;
                    }
                };
                let comp = MealComponent::Recipe {
                    recipe_id,
                    servings: dec!(4),
                };
                ScheduledMealSource::OneOff(comp)
            }
        };

        match self.services.meals.schedule_meal(source, dt, self.form_people) {
            Ok(meal) => {
                self.status_msg = format!("Scheduled meal for {} people on {}", meal.people, self.form_date);
                self.show_schedule_modal = false;
            }
            Err(e) => {
                self.status_msg = format!("Error scheduling meal: {}", e);
            }
        }
        cx.notify();
    }

    pub fn prompt_confirm_consumed(&mut self, meal_id: ScheduledMealId, cx: &mut Context<Self>) {
        self.target_meal_id = Some(meal_id);
        self.show_consumed_modal = true;
        cx.notify();
    }

    pub fn execute_confirm_consumed(&mut self, cx: &mut Context<Self>) {
        let meal_id = match self.target_meal_id {
            Some(id) => id,
            None => return,
        };

        match self.services.meals.confirm_meal_consumed(meal_id, Utc::now()) {
            Ok(_) => {
                self.status_msg = "Confirmed meal consumed! Updated schedule & pantry.".to_string();
                self.show_consumed_modal = false;
            }
            Err(e) => {
                self.status_msg = format!("Error confirming consumed: {}", e);
            }
        }
        cx.notify();
    }
}

impl Render for ScheduleView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let scheduled_meals = self.services.meals.list_scheduled_meals().unwrap_or_default();
        let recipes = self.services.recipes.list_recipes().unwrap_or_default();

        let recipe_options: Vec<SelectOption> = recipes
            .iter()
            .map(|r| SelectOption::new(r.id.0.to_string(), r.name.clone()))
            .collect();

        let meal_type_options = vec![
            SelectOption::new("recipe", "Recipe Meal"),
            SelectOption::new("pre_planned", "Pre-Planned Combo Meal"),
            SelectOption::new("restaurant", "Restaurant Dining / Takeout"),
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
                                    .child("Meal Schedule & Calendar"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Schedule meals, scale people counts, and confirm consumption to update pantry inventory"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(Badge::new().child(format!("Scheduled Meals: {}", scheduled_meals.len())))
                            .child(
                                Button::new("btn-schedule-meal")
                                    .primary()
                                    .label("+ Schedule Meal")
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.open_schedule_modal(cx);
                                    })),
                            ),
                    ),
            )
            // Status Bar
            .child(Alert::new("schedule-status-alert", format!("Status: {}", self.status_msg)))
            // Scheduled Meals List
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
                    .children(scheduled_meals.into_iter().map(|meal| {
                        let meal_id = meal.id;
                        let is_consumed = meal.consumed.is_some();
                        let date_str = meal.datetime.format("%Y-%m-%d %H:%M").to_string();

                        let title_str = match &meal.source {
                            ScheduledMealSource::OneOff(comp) => match comp {
                                MealComponent::Recipe { recipe_id, .. } => recipes
                                    .iter()
                                    .find(|r| r.id == *recipe_id)
                                    .map(|r| r.name.clone())
                                    .unwrap_or_else(|| "One-Off Recipe".to_string()),
                                MealComponent::Item { .. } => "One-Off Item Meal".to_string(),
                                MealComponent::Restaurant { name, .. } => name.clone(),
                            },
                            ScheduledMealSource::PrePlanned(_) => "Pre-Planned Combo Meal".to_string(),
                            ScheduledMealSource::Restaurant { name, .. } => format!("Restaurant: {}", name),
                        };

                        let card_id = format!("scheduled-meal-{}", meal_id);
                        div()
                            .id(ElementId::from(card_id))
                            .flex()
                            .items_center()
                            .justify_between()
                            .p_4()
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_lg()
                            .bg(if is_consumed {
                                cx.theme().muted
                            } else {
                                cx.theme().background
                            })
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(div().font_weight(FontWeight::BOLD).text_base().child(title_str))
                                            .child(Badge::new().child(format!("{} People", meal.people))),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(format!("📅 Scheduled for: {}", date_str)),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(if is_consumed {
                                        Tag::new().child("✓ Consumed")
                                    } else {
                                        Tag::new().child("Pending")
                                    })
                                    .when(!is_consumed, |this| {
                                        this.child(
                                            Button::new(format!("btn-confirm-consumed-{}", meal_id))
                                                .primary()
                                                .label("Confirm Consumed")
                                                .on_click(cx.listener(move |this, _event, _window, cx| {
                                                    this.prompt_confirm_consumed(meal_id, cx);
                                                })),
                                        )
                                    }),
                            )
                    })),
            )
            // Schedule Meal Modal Dialog
            .child(
                Dialog::new("schedule-meal-modal", "Schedule New Meal")
                    .subtitle("Add a meal to your schedule and scale target people headcount")
                    .is_open(self.show_schedule_modal)
                    .on_close(cx.listener(|this, _event, _window, cx| {
                        this.show_schedule_modal = false;
                        cx.notify();
                    }))
                    .child(
                        Select::new("select-meal-type", meal_type_options)
                            .label("Meal Source Category")
                            .selected_id(Some(self.form_meal_type.clone()))
                            .on_select(cx.listener(|this, opt: &SelectOption, _window, cx| {
                                this.form_meal_type = opt.id.clone();
                                cx.notify();
                            })),
                    )
                    .when(self.form_meal_type == "recipe", |this| {
                        this.child(
                            Select::new("select-schedule-recipe", recipe_options.clone())
                                .label("Recipe")
                                .selected_id(self.form_recipe_id.map(|id| id.0.to_string()))
                                .on_select(cx.listener(|this, opt: &SelectOption, _window, cx| {
                                    if let Ok(uuid) = uuid::Uuid::from_str(&opt.id) {
                                        this.form_recipe_id = Some(RecipeId(uuid));
                                    }
                                    cx.notify();
                                })),
                        )
                    })
                    .when(self.form_meal_type == "pre_planned", |this| {
                        this.child(
                            FormInput::new("input-pre-planned-name")
                                .label("Pre-Planned Combo Meal Name")
                                .placeholder("e.g. Sunday Family Roast Dinner")
                                .value(self.form_pre_planned_name.clone()),
                        )
                    })
                    .when(self.form_meal_type == "restaurant", |this| {
                        this.child(
                            FormInput::new("input-restaurant-name")
                                .label("Restaurant / Takeout Name")
                                .placeholder("e.g. Olive Garden, Thai Bistro")
                                .value(self.form_restaurant_name.clone()),
                        )
                        .child(
                            NumberInput::new("input-restaurant-cost", self.form_restaurant_cost)
                                .label("Estimated Dining Cost ($)")
                                .step(dec!(5.00))
                                .unit("$")
                                .on_increment(cx.listener(|this, val, _window, cx| {
                                    this.form_restaurant_cost = *val;
                                    cx.notify();
                                }))
                                .on_decrement(cx.listener(|this, val, _window, cx| {
                                    this.form_restaurant_cost = *val;
                                    cx.notify();
                                })),
                        )
                    })
                    .child(
                        NumberInput::new("input-meal-people", Decimal::from(self.form_people))
                            .label("People Count (Headcount)")
                            .step(dec!(1))
                            .on_increment(cx.listener(|this, val: &Decimal, _window, cx| {
                                this.form_people = val.to_string().parse().unwrap_or(1);
                                cx.notify();
                            }))
                            .on_decrement(cx.listener(|this, val: &Decimal, _window, cx| {
                                this.form_people = val.to_string().parse().unwrap_or(1);
                                cx.notify();
                            })),
                    )
                    .child(
                        DatePicker::new("dp-meal-date", self.form_date)
                            .label("Scheduled Date")
                            .on_change(cx.listener(|this, date, _window, cx| {
                                this.form_date = *date;
                                cx.notify();
                            })),
                    )
                    .footer_action(
                        Button::new("btn-cancel-schedule")
                            .secondary()
                            .label("Cancel")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.show_schedule_modal = false;
                                cx.notify();
                            })),
                    )
                    .footer_action(
                        Button::new("btn-save-schedule")
                            .primary()
                            .label("Schedule Meal")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.save_scheduled_meal(cx);
                            })),
                    ),
            )
            // Confirm Consumed Modal Dialog
            .child(
                Dialog::new("confirm-consumed-modal", "Confirm Meal Consumed")
                    .subtitle("Mark this scheduled meal as consumed and update pantry inventory")
                    .is_open(self.show_consumed_modal)
                    .on_close(cx.listener(|this, _event, _window, cx| {
                        this.show_consumed_modal = false;
                        cx.notify();
                    }))
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().foreground)
                            .child("Are you sure you want to mark this meal as consumed? This will update the meal status and deduct required ingredients from your Pantry."),
                    )
                    .footer_action(
                        Button::new("btn-cancel-consumed")
                            .secondary()
                            .label("Cancel")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.show_consumed_modal = false;
                                cx.notify();
                            })),
                    )
                    .footer_action(
                        Button::new("btn-confirm-consumed-action")
                            .primary()
                            .label("Confirm & Deduct Stock")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.execute_confirm_consumed(cx);
                            })),
                    ),
            )
    }
}
