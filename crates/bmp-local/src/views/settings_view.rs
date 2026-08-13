use bmp_services::AppServices;
use gpui::prelude::*;
use gpui::*;
use gpui_component::alert::Alert;
use gpui_component::avatar::Avatar;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::kbd::Kbd;
use gpui_component::pagination::Pagination;
use gpui_component::progress::Progress;
use gpui_component::rating::Rating;
use gpui_component::switch::Switch;
use gpui_component::ActiveTheme;

pub struct SettingsView {
    pub services: AppServices,
    pub status_msg: String,

    pub user_name: String,
    pub favorite_rating: usize,
    pub current_page: usize,
    pub progress_val: f32,
    pub auto_deduct_pantry: bool,
    pub preferred_package_lock: bool,
}

impl SettingsView {
    pub fn new(_cx: &mut Context<Self>, services: AppServices) -> Self {
        Self {
            services,
            status_msg: "Settings & System Controls Ready".to_string(),

            user_name: "Local Chef".to_string(),
            favorite_rating: 5,
            current_page: 1,
            progress_val: 75.0,
            auto_deduct_pantry: true,
            preferred_package_lock: true,
        }
    }

    pub fn seed_data(&mut self, cx: &mut Context<Self>) {
        match bmp_common_ingredients::seed_common_data_if_not_exists(&self.services.storage) {
            Ok((items_added, recipes_added)) => {
                if items_added == 0 && recipes_added == 0 {
                    self.status_msg = "Database already contains sample ingredients and recipes.".to_string();
                } else {
                    self.status_msg = format!(
                        "Successfully seeded {} new ingredients and {} new recipes!",
                        items_added, recipes_added
                    );
                }
            }
            Err(e) => {
                self.status_msg = format!("Seeding Error: {}", e);
            }
        }
        cx.notify();
    }
}

impl Render for SettingsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let keystroke_ctrl_n = Keystroke::parse("ctrl-n").unwrap();
        let keystroke_ctrl_f = Keystroke::parse("ctrl-f").unwrap();

        div()
            .flex()
            .flex_col()
            .gap_6()
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
                            .items_center()
                            .gap_3()
                            .child(Avatar::new().name(self.user_name.clone()))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .child(
                                        div()
                                            .text_2xl()
                                            .font_weight(FontWeight::BOLD)
                                            .text_color(cx.theme().foreground)
                                            .child("Preferences & Extended Controls"),
                                    )
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child("Configure local profile, sample data seeding, ratings, and keyboard shortcuts"),
                                    ),
                            ),
                    ),
            )
            // Status Bar
            .child(
                div()
                    .p_3()
                    .bg(cx.theme().muted)
                    .rounded_lg()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child(format!("Status: {}", self.status_msg)),
            )
            // Components Showcase Grid
            .child(
                div()
                    .grid()
                    .grid_cols(2)
                    .gap_6()
                    // Card 1: User Profile & Rating Controls
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .p_5()
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_xl()
                            .child(div().text_sm().font_weight(FontWeight::BOLD).child("Profile & Recipe Rating"))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_3()
                                    .child(Avatar::new().name("Chef Jarom"))
                                    .child(
                                        div()
                                            .flex()
                                            .flex_col()
                                            .child(div().text_sm().font_weight(FontWeight::BOLD).child("Jarom (Local Workspace)"))
                                            .child(div().text_xs().text_color(cx.theme().muted_foreground).child("Default Recipe Evaluator")),
                                    ),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_3()
                                    .child(div().text_xs().text_color(cx.theme().muted_foreground).child("Default Recipe Rating:"))
                                    .child(
                                        Rating::new("setting-rating")
                                            .value(self.favorite_rating)
                                            .on_click(cx.listener(|this, val: &usize, _window, cx| {
                                                this.favorite_rating = *val;
                                                this.status_msg = format!("Set default recipe rating to {} stars", val);
                                                cx.notify();
                                            })),
                                    ),
                            ),
                    )
                    // Card 2: Sample Data Seeding & Database Controls
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .p_5()
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_xl()
                            .child(div().text_sm().font_weight(FontWeight::BOLD).child("Sample Data Seeding"))
                            .child(div().text_xs().text_color(cx.theme().muted_foreground).child("Populate missing common ingredients and starter recipes into your database."))
                            .child(
                                Button::new("btn-seed-common-data")
                                    .primary()
                                    .label("🌱 Seed Sample Ingredients & Recipes")
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.seed_data(cx);
                                    })),
                            ),
                    )
                    // Card 3: Progress & Pagination Controls
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .p_5()
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_xl()
                            .child(div().text_sm().font_weight(FontWeight::BOLD).child("Operation Progress & List Pagination"))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(div().text_xs().text_color(cx.theme().muted_foreground).child(format!("Pantry Audit Completion ({}%)", self.progress_val)))
                                    .child(Progress::new("setting-progress").value(self.progress_val)),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(div().text_xs().text_color(cx.theme().muted_foreground).child("Catalog Pagination:"))
                                    .child(
                                        Pagination::new("setting-pagination")
                                            .current_page(self.current_page)
                                            .total_pages(5)
                                            .on_click(cx.listener(|this, page: &usize, _window, cx| {
                                                this.current_page = *page;
                                                this.status_msg = format!("Navigated to page {}", page);
                                                cx.notify();
                                            })),
                                    ),
                            ),
                    )
                    // Card 4: Keyboard Shortcuts (Kbd) & Settings Group
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .p_5()
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_xl()
                            .child(div().text_sm().font_weight(FontWeight::BOLD).child("Keyboard Shortcuts (Kbd)"))
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(div().text_xs().text_color(cx.theme().muted_foreground).child("Create New Item / Recipe"))
                                    .child(Kbd::new(keystroke_ctrl_n)),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(div().text_xs().text_color(cx.theme().muted_foreground).child("Focus Search Input"))
                                    .child(Kbd::new(keystroke_ctrl_f)),
                            ),
                    )
                    // Card 5: Interactive Switch Toggles & Privacy
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .p_5()
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_xl()
                            .child(div().text_sm().font_weight(FontWeight::BOLD).child("Preferences & Switches"))
                            .child(
                                Switch::new("switch-auto-deduct")
                                    .label("Auto-deduct pantry on meal consumption")
                                    .checked(self.auto_deduct_pantry)
                                    .on_click(cx.listener(|this, checked: &bool, _window, cx| {
                                        this.auto_deduct_pantry = *checked;
                                        this.status_msg = format!("Auto-deduct pantry set to {}", checked);
                                        cx.notify();
                                    })),
                            )
                            .child(
                                Switch::new("switch-package-lock")
                                    .label("Lock shopping list to preferred store packages")
                                    .checked(self.preferred_package_lock)
                                    .on_click(cx.listener(|this, checked: &bool, _window, cx| {
                                        this.preferred_package_lock = *checked;
                                        this.status_msg = format!("Preferred package lock set to {}", checked);
                                        cx.notify();
                                    })),
                            )
                            .child(Alert::new("privacy-alert", "Local database active. All data remains 100% on-device.")),
                    ),
            )
    }
}
