use crate::components::TitleBar;
use crate::views::*;
use bmp_common_ingredients::seed_common_ingredients;
use bmp_services::AppServices;
use bmp_storage::Storage;
use gpui::*;
use gpui_component::tab::{Tab, TabBar};
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
            // Navigation TabBar using gpui_component::tab::TabBar & Tab
            .child(
                div()
                    .px_4()
                    .py_2()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(
                        TabBar::new("app-main-tabs")
                            .selected_index(active_tab)
                            .on_click(cx.listener(|this, idx, _window, cx| {
                                this.set_tab(*idx, cx);
                            }))
                            .child(Tab::new().label("Items & Packages"))
                            .child(Tab::new().label("Recipes"))
                            .child(Tab::new().label("Schedule"))
                            .child(Tab::new().label("Shopping List"))
                            .child(Tab::new().label("Pantry"))
                            .child(Tab::new().label("Analytics")),
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
