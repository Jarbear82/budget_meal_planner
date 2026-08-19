use gpui::prelude::FluentBuilder;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::ActiveTheme;
use bmp_domain::Recipe;

/// Helper to render a compact recipe summary row
pub fn render_recipe_summary_row(
    recipe: &Recipe,
    is_selected: bool,
    on_select: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    cx: &App,
) -> impl IntoElement {
    div()
        .flex()
        .items_center()
        .justify_between()
        .p_2()
        .rounded_lg()
        .map(|this| {
            if is_selected {
                this.bg(cx.theme().accent)
            } else {
                this.bg(cx.theme().muted)
            }
        })
        .child(
            div()
                .flex()
                .flex_col()
                .child(div().text_sm().font_weight(FontWeight::BOLD).child(recipe.name.clone()))
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("Servings: {} | {} ingredients", recipe.servings, recipe.ingredients.len())),
                ),
        )
        .child(
            Button::new(format!("btn-sel-{}", recipe.id.0))
                .ghost()
                .label("Select")
                .on_click(on_select),
        )
}
