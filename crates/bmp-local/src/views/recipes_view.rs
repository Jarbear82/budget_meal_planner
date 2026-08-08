use crate::components::*;
use bmp_domain::*;
use bmp_services::AppServices;
use gpui::prelude::*;
use gpui::*;
use gpui_component::alert::Alert;
use gpui_component::badge::Badge;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::scroll::ScrollableElement;
use gpui_component::tag::Tag;
use gpui_component::ActiveTheme;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;
use std::str::FromStr;

pub struct RecipesView {
    pub services: AppServices,
    pub search_query: String,
    pub status_msg: String,

    pub selected_recipe_id: Option<RecipeId>,

    // Modals
    pub show_recipe_modal: bool,
    pub show_make_modal: bool,

    // Recipe Editor Form State
    pub editing_recipe_id: Option<RecipeId>,
    pub recipe_form_name: String,
    pub recipe_form_servings: Decimal,
    pub recipe_form_instructions: String,
    pub recipe_form_yields: Vec<(ItemId, Quantity)>,
    pub recipe_form_ingredients: Vec<IngredientEdge>,

    // Ingredient addition form sub-state
    pub ing_target_is_recipe: bool,
    pub ing_target_item_id: Option<ItemId>,
    pub ing_target_recipe_id: Option<RecipeId>,
    pub ing_amount: Decimal,
    pub ing_unit: Unit,
    pub ing_required: bool,
    pub ing_substitute_id: Option<ItemId>,

    // Yield addition form sub-state
    pub yield_item_id: Option<ItemId>,
    pub yield_amount: Decimal,
    pub yield_unit: Unit,

    // Make Recipe Form State
    pub make_batches: Decimal,
    pub make_selected_yield: Option<ItemId>,
    pub make_status: String,
}

impl RecipesView {
    pub fn new(services: AppServices) -> Self {
        Self {
            services,
            search_query: String::new(),
            status_msg: "Recipes manager ready".to_string(),

            selected_recipe_id: None,
            show_recipe_modal: false,
            show_make_modal: false,

            editing_recipe_id: None,
            recipe_form_name: String::new(),
            recipe_form_servings: dec!(4),
            recipe_form_instructions: String::new(),
            recipe_form_yields: Vec::new(),
            recipe_form_ingredients: Vec::new(),

            ing_target_is_recipe: false,
            ing_target_item_id: None,
            ing_target_recipe_id: None,
            ing_amount: dec!(100),
            ing_unit: Unit::Gram,
            ing_required: true,
            ing_substitute_id: None,

            yield_item_id: None,
            yield_amount: dec!(1),
            yield_unit: Unit::Each,

            make_batches: dec!(1.0),
            make_selected_yield: None,
            make_status: String::new(),
        }
    }

    pub fn open_create_recipe_modal(&mut self, cx: &mut Context<Self>) {
        let items = self.services.items.list_items().unwrap_or_default();
        let first_item = items.first().map(|i| i.id);

        self.editing_recipe_id = None;
        self.recipe_form_name = String::new();
        self.recipe_form_servings = dec!(4);
        self.recipe_form_instructions = "Mix ingredients thoroughly and cook as directed.".to_string();
        self.recipe_form_yields = Vec::new();
        self.recipe_form_ingredients = Vec::new();

        self.ing_target_is_recipe = false;
        self.ing_target_item_id = first_item;
        self.ing_target_recipe_id = None;
        self.ing_amount = dec!(100);
        self.ing_unit = Unit::Gram;
        self.ing_required = true;
        self.ing_substitute_id = None;

        self.yield_item_id = first_item;
        self.yield_amount = dec!(1);
        self.yield_unit = Unit::Each;

        self.show_recipe_modal = true;
        cx.notify();
    }

    pub fn open_edit_recipe_modal(&mut self, recipe: &Recipe, cx: &mut Context<Self>) {
        let items = self.services.items.list_items().unwrap_or_default();
        let first_item = items.first().map(|i| i.id);

        self.editing_recipe_id = Some(recipe.id);
        self.recipe_form_name = recipe.name.clone();
        self.recipe_form_servings = recipe.servings;
        self.recipe_form_instructions = recipe.instructions.clone();
        self.recipe_form_yields = recipe.yields.clone();
        self.recipe_form_ingredients = recipe.ingredients.clone();

        self.ing_target_is_recipe = false;
        self.ing_target_item_id = first_item;
        self.ing_target_recipe_id = None;
        self.ing_amount = dec!(100);
        self.ing_unit = Unit::Gram;
        self.ing_required = true;
        self.ing_substitute_id = None;

        self.yield_item_id = first_item;
        self.yield_amount = dec!(1);
        self.yield_unit = Unit::Each;

        self.show_recipe_modal = true;
        cx.notify();
    }

    pub fn add_ingredient_to_form(&mut self, cx: &mut Context<Self>) {
        let target = if self.ing_target_is_recipe {
            match self.ing_target_recipe_id {
                Some(rid) => ItemOrRecipeId::Recipe(rid),
                None => {
                    self.status_msg = "Error: Please select a target sub-recipe".to_string();
                    return;
                }
            }
        } else {
            match self.ing_target_item_id {
                Some(iid) => ItemOrRecipeId::Item(iid),
                None => {
                    self.status_msg = "Error: Please select a target item".to_string();
                    return;
                }
            }
        };

        let qty = match Quantity::new(self.ing_amount, self.ing_unit.clone()) {
            Ok(q) => q,
            Err(e) => {
                self.status_msg = format!("Error: {}", e);
                return;
            }
        };

        let edge = IngredientEdge {
            target,
            quantity: qty,
            required: self.ing_required,
            cycle_flag: false,
            per_recipe_substitute: self.ing_substitute_id,
        };

        self.recipe_form_ingredients.push(edge);
        self.status_msg = "Added ingredient edge".to_string();
        cx.notify();
    }

    pub fn remove_ingredient_from_form(&mut self, idx: usize, cx: &mut Context<Self>) {
        if idx < self.recipe_form_ingredients.len() {
            self.recipe_form_ingredients.remove(idx);
        }
        cx.notify();
    }

    pub fn add_yield_to_form(&mut self, cx: &mut Context<Self>) {
        let item_id = match self.yield_item_id {
            Some(id) => id,
            None => {
                self.status_msg = "Error: Please select a yield item".to_string();
                return;
            }
        };

        let qty = match Quantity::new(self.yield_amount, self.yield_unit.clone()) {
            Ok(q) => q,
            Err(e) => {
                self.status_msg = format!("Error: {}", e);
                return;
            }
        };

        self.recipe_form_yields.push((item_id, qty));
        self.status_msg = "Added yield definition".to_string();
        cx.notify();
    }

    pub fn remove_yield_from_form(&mut self, idx: usize, cx: &mut Context<Self>) {
        if idx < self.recipe_form_yields.len() {
            self.recipe_form_yields.remove(idx);
        }
        cx.notify();
    }

    pub fn save_recipe(&mut self, cx: &mut Context<Self>) {
        if self.recipe_form_name.trim().is_empty() {
            self.status_msg = "Error: Recipe name cannot be empty".to_string();
            return;
        }

        let mut recipe = if let Some(id) = self.editing_recipe_id {
            let mut r = Recipe::new(self.recipe_form_name.trim(), self.recipe_form_servings);
            r.id = id;
            r
        } else {
            Recipe::new(self.recipe_form_name.trim(), self.recipe_form_servings)
        };

        recipe.instructions = self.recipe_form_instructions.trim().to_string();
        recipe.yields = self.recipe_form_yields.clone();
        recipe.ingredients = self.recipe_form_ingredients.clone();

        match self.services.recipes.save_recipe(recipe) {
            Ok(saved) => {
                self.status_msg = format!("Saved recipe: {}", saved.name);
                self.selected_recipe_id = Some(saved.id);
                self.show_recipe_modal = false;
            }
            Err(e) => {
                self.status_msg = format!("Error saving recipe: {}", e);
            }
        }
        cx.notify();
    }

    pub fn delete_recipe(&mut self, recipe_id: RecipeId, cx: &mut Context<Self>) {
        match self.services.recipes.delete_recipe(recipe_id) {
            Ok(_) => {
                self.status_msg = "Deleted recipe successfully".to_string();
                if self.selected_recipe_id == Some(recipe_id) {
                    self.selected_recipe_id = None;
                }
            }
            Err(e) => {
                self.status_msg = format!("Error deleting recipe: {}", e);
            }
        }
        cx.notify();
    }

    pub fn open_make_recipe_modal(&mut self, recipe: &Recipe, cx: &mut Context<Self>) {
        self.selected_recipe_id = Some(recipe.id);
        self.make_batches = dec!(1.0);
        self.make_selected_yield = recipe.yields.first().map(|y| y.0);
        self.make_status = format!("Ready to batch cook '{}'", recipe.name);
        self.show_make_modal = true;
        cx.notify();
    }

    pub fn execute_make_recipe(&mut self, cx: &mut Context<Self>) {
        let recipe_id = match self.selected_recipe_id {
            Some(id) => id,
            None => return,
        };

        let recipes = self.services.recipes.list_recipes().unwrap_or_default();
        let recipe = match recipes.into_iter().find(|r| r.id == recipe_id) {
            Some(r) => r,
            None => return,
        };

        let items_list = self.services.items.list_items().unwrap_or_default();
        let items_map: HashMap<ItemId, Item> = items_list.into_iter().map(|i| (i.id, i)).collect();

        let recipes_list = self.services.recipes.list_recipes().unwrap_or_default();
        let recipes_map: HashMap<RecipeId, Recipe> = recipes_list.into_iter().map(|r| (r.id, r)).collect();

        let mut config = MakeRecipeConfig::default();
        config.batches = self.make_batches;
        config.selected_yield_item = self.make_selected_yield;

        match evaluate_make_recipe_full(&recipe, &config, &items_map, &recipes_map) {
            Ok(exec) => {
                // Deduct consumed ingredients & add produced yields to Pantry
                for (item_id, qty) in exec.ingredients_to_consume {
                    let _ = self.services.pantry.consume_pantry_item(item_id, qty.amount, qty.unit);
                }
                for (yield_id, qty) in exec.yields_produced {
                    let _ = self.services.pantry.add_pantry_entry(yield_id, qty.amount, qty.unit, None);
                }

                self.make_status = format!("Successfully produced {}x batch of '{}'! Pantry updated.", self.make_batches, recipe.name);
                self.status_msg = self.make_status.clone();
            }
            Err(e) => {
                self.make_status = format!("Execution Error: {}", e);
            }
        }
        cx.notify();
    }
}

impl Render for RecipesView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let recipes = self.services.recipes.list_recipes().unwrap_or_default();
        let items = self.services.items.list_items().unwrap_or_default();

        let filtered_recipes: Vec<Recipe> = recipes
            .into_iter()
            .filter(|r| {
                if self.search_query.trim().is_empty() {
                    true
                } else {
                    r.name.to_lowercase().contains(&self.search_query.to_lowercase())
                }
            })
            .collect();

        let selected_recipe = self
            .selected_recipe_id
            .and_then(|id| filtered_recipes.iter().find(|r| r.id == id).cloned());
        let has_selected_recipe = selected_recipe.is_some();

        let recipe_cost_estimate = if let Some(ref r) = selected_recipe {
            self.services.recipes.estimate_cost(r.id).ok()
        } else {
            None
        };

        let item_options: Vec<SelectOption> = items
            .iter()
            .map(|i| SelectOption::new(i.id.0.to_string(), i.name.clone()))
            .collect();

        let _recipe_options: Vec<SelectOption> = filtered_recipes
            .iter()
            .map(|r| SelectOption::new(r.id.0.to_string(), format!("Recipe: {}", r.name)))
            .collect();

        let unit_options = vec![
            SelectOption::new("Gram", "Gram (g)"),
            SelectOption::new("Kilogram", "Kilogram (kg)"),
            SelectOption::new("Milliliter", "Milliliter (ml)"),
            SelectOption::new("Liter", "Liter (L)"),
            SelectOption::new("Cup", "Cup"),
            SelectOption::new("Tablespoon", "Tablespoon (tbsp)"),
            SelectOption::new("Teaspoon", "Teaspoon (tsp)"),
            SelectOption::new("Ounce", "Ounce (oz)"),
            SelectOption::new("Pound", "Pound (lb)"),
            SelectOption::new("Each", "Each (count)"),
            SelectOption::new("Batch", "Batch"),
            SelectOption::new("Serving", "Serving"),
        ];

        div()
            .flex()
            .flex_col()
            .gap_4()
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
                            .flex_col()
                            .child(
                                div()
                                    .text_2xl()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(cx.theme().foreground)
                                    .child("Recipes & Sub-Recipe Builder"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Design multi-level recipes, yields, per-recipe substitutes, and evaluate live costings"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(Badge::new().child(format!("Saved Recipes: {}", filtered_recipes.len())))
                            .child(
                                Button::new("btn-new-recipe")
                                    .primary()
                                    .label("+ Create Recipe")
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.open_create_recipe_modal(cx);
                                    })),
                            ),
                    ),
            )
            // Status Banner
            .child(Alert::new("recipes-status-alert", format!("Status: {}", self.status_msg)))
            // Split Master-Detail View
            .child(
                div()
                    .flex()
                    .gap_4()
                    .flex_1()
                    // Recipes List Sidebar
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
                            .child(
                                div()
                                    .text_xs()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(cx.theme().muted_foreground)
                                    .child("RECIPE CATALOG"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .overflow_y_scrollbar()
                                    .children(filtered_recipes.into_iter().map(|recipe| {
                                        let recipe_id = recipe.id;
                                        let is_sel = self.selected_recipe_id == Some(recipe_id);
                                        let yields_count = recipe.yields.len();
                                        let ing_count = recipe.ingredients.len();
                                        let recipe_clone = recipe.clone();

                                        let has_cycle = recipe.ingredients.iter().any(|i| i.cycle_flag);

                                        let card_id = format!("recipe-card-{}", recipe_id);
                                        div()
                                            .id(ElementId::from(card_id))
                                            .flex()
                                            .flex_col()
                                            .gap_1()
                                            .p_3()
                                            .rounded_lg()
                                            .border_1()
                                            .border_color(cx.theme().border)
                                            .cursor_pointer()
                                            .bg(if is_sel {
                                                cx.theme().accent
                                            } else {
                                                cx.theme().background
                                            })
                                            .hover(|s| s.bg(cx.theme().muted))
                                            .on_click(cx.listener(move |this, _event, _window, cx| {
                                                this.selected_recipe_id = Some(recipe_id);
                                                cx.notify();
                                            }))
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .justify_between()
                                                    .child(
                                                        div()
                                                            .font_weight(FontWeight::BOLD)
                                                            .text_sm()
                                                            .child(recipe.name.clone()),
                                                    )
                                                    .when(has_cycle, |this| {
                                                        this.child(Tag::new().child("Cycle Aware"))
                                                    }),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .justify_between()
                                                    .text_xs()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .child(format!("{} Servings", recipe.servings.normalize()))
                                                    .child(format!("{} Ingredients, {} Yields", ing_count, yields_count)),
                                            )
                                            .child(
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .gap_1()
                                                    .mt_1()
                                                    .child({
                                                        let rec = recipe_clone.clone();
                                                        Button::new(format!("btn-make-{}", recipe_id))
                                                            .primary()
                                                            .label("Make Recipe")
                                                            .on_click(cx.listener(move |this, _event, _window, cx| {
                                                                this.open_make_recipe_modal(&rec, cx);
                                                            }))
                                                    })
                                                    .child({
                                                        let rec = recipe_clone.clone();
                                                        Button::new(format!("btn-edit-rec-{}", recipe_id))
                                                            .secondary()
                                                            .label("Edit")
                                                            .on_click(cx.listener(move |this, _event, _window, cx| {
                                                                this.open_edit_recipe_modal(&rec, cx);
                                                            }))
                                                    })
                                                    .child(
                                                        Button::new(format!("btn-del-rec-{}", recipe_id))
                                                            .ghost()
                                                            .label("🗑")
                                                            .on_click(cx.listener(move |this, _event, _window, cx| {
                                                                this.delete_recipe(recipe_id, cx);
                                                            })),
                                                    ),
                                            )
                                    })),
                            ),
                    )
                    // Recipe Details & Live Cost Calculator Pane
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
                            .when_some(selected_recipe, |this, recipe| {
                                let _recipe_id = recipe.id;
                                let recipe_clone = recipe.clone();

                                this.child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .justify_between()
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .child(
                                                    div()
                                                        .text_xl()
                                                        .font_weight(FontWeight::BOLD)
                                                        .child(recipe.name.clone()),
                                                )
                                                .child(
                                                    div()
                                                        .text_xs()
                                                        .text_color(cx.theme().muted_foreground)
                                                        .child(format!("Base Servings: {}", recipe.servings.normalize())),
                                                ),
                                        )
                                        .child(
                                            Button::new("btn-pane-make-recipe")
                                                .primary()
                                                .label("🍳 Cook / Produce Batch")
                                                .on_click(cx.listener(move |this, _event, _window, cx| {
                                                    this.open_make_recipe_modal(&recipe_clone, cx);
                                                })),
                                        ),
                                )
                                // Live Cost Analysis Summary Box
                                .child(
                                    div()
                                        .flex()
                                        .gap_4()
                                        .p_4()
                                        .bg(cx.theme().muted)
                                        .rounded_lg()
                                        .child(
                                            div()
                                                .flex_1()
                                                .flex()
                                                .flex_col()
                                                .child(div().text_xs().text_color(cx.theme().muted_foreground).child("Total Estimated Batch Cost"))
                                                .child(
                                                    div()
                                                        .text_lg()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(cx.theme().foreground)
                                                        .child(match &recipe_cost_estimate {
                                                            Some(c) => format!("${}", c.price_per_batch.normalize()),
                                                            None => "Cost pending (packages missing)".to_string(),
                                                        }),
                                                ),
                                        )
                                        .child(
                                            div()
                                                .flex_1()
                                                .flex()
                                                .flex_col()
                                                .child(div().text_xs().text_color(cx.theme().muted_foreground).child("Cost Per Serving"))
                                                .child(
                                                    div()
                                                        .text_lg()
                                                        .font_weight(FontWeight::BOLD)
                                                        .text_color(cx.theme().foreground)
                                                        .child(match &recipe_cost_estimate {
                                                            Some(c) => format!("${}", c.price_per_serving.normalize()),
                                                            None => "N/A".to_string(),
                                                        }),
                                                ),
                                        ),
                                )
                                // Ingredients Edges List
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_2()
                                        .child(div().text_sm().font_weight(FontWeight::BOLD).child("Ingredient Dependencies"))
                                        .children(recipe.ingredients.iter().map(|edge| {
                                            let target_name = match edge.target {
                                                ItemOrRecipeId::Item(iid) => items
                                                    .iter()
                                                    .find(|i| i.id == iid)
                                                    .map(|i| i.name.clone())
                                                    .unwrap_or_else(|| "Unknown Item".to_string()),
                                                ItemOrRecipeId::Recipe(rid) => {
                                                    format!("[Sub-Recipe] ID {:?}", rid)
                                                }
                                            };

                                            let req_tag = if edge.required { "Required" } else { "Optional" };
                                            let cycle_tag = if edge.cycle_flag { "Cycle Flagged" } else { "Normal" };

                                            div()
                                                .flex()
                                                .items_center()
                                                .justify_between()
                                                .py_2()
                                                .px_3()
                                                .border_1()
                                                .border_color(cx.theme().border)
                                                .rounded_md()
                                                .child(
                                                    div()
                                                        .flex()
                                                        .items_center()
                                                        .gap_2()
                                                        .child(div().font_weight(FontWeight::BOLD).text_sm().child(target_name))
                                                        .child(Tag::new().child(format!("{} {}", edge.quantity.amount, edge.quantity.unit))),
                                                )
                                                .child(
                                                    div()
                                                        .flex()
                                                        .items_center()
                                                        .gap_2()
                                                        .child(Badge::new().child(req_tag))
                                                        .when(edge.cycle_flag, |this| this.child(Tag::new().child(cycle_tag))),
                                                )
                                        })),
                                )
                                // Instructions Box
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .p_3()
                                        .bg(cx.theme().muted)
                                        .rounded_md()
                                        .child(div().text_xs().font_weight(FontWeight::BOLD).child("Preparation Instructions"))
                                        .child(div().text_xs().text_color(cx.theme().foreground).child(recipe.instructions.clone())),
                                )
                            })
                            .when(!has_selected_recipe, |this| {
                                this.child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .items_center()
                                        .justify_center()
                                        .h_full()
                                        .text_center()
                                        .text_xs()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("Select a recipe from the sidebar to inspect dependencies, evaluation costings, and execute cooking."),
                                )
                            }),
                    ),
            )
            // Recipe Creation / Edit Modal Dialog
            .child(
                Dialog::new(
                    "recipe-crud-modal",
                    if self.editing_recipe_id.is_some() {
                        "Edit Recipe Definition"
                    } else {
                        "Create New Recipe"
                    },
                )
                .subtitle("Configure recipe yields, ingredient edges, required/optional flags, and instructions")
                .is_open(self.show_recipe_modal)
                .on_close(cx.listener(|this, _event, _window, cx| {
                    this.show_recipe_modal = false;
                    cx.notify();
                }))
                .child(
                    FormInput::new("input-recipe-name")
                        .label("Recipe Name")
                        .placeholder("e.g. Homemade Bolognese Sauce")
                        .value(self.recipe_form_name.clone()),
                )
                .child(
                    NumberInput::new("input-recipe-servings", self.recipe_form_servings)
                        .label("Base Servings Count")
                        .step(dec!(1))
                        .on_increment(cx.listener(|this, val, _window, cx| {
                            this.recipe_form_servings = *val;
                            cx.notify();
                        }))
                        .on_decrement(cx.listener(|this, val, _window, cx| {
                            this.recipe_form_servings = *val;
                            cx.notify();
                        })),
                )
                .child(
                    FormInput::new("input-recipe-instructions")
                        .label("Preparation Instructions")
                        .placeholder("e.g. Sauté onions, brown meat, simmer for 45 mins")
                        .value(self.recipe_form_instructions.clone()),
                )
                // Sub-form for adding Ingredient Edges
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .p_3()
                        .bg(cx.theme().muted)
                        .rounded_md()
                        .child(div().text_xs().font_weight(FontWeight::BOLD).child("Add Ingredient Edge"))
                        .child(
                            Select::new("select-ing-item-target", item_options.clone())
                                .label("Target Ingredient Item")
                                .selected_id(self.ing_target_item_id.map(|id| id.0.to_string()))
                                .on_select(cx.listener(|this, opt: &SelectOption, _window, cx| {
                                    if let Ok(uuid) = uuid::Uuid::from_str(&opt.id) {
                                        this.ing_target_item_id = Some(ItemId(uuid));
                                    }
                                    cx.notify();
                                })),
                        )
                        .child(
                            div()
                                .flex()
                                .gap_2()
                                .child(
                                    NumberInput::new("input-ing-amount", self.ing_amount)
                                        .label("Amount")
                                        .step(dec!(10))
                                        .on_increment(cx.listener(|this, val, _window, cx| {
                                            this.ing_amount = *val;
                                            cx.notify();
                                        }))
                                        .on_decrement(cx.listener(|this, val, _window, cx| {
                                            this.ing_amount = *val;
                                            cx.notify();
                                        })),
                                )
                                .child(
                                    Select::new("select-ing-unit", unit_options.clone())
                                        .label("Unit")
                                        .selected_id(Some(format!("{:?}", self.ing_unit)))
                                        .on_select(cx.listener(|this, opt: &SelectOption, _window, cx| {
                                            this.ing_unit = match opt.id.as_str() {
                                                "Kilogram" => Unit::Kilogram,
                                                "Milliliter" => Unit::Milliliter,
                                                "Liter" => Unit::Liter,
                                                "Cup" => Unit::Cup,
                                                "Tablespoon" => Unit::Tablespoon,
                                                "Teaspoon" => Unit::Teaspoon,
                                                "Ounce" => Unit::Ounce,
                                                "Pound" => Unit::Pound,
                                                "Each" => Unit::Each,
                                                _ => Unit::Gram,
                                            };
                                            cx.notify();
                                        })),
                                ),
                        )
                        .child(
                            Checkbox::new("cb-ing-required")
                                .label("Required Ingredient (unchecked = optional)")
                                .checked(self.ing_required)
                                .on_click(cx.listener(|this, checked, _window, cx| {
                                    this.ing_required = *checked;
                                    cx.notify();
                                })),
                        )
                        .child(
                            Button::new("btn-add-ing-edge")
                                .secondary()
                                .label("+ Add Ingredient Edge")
                                .on_click(cx.listener(|this, _event, _window, cx| {
                                    this.add_ingredient_to_form(cx);
                                })),
                        ),
                )
                // List of current configured ingredient edges in form
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().text_xs().font_weight(FontWeight::BOLD).child(format!("Configured Ingredients ({})", self.recipe_form_ingredients.len())))
                        .children(self.recipe_form_ingredients.clone().into_iter().enumerate().map(|(idx, edge)| {
                            let target_str = match edge.target {
                                ItemOrRecipeId::Item(iid) => items
                                    .iter()
                                    .find(|i| i.id == iid)
                                    .map(|i| i.name.clone())
                                    .unwrap_or_else(|| "Item".to_string()),
                                ItemOrRecipeId::Recipe(_) => "Sub-Recipe".to_string(),
                            };

                            let edge_id = format!("form-edge-{}", idx);
                            div()
                                .id(ElementId::from(edge_id))
                                .flex()
                                .items_center()
                                .justify_between()
                                .p_2()
                                .border_1()
                                .border_color(cx.theme().border)
                                .rounded_md()
                                .text_xs()
                                .child(format!("{} - {} {}", target_str, edge.quantity.amount, edge.quantity.unit))
                                .child(
                                    Button::new(format!("btn-rem-edge-{}", idx))
                                        .ghost()
                                        .label("✕")
                                        .on_click(cx.listener(move |this, _event, _window, cx| {
                                            this.remove_ingredient_from_form(idx, cx);
                                        })),
                                )
                        })),
                )
                .footer_action(
                    Button::new("btn-cancel-recipe-modal")
                        .secondary()
                        .label("Cancel")
                        .on_click(cx.listener(|this, _event, _window, cx| {
                            this.show_recipe_modal = false;
                            cx.notify();
                        })),
                )
                .footer_action(
                    Button::new("btn-save-recipe-modal")
                        .primary()
                        .label("Save Recipe")
                        .on_click(cx.listener(|this, _event, _window, cx| {
                            this.save_recipe(cx);
                        })),
                ),
            )
            // Make Recipe Modal Dialog
            .child(
                Dialog::new("make-recipe-exec-modal", "Execute Make Recipe")
                    .subtitle("Batch cook recipe and update Pantry inventory")
                    .is_open(self.show_make_modal)
                    .on_close(cx.listener(|this, _event, _window, cx| {
                        this.show_make_modal = false;
                        cx.notify();
                    }))
                    .child(
                        NumberInput::new("input-make-batches", self.make_batches)
                            .label("Batch Multiplier / Scale")
                            .step(dec!(0.5))
                            .unit("x")
                            .on_increment(cx.listener(|this, val, _window, cx| {
                                this.make_batches = *val;
                                cx.notify();
                            }))
                            .on_decrement(cx.listener(|this, val, _window, cx| {
                                this.make_batches = *val;
                                cx.notify();
                            })),
                    )
                    .when(!self.make_status.is_empty(), |this| {
                        let status_str = self.make_status.clone();
                        this.child(
                            div()
                                .p_3()
                                .bg(cx.theme().accent)
                                .rounded_md()
                                .text_xs()
                                .font_weight(FontWeight::BOLD)
                                .child(status_str),
                        )
                    })
                    .footer_action(
                        Button::new("btn-cancel-make-modal")
                            .secondary()
                            .label("Close")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.show_make_modal = false;
                                cx.notify();
                            })),
                    )
                    .footer_action(
                        Button::new("btn-confirm-make-cook")
                            .primary()
                            .label("🍳 Produce Batch & Update Pantry")
                            .on_click(cx.listener(|this, _event, _window, cx| {
                                this.execute_make_recipe(cx);
                            })),
                    ),
            )
    }
}
