use crate::components::TitleBar;
use crate::views::*;
use bmp_common_ingredients::seed_common_ingredients;
use bmp_services::AppServices;
use bmp_storage::Storage;
use gpui::*;
use gpui_component::ActiveTheme;

pub struct BudgetMealPlannerApp {
    pub _services: AppServices,
    pub active_tab: usize,
    pub status_msg: String,

    pub items_view: Entity<ItemsView>,
    pub recipes_view: Entity<RecipesView>,
    pub schedule_view: Entity<ScheduleView>,
    pub shopping_view: Entity<ShoppingView>,
    pub pantry_view: Entity<PantryView>,
    pub analytics_view: Entity<AnalyticsView>,
}

impl BudgetMealPlannerApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let storage = Storage::in_memory().expect("Failed to initialize database");
        let services = AppServices::new(storage.clone());
        let _ = seed_common_ingredients(&storage);

        let items_view = cx.new(|_| ItemsView::new(services.clone()));
        let recipes_view = cx.new(|_| RecipesView::new(services.clone()));
        let schedule_view = cx.new(|_| ScheduleView::new(services.clone()));
        let shopping_view = cx.new(|_| ShoppingView::new(services.clone()));
        let pantry_view = cx.new(|_| PantryView::new(services.clone()));
        let analytics_view = cx.new(|_| AnalyticsView::new(services.clone()));

        Self {
            _services: services,
            active_tab: 0,
            status_msg: "Welcome to Budget Meal Planner v5!".to_string(),
            items_view,
            recipes_view,
            schedule_view,
            shopping_view,
            pantry_view,
            analytics_view,
        }
    }

    pub fn set_tab(&mut self, tab_idx: usize, cx: &mut Context<Self>) {
        self.active_tab = tab_idx;
        cx.notify();
    }
}

impl Render for BudgetMealPlannerApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active_tab = self.active_tab;

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            // Custom Title Bar
            .child(cx.new(|_| TitleBar::new("Budget Meal Planner v5", "Database Ready")))
            // Navigation Tab Bar
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap_1()
                    .px_4()
                    .py_2()
                    .bg(cx.theme().background)
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(
                        div()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .bg(if active_tab == 0 { cx.theme().accent } else { cx.theme().muted })
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| this.set_tab(0, cx)))
                            .child("Items & Packages"),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .bg(if active_tab == 1 { cx.theme().accent } else { cx.theme().muted })
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| this.set_tab(1, cx)))
                            .child("Recipes"),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .bg(if active_tab == 2 { cx.theme().accent } else { cx.theme().muted })
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| this.set_tab(2, cx)))
                            .child("Schedule"),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .bg(if active_tab == 3 { cx.theme().accent } else { cx.theme().muted })
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| this.set_tab(3, cx)))
                            .child("Shopping List"),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .bg(if active_tab == 4 { cx.theme().accent } else { cx.theme().muted })
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| this.set_tab(4, cx)))
                            .child("Pantry"),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .bg(if active_tab == 5 { cx.theme().accent } else { cx.theme().muted })
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .on_mouse_down(MouseButton::Left, cx.listener(|this, _, _window, cx| this.set_tab(5, cx)))
                            .child("Analytics"),
                    ),
            )
            // Active Tab Content View
            .child(
                div().flex_1().child(match active_tab {
                    0 => self.items_view.clone().into_any_element(),
                    1 => self.recipes_view.clone().into_any_element(),
                    2 => self.schedule_view.clone().into_any_element(),
                    3 => self.shopping_view.clone().into_any_element(),
                    4 => self.pantry_view.clone().into_any_element(),
                    _ => self.analytics_view.clone().into_any_element(),
                }),
            )
    }
}
