use crate::components::TitleBar;
use crate::views::*;
use bmp_common_ingredients::seed_common_ingredients;
use bmp_services::AppServices;
use bmp_storage::Storage;
use gpui::*;
use gpui_component::status_bar::StatusBar;
use gpui_component::tab::{Tab, TabBar};
use gpui_component::ActiveTheme;

pub struct BudgetMealPlannerApp {
    pub _services: AppServices,
    pub db_path_str: String,
    pub active_tab: usize,
    pub status_msg: String,

    pub items_view: Entity<ItemsView>,
    pub recipes_view: Entity<RecipesView>,
    pub schedule_view: Entity<ScheduleView>,
    pub shopping_view: Entity<ShoppingView>,
    pub pantry_view: Entity<PantryView>,
    pub analytics_view: Entity<AnalyticsView>,
    pub settings_view: Entity<SettingsView>,
    pub showcase_view: Entity<ComponentShowcaseView>,
}

impl BudgetMealPlannerApp {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let (storage, db_path_str) = match Storage::open_default() {
            Ok(s) => {
                let path_display = Storage::default_db_path()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| "File-Backed DB".to_string());
                (s, path_display)
            }
            Err(_) => (
                Storage::in_memory().expect("Failed to initialize database"),
                "In-Memory Fallback".to_string(),
            ),
        };

        let services = AppServices::new(storage.clone());
        let _ = seed_common_ingredients(&storage);

        let items_view = cx.new(|_| ItemsView::new(services.clone()));
        let recipes_view = cx.new(|_| RecipesView::new(services.clone()));
        let schedule_view = cx.new(|_| ScheduleView::new(services.clone()));
        let shopping_view = cx.new(|_| ShoppingView::new(services.clone()));
        let pantry_view = cx.new(|_| PantryView::new(services.clone()));
        let analytics_view = cx.new(|_| AnalyticsView::new(services.clone()));
        let settings_view = cx.new(|cx| SettingsView::new(cx, services.clone()));
        let showcase_view = cx.new(|_| ComponentShowcaseView::new());

        Self {
            _services: services,
            db_path_str,
            active_tab: 0,
            status_msg: "Welcome to Budget Meal Planner v5!".to_string(),
            items_view,
            recipes_view,
            schedule_view,
            shopping_view,
            pantry_view,
            analytics_view,
            settings_view,
            showcase_view,
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
        let tab_name = match active_tab {
            0 => "Items & Packages",
            1 => "Recipes",
            2 => "Schedule",
            3 => "Shopping List",
            4 => "Pantry",
            5 => "Analytics",
            6 => "Settings & Controls",
            _ => "UI Primitives",
        };

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            // Custom Title Bar
            .child(TitleBar::new("Budget Meal Planner v5", "Database Ready"))
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
                            .child(Tab::new().label("Analytics"))
                            .child(Tab::new().label("Settings & Controls"))
                            .child(Tab::new().label("UI Primitives")),
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
                    5 => self.analytics_view.clone().into_any_element(),
                    6 => self.settings_view.clone().into_any_element(),
                    _ => self.showcase_view.clone().into_any_element(),
                }),
            )
            // Native gpui_component::status_bar::StatusBar
            .child(
                StatusBar::new()
                    .left(format!("● SQLite Storage: {}", self.db_path_str))
                    .child(format!("Active Context: {}", tab_name))
                    .right("v5.0 Local | File-Backed Default"),
            )
    }
}
