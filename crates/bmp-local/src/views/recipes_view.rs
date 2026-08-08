use bmp_domain::*;
use bmp_services::AppServices;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::ActiveTheme;
use rust_decimal_macros::dec;

pub struct RecipesView {
    pub services: AppServices,
    pub _selected_recipe: Option<RecipeId>,
    pub status_msg: String,
}

impl RecipesView {
    pub fn new(services: AppServices) -> Self {
        Self {
            services,
            _selected_recipe: None,
            status_msg: "Ready".to_string(),
        }
    }

    pub fn create_sample_recipe(&mut self, cx: &mut Context<Self>) {
        let count = self.services.recipes.list_recipes().map(|l| l.len()).unwrap_or(0);
        let name = format!("New Recipe Template {}", count + 1);
        let mut recipe = Recipe::new(&name, dec!(4));
        recipe.instructions = "Mix ingredients, bake at 350F for 30 minutes, and serve warm.".to_string();

        match self.services.recipes.save_recipe(recipe) {
            Ok(saved) => {
                self.status_msg = format!("Saved recipe: {}", saved.name);
            }
            Err(e) => {
                self.status_msg = format!("Error: {}", e);
            }
        }
        cx.notify();
    }
}

impl Render for RecipesView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let recipes = self.services.recipes.list_recipes().unwrap_or_default();

        div()
            .flex()
            .flex_col()
            .gap_4()
            .size_full()
            .p_6()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(cx.theme().foreground)
                            .child("Recipes & Sub-Recipe Builder"),
                    )
                    .child(
                        Button::new("btn-new-recipe")
                            .primary()
                            .label("+ New Recipe")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.create_sample_recipe(cx);
                            })),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_6()
                    .child(
                        div()
                            .w_1_3()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .p_4()
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_lg()
                            .child(div().text_xs().font_weight(FontWeight::BOLD).text_color(cx.theme().muted_foreground).child("SAVED RECIPES"))
                            .children(recipes.iter().map(|r| {
                                div()
                                    .p_3()
                                    .rounded_md()
                                    .bg(cx.theme().muted)
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(cx.theme().foreground)
                                    .child(format!("{} ({} servings)", r.name, r.servings))
                            })),
                    )
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .p_6()
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_lg()
                            .child(div().text_lg().font_weight(FontWeight::BOLD).text_color(cx.theme().foreground).child("Recipe Details & Ingredient Edges"))
                            .child(div().text_xs().text_color(cx.theme().muted_foreground).child(format!("Status: {}", self.status_msg))),
                    ),
            )
    }
}
