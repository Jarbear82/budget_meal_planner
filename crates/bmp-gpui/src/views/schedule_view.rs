use crate::components::*;
use bmp_domain::*;
use bmp_services::AppServices;
use chrono::{DateTime, Local, NaiveDate, Utc};
use gpui::prelude::*;
use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::WindowExt;
use gpui_component::alert::Alert;
use gpui_component::badge::Badge;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::dialog::{DialogDescription, DialogFooter, DialogHeader, DialogTitle};
use gpui_component::scroll::ScrollableElement;
use gpui_component::tag::Tag;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::str::FromStr;

pub struct ScheduleView {
    pub services: AppServices,
    pub status_msg: String,

    pub cached_scheduled_meals: Vec<ScheduledMeal>,
    pub cached_recipes: Vec<Recipe>,

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
        let mut view = Self {
            services,
            status_msg: "Schedule manager ready".to_string(),

            cached_scheduled_meals: Vec::new(),
            cached_recipes: Vec::new(),

            form_meal_type: "recipe".to_string(),
            form_recipe_id: None,
            form_pre_planned_name: String::new(),
            form_restaurant_name: String::new(),
            form_restaurant_cost: dec!(25.00),
            form_people: 2,
            form_date: Local::now().date_naive(),

            target_meal_id: None,
        };
        view.reload_data();
        view
    }

    pub fn reload_data(&mut self) {
        self.cached_scheduled_meals = self
            .services
            .meals
            .list_scheduled_meals()
            .unwrap_or_default();
        self.cached_recipes = self.services.recipes.list_recipes().unwrap_or_default();

        if let Ok(pending) = self
            .services
            .notification
            .check_pending_notifications(Utc::now())
            && !pending.is_empty() {
                self.status_msg = format!(
                    "🔔 Alert: You have {} scheduled meal(s) past 30-min window awaiting consumption confirmation!",
                    pending.len()
                );
            }
    }

    pub fn check_pending_alerts(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let pending = self
            .services
            .notification
            .check_pending_notifications(Utc::now())
            .unwrap_or_default();
        if let Some(first_due) = pending.first() {
            self.prompt_confirm_consumed(first_due.id, window, cx);
        } else {
            self.status_msg = "No pending meal consumption notifications due.".to_string();
            cx.notify();
        }
    }

    pub fn open_schedule_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let recipes = &self.cached_recipes;
        self.form_recipe_id = recipes.first().map(|r| r.id);
        self.form_meal_type = "recipe".to_string();
        self.form_people = 2;
        self.form_date = Local::now().date_naive();

        let pre_planned_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("e.g. Sunday Family Roast Dinner")
                .default_value(self.form_pre_planned_name.clone())
        });
        let restaurant_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("e.g. Olive Garden, Thai Bistro")
                .default_value(self.form_restaurant_name.clone())
        });

        let recipe_options: Vec<SelectOption> = recipes
            .iter()
            .map(|r| SelectOption::new(r.id.0.to_string(), r.name.clone()))
            .collect();

        let meal_type_options = vec![
            SelectOption::new("recipe", "Recipe Meal"),
            SelectOption::new("pre_planned", "Pre-Planned Combo Meal"),
            SelectOption::new("restaurant", "Restaurant Dining / Takeout"),
        ];

        let meal_type_select = cx.new(|cx| {
            SelectState::new(
                meal_type_options,
                Some(IndexPath::default().row(0)),
                window,
                cx,
            )
        });
        let recipe_select = cx.new(|cx| {
            SelectState::new(
                recipe_options,
                if recipes.is_empty() {
                    None
                } else {
                    Some(IndexPath::default().row(0))
                },
                window,
                cx,
            )
            .searchable(true)
        });
        let date_picker_state = cx.new(|cx| {
            let mut picker = DatePickerState::new(window, cx);
            picker.set_date(self.form_date, window, cx);
            picker
        });

        let view = cx.entity().clone();
        window.open_dialog(cx, move |dialog, _, _| {
            let view = view.clone();
            let p_in = pre_planned_input.clone();
            let r_in = restaurant_input.clone();
            let mt_in = meal_type_select.clone();
            let rec_in = recipe_select.clone();
            let dp_in = date_picker_state.clone();

            dialog.w(px(500.)).content(move |content, _, cx| {
                let view_read = view.read(cx);
                let form_restaurant_cost = view_read.form_restaurant_cost;
                let form_people = view_read.form_people;

                let selected_meal_type = mt_in
                    .read(cx)
                    .selected_value()
                    .cloned()
                    .unwrap_or_else(|| "recipe".to_string());

                let v_cost = view.clone();
                let v_people = view.clone();
                let v_save = view.clone();
                let p_save = p_in.clone();
                let r_save = r_in.clone();
                let mt_save = mt_in.clone();
                let rec_save = rec_in.clone();
                let dp_save = dp_in.clone();

                content
                    .child(
                        DialogHeader::new()
                            .child(DialogTitle::new().child("Schedule New Meal"))
                            .child(DialogDescription::new().child(
                                "Add a meal to your schedule and scale target people headcount",
                            )),
                    )
                    .child(
                        div()
                            .py_4()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .child(select_field("Meal Source Category", Select::new(&mt_in)))
                            .when(selected_meal_type == "recipe", |this| {
                                this.child(select_field("Recipe", Select::new(&rec_in)))
                            })
                            .when(selected_meal_type == "pre_planned", |this| {
                                this.child(form_field(
                                    "Pre-Planned Combo Meal Name",
                                    Input::new(&p_in),
                                ))
                            })
                            .when(selected_meal_type == "restaurant", |this| {
                                this.child(form_field(
                                    "Restaurant / Takeout Name",
                                    Input::new(&r_in),
                                ))
                                .child(
                                    NumberInput::new("input-restaurant-cost", form_restaurant_cost)
                                        .label("Estimated Dining Cost ($)")
                                        .step(dec!(5.00))
                                        .unit("$")
                                        .on_increment({
                                            let v = v_cost.clone();
                                            move |val, _window, cx| {
                                                v.update(cx, |this, cx| {
                                                    this.form_restaurant_cost = *val;
                                                    cx.notify();
                                                });
                                            }
                                        })
                                        .on_decrement({
                                            let v = v_cost.clone();
                                            move |val, _window, cx| {
                                                v.update(cx, |this, cx| {
                                                    this.form_restaurant_cost = *val;
                                                    cx.notify();
                                                });
                                            }
                                        }),
                                )
                            })
                            .child(
                                NumberInput::new("input-meal-people", Decimal::from(form_people))
                                    .label("People Count (Headcount)")
                                    .step(dec!(1))
                                    .on_increment({
                                        let v = v_people.clone();
                                        move |val: &Decimal, _window, cx| {
                                            let count = val.to_string().parse().unwrap_or(1);
                                            v.update(cx, |this, cx| {
                                                this.form_people = count;
                                                cx.notify();
                                            });
                                        }
                                    })
                                    .on_decrement({
                                        let v = v_people.clone();
                                        move |val: &Decimal, _window, cx| {
                                            let count = val.to_string().parse().unwrap_or(1);
                                            v.update(cx, |this, cx| {
                                                this.form_people = count;
                                                cx.notify();
                                            });
                                        }
                                    }),
                            )
                            .child(date_picker_field("Scheduled Date", DatePicker::new(&dp_in))),
                    )
                    .child(
                        DialogFooter::new()
                            .child(
                                Button::new("btn-cancel-schedule")
                                    .secondary()
                                    .label("Cancel")
                                    .on_click(|_, window, cx| {
                                        window.close_dialog(cx);
                                    }),
                            )
                            .child(
                                Button::new("btn-save-schedule")
                                    .primary()
                                    .label("Schedule Meal")
                                    .on_click(move |_, window, cx| {
                                        let pre_name = p_save.read(cx).value().to_string();
                                        let rest_name = r_save.read(cx).value().to_string();
                                        let m_type = mt_save
                                            .read(cx)
                                            .selected_value()
                                            .cloned()
                                            .unwrap_or_else(|| "recipe".to_string());
                                        let r_uuid = rec_save
                                            .read(cx)
                                            .selected_value()
                                            .and_then(|s| uuid::Uuid::from_str(s).ok())
                                            .map(RecipeId);
                                        let sel_date = match dp_save.read(cx).date() {
                                            Date::Single(Some(d)) => d,
                                            Date::Range(Some(d), _) => d,
                                            _ => Local::now().date_naive(),
                                        };
                                        v_save.update(cx, |this, cx| {
                                            this.form_meal_type = m_type;
                                            this.form_recipe_id = r_uuid;
                                            this.form_date = sel_date;
                                            this.form_pre_planned_name = pre_name;
                                            this.form_restaurant_name = rest_name;
                                            this.save_scheduled_meal(cx);
                                        });
                                        window.close_dialog(cx);
                                    }),
                            ),
                    )
            })
        });
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
                let ppm = match self
                    .services
                    .meals
                    .create_pre_planned_meal(&name, Vec::new())
                {
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

        match self
            .services
            .meals
            .schedule_meal(source, dt, self.form_people)
        {
            Ok(meal) => {
                self.status_msg = format!(
                    "Scheduled meal for {} people on {}",
                    meal.people, self.form_date
                );
            }
            Err(e) => {
                self.status_msg = format!("Error scheduling meal: {}", e);
            }
        }
        self.reload_data();
        cx.notify();
    }

    pub fn prompt_confirm_consumed(
        &mut self,
        meal_id: ScheduledMealId,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.target_meal_id = Some(meal_id);
        let view = cx.entity().clone();
        window.open_dialog(cx, move |dialog, _, _| {
            let view_confirm = view.clone();
            dialog
                .w(px(500.))
                .content(move |content, _, _| {
                    let v_confirm = view_confirm.clone();
                    content
                        .child(
                            DialogHeader::new()
                                .child(DialogTitle::new().child("Confirm Meal Consumed"))
                                .child(DialogDescription::new().child("Mark this scheduled meal as consumed and update pantry inventory")),
                        )
                        .child(
                            div()
                                .py_4()
                                .text_sm()
                                .child("Are you sure you want to mark this meal as consumed? This will update the meal status and deduct required ingredients from your Pantry."),
                        )
                        .child(
                            DialogFooter::new()
                                .child(
                                    Button::new("btn-cancel-consumed")
                                        .secondary()
                                        .label("Cancel")
                                        .on_click(|_, window, cx| {
                                            window.close_dialog(cx);
                                        }),
                                )
                                .child(
                                    Button::new("btn-confirm-consumed-action")
                                        .primary()
                                        .label("Confirm & Deduct Stock")
                                        .on_click(move |_, window, cx| {
                                            v_confirm.update(cx, |this, cx| {
                                                this.execute_confirm_consumed(cx);
                                            });
                                            window.close_dialog(cx);
                                        }),
                                ),
                        )
                })
        });
    }

    pub fn execute_confirm_consumed(&mut self, cx: &mut Context<Self>) {
        let meal_id = match self.target_meal_id {
            Some(id) => id,
            None => return,
        };

        match self
            .services
            .meals
            .confirm_meal_consumed(meal_id, Utc::now())
        {
            Ok(_) => {
                self.status_msg = "Confirmed meal consumed! Updated schedule & pantry.".to_string();
            }
            Err(e) => {
                self.status_msg = format!("Error confirming consumed: {}", e);
            }
        }
        self.reload_data();
        cx.notify();
    }
}

impl Render for ScheduleView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let scheduled_meals = self.cached_scheduled_meals.clone();
        let recipes = self.cached_recipes.clone();

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
                                Button::new("btn-check-pending-alerts")
                                    .secondary()
                                    .label("🔔 Pending Alerts")
                                    .on_click(cx.listener(|this, _event, window, cx| {
                                        this.check_pending_alerts(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("btn-schedule-meal")
                                    .primary()
                                    .label("+ Schedule Meal")
                                    .on_click(cx.listener(|this, _event, window, cx| {
                                        this.open_schedule_modal(window, cx);
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
                                                .on_click(cx.listener(move |this, _event, window, cx| {
                                                    this.prompt_confirm_consumed(meal_id, window, cx);
                                                })),
                                        )
                                    }),
                            )
                    })),
            )
    }
}
