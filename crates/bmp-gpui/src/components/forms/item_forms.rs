use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::ActiveTheme;
use bmp_domain::DietaryFlag;

/// Helper building a styled form section header
pub fn form_section_title(title: &str, cx: &App) -> impl IntoElement {
    div()
        .text_sm()
        .font_weight(FontWeight::BOLD)
        .text_color(cx.theme().foreground)
        .child(title.to_string())
}

/// Helper for dietary flag toggle chips
pub fn render_dietary_flag_chip(
    flag: DietaryFlag,
    is_selected: bool,
    on_toggle: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    _cx: &App,
) -> impl IntoElement {
    Button::new(format!("flag-chip-{}", flag.as_str()))
        .label(flag.as_str())
        .map(|this| {
            if is_selected {
                this.primary()
            } else {
                this.ghost()
            }
        })
        .on_click(on_toggle)
}
