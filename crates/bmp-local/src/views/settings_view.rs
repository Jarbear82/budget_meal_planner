use bmp_services::AppServices;
use gpui::prelude::*;
use gpui::*;
use gpui_component::alert::Alert;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::group_box::GroupBoxVariant;
use gpui_component::label::Label;
use gpui_component::setting::{
    NumberFieldOptions, SettingField, SettingGroup, SettingItem, SettingPage, Settings,
};
use gpui_component::{
    h_flex, v_flex, ActiveTheme, Disableable, Icon, IconName, Sizable, Size, Theme, ThemeMode,
};

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub default_servings: f64,
    pub auto_deduct_pantry: bool,
    pub preferred_package_lock: bool,
    pub default_purchase_mode: SharedString,
    pub meal_reminder_enabled: bool,
    pub reminder_delay_mins: f64,
    pub density: SharedString,
    pub resettable: bool,
    pub disabled: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            default_servings: 4.0,
            auto_deduct_pantry: true,
            preferred_package_lock: true,
            default_purchase_mode: "Buy Finished".into(),
            meal_reminder_enabled: true,
            reminder_delay_mins: 30.0,
            density: "Comfortable".into(),
            resettable: true,
            disabled: false,
        }
    }
}

pub struct SettingsView {
    pub services: AppServices,
    pub focus_handle: FocusHandle,
    pub group_variant: GroupBoxVariant,
    pub size: Size,
    pub settings: AppSettings,
    pub status_msg: String,
}

impl SettingsView {
    pub fn new(cx: &mut Context<Self>, services: AppServices) -> Self {
        Self {
            services,
            focus_handle: cx.focus_handle(),
            group_variant: GroupBoxVariant::Outline,
            size: Size::Medium,
            settings: AppSettings::default(),
            status_msg: "Settings & System Controls Ready".to_string(),
        }
    }

    pub fn seed_data(&mut self, cx: &mut Context<Self>) {
        match bmp_common_ingredients::seed_common_data_if_not_exists(&self.services.storage) {
            Ok((items_added, recipes_added)) => {
                if items_added == 0 && recipes_added == 0 {
                    self.status_msg =
                        "Database already contains all sample ingredients and recipes.".to_string();
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

    fn setting_pages(&self, _: &mut Window, cx: &mut Context<Self>) -> Vec<SettingPage> {
        let view = cx.entity().clone();
        let default_settings = AppSettings::default();
        let resettable = self.settings.resettable;
        let disabled = self.settings.disabled;

        vec![
            // Page 1: General & Meal Planning
            SettingPage::new("General & Planning")
                .resettable(resettable)
                .default_open(true)
                .icon(Icon::new(IconName::Settings2))
                .groups(vec![
                    SettingGroup::new().title("Meal Planning Defaults").items(vec![
                        SettingItem::new(
                            "Default Servings",
                            SettingField::number_input(
                                NumberFieldOptions {
                                    min: 1.0,
                                    max: 20.0,
                                    step: 1.0,
                                    ..Default::default()
                                },
                                {
                                    let view = view.clone();
                                    move |cx: &App| view.read(cx).settings.default_servings
                                },
                                {
                                    let view = view.clone();
                                    move |val: f64, cx: &mut App| {
                                        view.update(cx, |this, cx| {
                                            this.settings.default_servings = val;
                                            this.status_msg = format!("Default servings set to {}", val);
                                            cx.notify();
                                        });
                                    }
                                },
                            )
                            .default_value(default_settings.default_servings),
                        )
                        .description("Default base servings count when drafting new recipes or meal schedules.")
                        .disabled(disabled)
                        .keywords(["servings", "portions", "scale"]),
                        SettingItem::new(
                            "Auto-Deduct Pantry",
                            SettingField::switch(
                                {
                                    let view = view.clone();
                                    move |cx: &App| view.read(cx).settings.auto_deduct_pantry
                                },
                                {
                                    let view = view.clone();
                                    move |val: bool, cx: &mut App| {
                                        view.update(cx, |this, cx| {
                                            this.settings.auto_deduct_pantry = val;
                                            this.status_msg = format!("Auto-deduct pantry set to {}", val);
                                            cx.notify();
                                        });
                                    }
                                },
                            )
                            .default_value(default_settings.auto_deduct_pantry),
                        )
                        .description("Automatically decrement pantry inventory when scheduled meals are marked as consumed.")
                        .disabled(disabled)
                        .keywords(["pantry", "inventory", "auto-decrement"]),
                        SettingItem::new(
                            "Preferred Package Lock",
                            SettingField::switch(
                                {
                                    let view = view.clone();
                                    move |cx: &App| view.read(cx).settings.preferred_package_lock
                                },
                                {
                                    let view = view.clone();
                                    move |val: bool, cx: &mut App| {
                                        view.update(cx, |this, cx| {
                                            this.settings.preferred_package_lock = val;
                                            this.status_msg = format!("Preferred package lock set to {}", val);
                                            cx.notify();
                                        });
                                    }
                                },
                            )
                            .default_value(default_settings.preferred_package_lock),
                        )
                        .description("Lock shopping list calculations to pinned preferred store packages when available.")
                        .disabled(disabled)
                        .keywords(["shopping", "store", "packages", "price"]),
                        SettingItem::new(
                            "Default Purchase Mode",
                            SettingField::dropdown(
                                vec![
                                    ("Buy Finished".into(), "Buy Finished Package".into()),
                                    ("Prefer Make".into(), "Prefer Make / Expand".into()),
                                    ("Ask Every Time".into(), "Ask Every Time".into()),
                                ],
                                {
                                    let view = view.clone();
                                    move |cx: &App| view.read(cx).settings.default_purchase_mode.clone()
                                },
                                {
                                    let view = view.clone();
                                    move |val: SharedString, cx: &mut App| {
                                        view.update(cx, |this, cx| {
                                            this.settings.default_purchase_mode = val.clone();
                                            this.status_msg = format!("Default purchase mode set to {}", val);
                                            cx.notify();
                                        });
                                    }
                                },
                            )
                            .default_value(default_settings.default_purchase_mode),
                        )
                        .description("Initial buy vs make behavior assigned to new ingredients and sub-recipes.")
                        .disabled(disabled)
                        .keywords(["purchase", "mode", "buy", "make"]),
                    ]),
                    SettingGroup::new().title("Meal Notifications").items(vec![
                        SettingItem::new(
                            "Consumption Reminders",
                            SettingField::switch(
                                {
                                    let view = view.clone();
                                    move |cx: &App| view.read(cx).settings.meal_reminder_enabled
                                },
                                {
                                    let view = view.clone();
                                    move |val: bool, cx: &mut App| {
                                        view.update(cx, |this, cx| {
                                            this.settings.meal_reminder_enabled = val;
                                            this.status_msg = format!("Meal reminders set to {}", val);
                                            cx.notify();
                                        });
                                    }
                                },
                            )
                            .default_value(default_settings.meal_reminder_enabled),
                        )
                        .description("Prompt verification notifications following scheduled meal times.")
                        .disabled(disabled)
                        .keywords(["notifications", "reminders", "verify"]),
                        SettingItem::new(
                            "Reminder Delay (Minutes)",
                            SettingField::number_input(
                                NumberFieldOptions {
                                    min: 5.0,
                                    max: 120.0,
                                    step: 5.0,
                                    ..Default::default()
                                },
                                {
                                    let view = view.clone();
                                    move |cx: &App| view.read(cx).settings.reminder_delay_mins
                                },
                                {
                                    let view = view.clone();
                                    move |val: f64, cx: &mut App| {
                                        view.update(cx, |this, cx| {
                                            this.settings.reminder_delay_mins = val;
                                            this.status_msg = format!("Reminder delay set to {} mins", val);
                                            cx.notify();
                                        });
                                    }
                                },
                            )
                            .default_value(default_settings.reminder_delay_mins),
                        )
                        .description("Elapsed minutes after scheduled meal before firing consumption prompt.")
                        .disabled(disabled)
                        .keywords(["delay", "timer", "minutes"]),
                    ]),
                ]),
            // Page 2: Appearance & Styling
            SettingPage::new("Appearance & Theme")
                .resettable(resettable)
                .icon(Icon::new(IconName::Palette))
                .groups(vec![
                    SettingGroup::new().title("Theme & Color Mode").items(vec![
                        SettingItem::new(
                            "Dark Mode",
                            SettingField::switch(
                                |cx: &App| cx.theme().mode.is_dark(),
                                |val: bool, cx: &mut App| {
                                    let mode = if val {
                                        ThemeMode::Dark
                                    } else {
                                        ThemeMode::Light
                                    };
                                    Theme::global_mut(cx).mode = mode;
                                    Theme::change(mode, None, cx);
                                },
                            )
                            .default_value(false),
                        )
                        .description("Switch between Light and Dark interface palettes.")
                        .disabled(disabled)
                        .keywords(["dark", "light", "theme", "color"]),
                        SettingItem::new(
                            "Group Box Variant",
                            SettingField::dropdown(
                                vec![
                                    (GroupBoxVariant::Outline.as_str().into(), "Outline".into()),
                                    (GroupBoxVariant::Normal.as_str().into(), "Normal".into()),
                                    (GroupBoxVariant::Fill.as_str().into(), "Fill".into()),
                                ],
                                {
                                    let view = view.clone();
                                    move |cx: &App| {
                                        SharedString::from(view.read(cx).group_variant.as_str().to_string())
                                    }
                                },
                                {
                                    let view = view.clone();
                                    move |val: SharedString, cx: &mut App| {
                                        view.update(cx, |this, cx| {
                                            this.group_variant = GroupBoxVariant::from_str(val.as_str());
                                            cx.notify();
                                        });
                                    }
                                },
                            )
                            .default_value(GroupBoxVariant::Outline.as_str().to_string()),
                        )
                        .description("Visual style for settings cards and container boundaries.")
                        .disabled(disabled)
                        .keywords(["variant", "border", "card"]),
                        SettingItem::new(
                            "Component Size",
                            SettingField::dropdown(
                                vec![
                                    (Size::Small.as_str().into(), "Small".into()),
                                    (Size::Medium.as_str().into(), "Medium".into()),
                                    (Size::Large.as_str().into(), "Large".into()),
                                ],
                                {
                                    let view = view.clone();
                                    move |cx: &App| {
                                        SharedString::from(view.read(cx).size.as_str().to_string())
                                    }
                                },
                                {
                                    let view = view.clone();
                                    move |val: SharedString, cx: &mut App| {
                                        view.update(cx, |this, cx| {
                                            this.size = Size::from_str(val.as_str());
                                            cx.notify();
                                        });
                                    }
                                },
                            )
                            .default_value(Size::Medium.as_str().to_string()),
                        )
                        .description("Overall padding and scale for controls across the settings panel.")
                        .disabled(disabled)
                        .keywords(["size", "scale", "padding"]),
                    ]),
                    SettingGroup::new().title("Interface Density").item(
                        SettingItem::new(
                            "Layout Density",
                            SettingField::render({
                                let view = view.clone();
                                move |options, _window, cx| {
                                    let current = view.read(cx).settings.density.clone();
                                    let v1 = view.clone();
                                    let v2 = view.clone();
                                    h_flex()
                                        .gap_2()
                                        .child(
                                            Button::new("density-comfortable")
                                                .label("Comfortable")
                                                .with_size(options.size)
                                                .map(|b| if current == "Comfortable" { b.primary() } else { b.outline() })
                                                .on_click(move |_, _, cx| {
                                                    v1.update(cx, |this, cx| {
                                                        this.settings.density = "Comfortable".into();
                                                        this.status_msg = "Layout density set to Comfortable".to_string();
                                                        cx.notify();
                                                    });
                                                }),
                                        )
                                        .child(
                                            Button::new("density-compact")
                                                .label("Compact")
                                                .with_size(options.size)
                                                .map(|b| if current == "Compact" { b.primary() } else { b.outline() })
                                                .on_click(move |_, _, cx| {
                                                    v2.update(cx, |this, cx| {
                                                        this.settings.density = "Compact".into();
                                                        this.status_msg = "Layout density set to Compact".to_string();
                                                        cx.notify();
                                                    });
                                                }),
                                        )
                                }
                            }),
                        )
                        .description("Adjust row spacing and table item density.")
                        .disabled(disabled)
                        .keywords(["density", "spacing", "compact", "comfortable"]),
                    ),
                ]),
            // Page 3: Data & Storage
            SettingPage::new("Data & Storage")
                .resettable(resettable)
                .icon(Icon::new(IconName::Folder))
                .groups(vec![
                    SettingGroup::new().title("Sample Data Management").items(vec![
                        SettingItem::new(
                            "Seed Sample Data",
                            SettingField::render({
                                let view = view.clone();
                                move |options, _window, _cx| {
                                    let v = view.clone();
                                    Button::new("btn-seed-sample-data")
                                        .primary()
                                        .label("🌱 Seed Sample Ingredients & Recipes")
                                        .with_size(options.size)
                                        .on_click(move |_, _, cx| {
                                            v.update(cx, |this, cx| {
                                                this.seed_data(cx);
                                            });
                                        })
                                }
                            }),
                        )
                        .description("Populate missing domain ingredients and starter recipes into your database without creating duplicates.")
                        .keywords(["seed", "sample", "ingredients", "recipes", "starter"]),
                    ]),
                    SettingGroup::new().title("Local SQLite Engine").items(vec![
                        SettingItem::render(|_options, _, cx| {
                            v_flex()
                                .gap_2()
                                .w_full()
                                .child(
                                    div()
                                        .font_weight(FontWeight::BOLD)
                                        .text_sm()
                                        .text_color(cx.theme().foreground)
                                        .child("100% Offline & Private"),
                                )
                                .child(
                                    div()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("Budget Meal Planner v5 persists all user items, recipes, schedules, and receipts strictly inside your local SQLite database file with zero cloud telemetry or account requirements."),
                                )
                                .into_any_element()
                        }),
                    ]),
                ]),
            // Page 4: About & System
            SettingPage::new("About & System")
                .resettable(resettable)
                .icon(Icon::new(IconName::Info))
                .groups(vec![
                    SettingGroup::new().item(
                        SettingItem::render(|_options, _, cx| {
                            v_flex()
                                .gap_2()
                                .w_full()
                                .items_center()
                                .justify_center()
                                .py_4()
                                .child(Icon::new(IconName::GalleryVerticalEnd).size_16())
                                .child(
                                    div()
                                        .text_lg()
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(cx.theme().foreground)
                                        .child("Budget Meal Planner v5"),
                                )
                                .child(
                                    Label::new("Pure Rust, Local-First Meal Planner with GPUI Desktop Interface")
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground),
                                )
                                .into_any_element()
                        }),
                    ),
                    SettingGroup::new().title("Specifications & Architecture").items(vec![
                        SettingItem::render(|_options, _, cx| {
                            h_flex()
                                .w_full()
                                .justify_between()
                                .items_center()
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .child(div().text_sm().font_weight(FontWeight::BOLD).child("SRS v5 Specification Compliant"))
                                        .child(div().text_xs().text_color(cx.theme().muted_foreground).child("Verified with 29 automated test suites across domain, storage, and services.")),
                                )
                                .child(
                                    Button::new("btn-srs-status")
                                        .outline()
                                        .label("Compliant")
                                        .disabled(true),
                                )
                                .into_any_element()
                        }),
                    ]),
                ]),
        ]
    }
}

impl Focusable for SettingsView {
    fn focus_handle(&self, _: &gpui::App) -> gpui::FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for SettingsView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .p_6()
            .gap_4()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .child(
                        v_flex()
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(cx.theme().foreground)
                                    .child("Settings & Preferences"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Configure meal planning defaults, appearance, theme palettes, and data seeding"),
                            ),
                    )
                    .child(Alert::new("settings-status-alert", format!("Status: {}", self.status_msg))),
            )
            .child(
                div()
                    .flex_1()
                    .w_full()
                    .child(
                        Settings::new("app-settings")
                            .with_size(self.size)
                            .with_group_variant(self.group_variant)
                            .pages(self.setting_pages(window, cx)),
                    ),
            )
    }
}
