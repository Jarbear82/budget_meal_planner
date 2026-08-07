use bmp_domain::*;
use bmp_services::AppServices;
use gpui::*;

pub struct RecipesView {
    pub services: AppServices,
    pub selected_recipe: Option<RecipeId>,
}

impl RecipesView {
    pub fn new(services: AppServices) -> Self {
        Self {
            services,
            selected_recipe: None,
        }
    }
}

impl Render for RecipesView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let recipes = self.services.recipes.list_recipes().unwrap_or_default();

        div()
            .flex()
            .flex_col()
            .gap_4()
            .size_full()
            .p_6()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xf4f4f5))
                            .child("Recipes & Sub-Recipe Builder"),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .bg(rgb(0x10b981))
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x18181b))
                            .child("+ New Recipe"),
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
                            .bg(rgb(0x18181b))
                            .border_1()
                            .border_color(rgb(0x27272a))
                            .rounded_lg()
                            .child(div().text_xs().font_weight(FontWeight::BOLD).text_color(rgb(0xa1a1aa)).child("SAVED RECIPES"))
                            .children(recipes.iter().map(|r| {
                                div()
                                    .p_3()
                                    .rounded_md()
                                    .bg(rgb(0x27272a))
                                    .text_sm()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0xf4f4f5))
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
                            .bg(rgb(0x18181b))
                            .border_1()
                            .border_color(rgb(0x27272a))
                            .rounded_lg()
                            .child(div().text_lg().font_weight(FontWeight::BOLD).child("Recipe Details & Ingredient Edges"))
                            .child(div().text_xs().text_color(rgb(0xa1a1aa)).child("Select or create a recipe to view yield variants, sub-recipe nesting, and cycle detection indicators.")),
                    ),
            )
    }
}
