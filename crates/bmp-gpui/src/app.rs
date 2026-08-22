use crate::components::TitleBar;
use crate::views::*;
use bmp_services::AppServices;
use bmp_storage::Storage;
use gpui::*;
use gpui_component::{
    ActiveTheme, Root, Theme,
    status_bar::StatusBar,
    tab::{Tab, TabBar},
};

#[derive(Copy, Clone)]
pub enum TabOption {
    Items = 0,
    Recipes = 1,
    Schedule = 2,
    ShoppingList = 3,
    Pantry = 4,
    Analytics = 5,
    Settings = 6,
    UiPrimitives = 7,
}

impl TabOption {
    fn name(&self) -> String {
        match self {
            TabOption::Items => "Items & Packages".into(),
            TabOption::Recipes => "Recipes".into(),
            TabOption::Schedule => "Schedule".into(),
            TabOption::ShoppingList => "Shopping List".into(),
            TabOption::Pantry => "Pantry".into(),
            TabOption::Analytics => "Analytics".into(),
            TabOption::Settings => "Settings & Controls".into(),
            TabOption::UiPrimitives => "UI Primitives".into(),
        }
    }
    fn from_index(index: usize) -> Option<TabOption> {
        match index {
            0 => Some(TabOption::Items),
            1 => Some(TabOption::Recipes),
            2 => Some(TabOption::Schedule),
            3 => Some(TabOption::ShoppingList),
            4 => Some(TabOption::Pantry),
            5 => Some(TabOption::Analytics),
            6 => Some(TabOption::Settings),
            7 => Some(TabOption::UiPrimitives),
            _ => None,
        }
    }

    fn index(self) -> usize {
        self as usize
    }
}

pub struct BudgetMealPlannerApp {
    _appearance_subscription: Subscription,
    pub _services: AppServices,
    pub db_path_str: String,
    pub active_tab: TabOption,
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
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
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
        let subscription = window.observe_window_appearance(|window, cx| {
            Theme::sync_system_appearance(Some(window), cx);
            cx.refresh_windows();
        });

        let existing_items = storage.get_all_items().unwrap_or_default();
        let existing_recipes = storage.get_all_recipes().unwrap_or_default();
        if existing_items.is_empty() && existing_recipes.is_empty() {
            let _ = bmp_common_ingredients::seed_common_data_if_not_exists(&storage);
        }

        let items_view = cx.new(|cx| ItemsView::new(services.clone(), window, cx));
        let recipes_view = cx.new(|cx| RecipesView::new(services.clone(), window, cx));
        let schedule_view = cx.new(|_| ScheduleView::new(services.clone()));
        let shopping_view = cx.new(|cx| ShoppingView::new(services.clone(), window, cx));
        let pantry_view = cx.new(|cx| PantryView::new(services.clone(), window, cx));
        let analytics_view = cx.new(|cx| AnalyticsView::new(services.clone(), window, cx));
        let settings_view = cx.new(|cx| SettingsView::new(cx, services.clone()));
        let showcase_view = cx.new(|cx| ComponentShowcaseView::new(window, cx));

        Self {
            _appearance_subscription: subscription,
            _services: services,
            db_path_str,
            active_tab: TabOption::Items,
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

    pub fn set_tab(&mut self, tab: TabOption, cx: &mut Context<Self>) {
        self.active_tab = tab;
        cx.notify();
    }
}

impl Render for BudgetMealPlannerApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active_tab = self.active_tab;
        let tab_name = active_tab.name();

        let dialog_layer = Root::render_dialog_layer(window, cx);
        let sheet_layer = Root::render_sheet_layer(window, cx);
        let notification_layer = Root::render_notification_layer(window, cx);

        div()
            .relative()
            .flex()
            .flex_col()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            // Custom Title Bar
            .child(TitleBar::new("Budget Meal Planner v5", "Database Ready"))
            // Navigation TabBar
            .child(
                div()
                    .px_4()
                    .py_2()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(
                        TabBar::new("app-main-tabs")
                            .selected_index(active_tab.index())
                            .on_click(cx.listener(|this, tab_idx, _window, cx| {
                                this.set_tab(
                                    TabOption::from_index(*tab_idx)
                                        .unwrap_or(TabOption::UiPrimitives),
                                    cx,
                                );
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
            .child(div().flex_1().child(match active_tab {
                TabOption::Items => self.items_view.clone().into_any_element(),
                TabOption::Recipes => self.recipes_view.clone().into_any_element(),
                TabOption::Schedule => self.schedule_view.clone().into_any_element(),
                TabOption::ShoppingList => self.shopping_view.clone().into_any_element(),
                TabOption::Pantry => self.pantry_view.clone().into_any_element(),
                TabOption::Analytics => self.analytics_view.clone().into_any_element(),
                TabOption::Settings => self.settings_view.clone().into_any_element(),
                TabOption::UiPrimitives => self.showcase_view.clone().into_any_element(),
            }))
            .child(
                StatusBar::new()
                    .left(format!("● SQLite Storage: {}", self.db_path_str))
                    .child(format!("Active Context: {}", tab_name))
                    .right("v5.0 Local | File-Backed Default"),
            )
            .children(dialog_layer)
            .children(sheet_layer)
            .children(notification_layer)
    }
}
