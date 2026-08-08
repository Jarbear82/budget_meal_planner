use bmp_domain::*;
use bmp_services::AppServices;
use gpui::*;
use gpui_component::ActiveTheme;
use gpui_component::button::{Button, ButtonVariants};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

pub struct MakeRecipeModal {
    pub services: AppServices,
    pub recipe: Option<Recipe>,
    pub batches: Decimal,
    pub selected_yield: Option<ItemId>,
    pub status: String,
}

impl MakeRecipeModal {
    pub fn new(services: AppServices, recipe: Option<Recipe>) -> Self {
        let first_yield = recipe.as_ref().and_then(|r| r.yields.first().map(|y| y.0));
        Self {
            services,
            recipe,
            batches: dec!(1.0),
            selected_yield: first_yield,
            status: "Ready to cook recipe".to_string(),
        }
    }

    pub fn execute_cook(&mut self) -> Result<String, String> {
        let recipe = match &self.recipe {
            Some(r) => r,
            None => return Err("No recipe selected".to_string()),
        };

        let items_list = self.services.items.list_items()?;
        let items_map: HashMap<ItemId, Item> = items_list.into_iter().map(|i| (i.id, i)).collect();

        let mut config = MakeRecipeConfig::default();
        config.batches = self.batches;
        config.selected_yield_item = self.selected_yield;

        let recipes_list = self.services.recipes.list_recipes()?;
        let recipes_map: HashMap<RecipeId, Recipe> =
            recipes_list.into_iter().map(|r| (r.id, r)).collect();

        let execution = evaluate_make_recipe_full(recipe, &config, &items_map, &recipes_map)
            .map_err(|e| e.to_string())?;

        // Deduct consumed ingredients & add produced yield items to Pantry
        for (item_id, qty) in execution.ingredients_to_consume {
            let _ = self
                .services
                .pantry
                .consume_pantry_item(item_id, qty.amount, qty.unit);
        }
        for (yield_id, qty) in execution.yields_produced {
            let _ = self
                .services
                .pantry
                .add_pantry_entry(yield_id, qty.amount, qty.unit, None);
        }

        self.status = format!("Recipe '{}' executed! Pantry updated.", recipe.name);
        Ok(self.status.clone())
    }

    pub fn adjust_batches(&mut self, delta: Decimal, cx: &mut Context<Self>) {
        if self.batches + delta > dec!(0.0) {
            self.batches += delta;
            cx.notify();
        }
    }
}

impl Render for MakeRecipeModal {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let recipe_name = self
            .recipe
            .as_ref()
            .map(|r| r.name.as_str())
            .unwrap_or("Select a Recipe");

        div()
            .flex()
            .flex_col()
            .gap_4()
            .p_6()
            .bg(cx.theme().background)
            .border_1()
            .border_color(cx.theme().border)
            .rounded_lg()
            .child(
                div()
                    .text_xl()
                    .font_weight(FontWeight::BOLD)
                    .text_color(cx.theme().foreground)
                    .child(format!("Make Recipe: {}", recipe_name)),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("Batch Scaling Factor:"),
                    )
                    .child(
                        Button::new("btn-dec-batch")
                            .secondary()
                            .label("- 0.5x")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.adjust_batches(dec!(-0.5), cx);
                            })),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1()
                            .bg(cx.theme().muted)
                            .rounded_md()
                            .text_sm()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x10b981))
                            .child(format!("{}x", self.batches)),
                    )
                    .child(
                        Button::new("btn-inc-batch")
                            .secondary()
                            .label("+ 0.5x")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.adjust_batches(dec!(0.5), cx);
                            })),
                    ),
            )
            .child(
                Button::new("btn-execute-cook")
                    .primary()
                    .label("Execute Cook Recipe & Update Pantry")
                    .on_click(cx.listener(|this, _event, _window, cx| {
                        let _ = this.execute_cook();
                        cx.notify();
                    })),
            )
            .child(
                div()
                    .p_3()
                    .bg(cx.theme().muted)
                    .rounded_md()
                    .text_xs()
                    .text_color(rgb(0x38bdf8))
                    .child(self.status.clone()),
            )
    }
}
