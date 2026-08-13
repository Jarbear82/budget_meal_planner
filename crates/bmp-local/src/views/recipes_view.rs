use crate::components::*;
use bmp_domain::*;
use bmp_services::AppServices;
use gpui::prelude::*;
use gpui::*;
use gpui_component::alert::Alert;
use gpui_component::badge::Badge;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::checkbox::Checkbox;
use gpui_component::dialog::{DialogDescription, DialogFooter, DialogHeader, DialogTitle};
use gpui_component::list::{List, ListDelegate, ListItem, ListState};
use gpui_component::tag::Tag;
use gpui_component::WindowExt;
use gpui_component::{ActiveTheme, IndexPath, Selectable};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecipeGroupMode {
    MealType,
    Alphabetical,
    Servings,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecipeSortMode {
    NameAsc,
    NameDesc,
    ServingsDesc,
}

pub struct RecipeSection {
    pub title: String,
    pub recipes: Vec<Recipe>,
}

#[derive(IntoElement)]
pub struct RecipeListItem {
    pub base: ListItem,
    pub recipe: Recipe,
    pub selected: bool,
    pub view: WeakEntity<RecipesView>,
}

impl Selectable for RecipeListItem {
    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    fn is_selected(&self) -> bool {
        self.selected
    }
}

impl RenderOnce for RecipeListItem {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let recipe_id = self.recipe.id;
        let recipe_clone = self.recipe.clone();
        let rec_make = recipe_clone.clone();
        let rec_edit = recipe_clone.clone();
        let yields_count = self.recipe.yields.len();
        let ing_count = self.recipe.ingredients.len();
        let has_cycle = self.recipe.ingredients.iter().any(|i| i.cycle_flag);

        let view_make = self.view.clone();
        let view_edit = self.view.clone();
        let view_delete = self.view.clone();

        self.base.py_2().px_2().rounded_md().child(
            div()
                .flex()
                .flex_col()
                .gap_1()
                .w_full()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_sm()
                                .text_color(cx.theme().foreground)
                                .child(self.recipe.name.clone()),
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
                        .child(format!("{} Servings", self.recipe.servings.normalize()))
                        .child(format!("{} Ingredients, {} Yields", ing_count, yields_count)),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_1()
                        .mt_1()
                        .child(
                            Button::new(format!("btn-make-{}", recipe_id))
                                .primary()
                                .label("Make Recipe")
                                .on_click(move |_, window, cx| {
                                    if let Some(view) = view_make.upgrade() {
                                        view.update(cx, |this, cx| {
                                            this.open_make_recipe_modal(&rec_make, window, cx);
                                        });
                                    }
                                }),
                        )
                        .child(
                            Button::new(format!("btn-edit-rec-{}", recipe_id))
                                .secondary()
                                .label("Edit")
                                .on_click(move |_, window, cx| {
                                    if let Some(view) = view_edit.upgrade() {
                                        view.update(cx, |this, cx| {
                                            this.open_edit_recipe_modal(&rec_edit, window, cx);
                                        });
                                    }
                                }),
                        )
                        .child(
                            Button::new(format!("btn-del-rec-{}", recipe_id))
                                .ghost()
                                .label("🗑")
                                .on_click(move |_, _, cx| {
                                    if let Some(view) = view_delete.upgrade() {
                                        view.update(cx, |this, cx| {
                                            this.delete_recipe(recipe_id, cx);
                                        });
                                    }
                                }),
                        ),
                ),
        )
    }
}

pub struct RecipeListDelegate {
    pub recipes: Vec<Recipe>,
    pub sections: Vec<RecipeSection>,
    pub selected_index: Option<IndexPath>,
    pub query: String,
    pub group_mode: RecipeGroupMode,
    pub sort_mode: RecipeSortMode,
    pub view: WeakEntity<RecipesView>,
}

impl RecipeListDelegate {
    pub fn prepare(&mut self, query: String) {
        self.query = query;
        let q = self.query.to_lowercase();

        let filtered: Vec<Recipe> = self
            .recipes
            .iter()
            .filter(|r| {
                if q.is_empty() {
                    true
                } else {
                    r.name.to_lowercase().contains(&q)
                        || r.meal_type
                            .as_ref()
                            .map(|m| m.to_lowercase().contains(&q))
                            .unwrap_or(false)
                }
            })
            .cloned()
            .collect();

        let mut groups: BTreeMap<String, Vec<Recipe>> = BTreeMap::new();
        for r in filtered {
            let key = match self.group_mode {
                RecipeGroupMode::MealType => r
                    .meal_type
                    .clone()
                    .unwrap_or_else(|| "Other / Uncategorized".to_string()),
                RecipeGroupMode::Alphabetical => {
                    let first = r.name.chars().next().unwrap_or('#').to_ascii_uppercase();
                    if first.is_alphabetic() {
                        first.to_string()
                    } else {
                        "#".to_string()
                    }
                }
                RecipeGroupMode::Servings => {
                    if r.servings <= dec!(2) {
                        "1-2 Servings (Small Batch)".to_string()
                    } else if r.servings <= dec!(4) {
                        "3-4 Servings (Standard)".to_string()
                    } else {
                        "5+ Servings (Large Family)".to_string()
                    }
                }
            };
            groups.entry(key).or_default().push(r);
        }

        self.sections = groups
            .into_iter()
            .map(|(title, mut recipes)| {
                match self.sort_mode {
                    RecipeSortMode::NameAsc => recipes.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase())),
                    RecipeSortMode::NameDesc => recipes.sort_by(|a, b| b.name.to_lowercase().cmp(&a.name.to_lowercase())),
                    RecipeSortMode::ServingsDesc => recipes.sort_by(|a, b| b.servings.cmp(&a.servings)),
                }
                RecipeSection { title, recipes }
            })
            .collect();
    }
}

impl ListDelegate for RecipeListDelegate {
    type Item = RecipeListItem;

    fn sections_count(&self, _: &App) -> usize {
        self.sections.len()
    }

    fn items_count(&self, section: usize, _: &App) -> usize {
        self.sections.get(section).map(|s| s.recipes.len()).unwrap_or(0)
    }

    fn render_section_header(
        &mut self,
        section: usize,
        _window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Option<impl IntoElement> {
        let sec = self.sections.get(section)?;
        Some(
            div()
                .px_3()
                .py_1_5()
                .bg(cx.theme().muted)
                .border_b_1()
                .border_color(cx.theme().border)
                .flex()
                .items_center()
                .justify_between()
                .child(
                    div()
                        .text_xs()
                        .font_weight(FontWeight::BOLD)
                        .text_color(cx.theme().foreground)
                        .child(sec.title.clone()),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(cx.theme().muted_foreground)
                        .child(format!("{} recipes", sec.recipes.len())),
                ),
        )
    }

    fn set_selected_index(
        &mut self,
        ix: Option<IndexPath>,
        _: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) {
        self.selected_index = ix;
        let selected_recipe_id = ix.and_then(|idx| {
            self.sections.get(idx.section).and_then(|s| s.recipes.get(idx.row).map(|r| r.id))
        });

        if let Some(view) = self.view.upgrade() {
            view.update(cx, |view, cx| {
                view.selected_recipe_id = selected_recipe_id;
                view.reload_data(cx);
            });
        }
        cx.notify();
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _: &mut Window,
        _: &mut Context<ListState<Self>>,
    ) -> Option<Self::Item> {
        let selected = Some(ix) == self.selected_index;
        let recipe = self.sections.get(ix.section)?.recipes.get(ix.row)?;
        Some(RecipeListItem {
            base: ListItem::new(format!("recipe-row-{}", recipe.id)).selected(selected),
            recipe: recipe.clone(),
            selected,
            view: self.view.clone(),
        })
    }
}

pub struct RecipesView {
    pub services: AppServices,
    pub recipe_list: Entity<ListState<RecipeListDelegate>>,
    pub status_msg: String,

    pub cached_recipes: Vec<Recipe>,
    pub cached_items: Vec<Item>,
    pub cached_cost: Option<RecipeCost>,

    pub selected_recipe_id: Option<RecipeId>,

    // Recipe Editor Form State
    pub editing_recipe_id: Option<RecipeId>,
    pub recipe_form_name: String,
    pub recipe_form_servings: Decimal,
    pub recipe_form_instructions: String,
    pub recipe_form_yields: Vec<(ItemId, Quantity)>,
    pub recipe_form_ingredients: Vec<IngredientEdge>,
    pub recipe_form_meal_type: String,

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

    // Grouping & Sorting state
    pub group_mode: RecipeGroupMode,
    pub sort_mode: RecipeSortMode,
}

impl RecipesView {
    pub fn new(services: AppServices, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let view = cx.entity().downgrade();
        let delegate = RecipeListDelegate {
            recipes: Vec::new(),
            sections: Vec::new(),
            selected_index: None,
            query: String::new(),
            group_mode: RecipeGroupMode::MealType,
            sort_mode: RecipeSortMode::NameAsc,
            view,
        };

        let recipe_list = cx.new(|cx| ListState::new(delegate, window, cx).searchable(true));

        let mut view_state = Self {
            services,
            recipe_list,
            status_msg: "Recipes manager ready".to_string(),

            cached_recipes: Vec::new(),
            cached_items: Vec::new(),
            cached_cost: None,

            selected_recipe_id: None,

            editing_recipe_id: None,
            recipe_form_name: String::new(),
            recipe_form_servings: dec!(4),
            recipe_form_instructions: String::new(),
            recipe_form_yields: Vec::new(),
            recipe_form_ingredients: Vec::new(),
            recipe_form_meal_type: "Dinner".to_string(),

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

            group_mode: RecipeGroupMode::MealType,
            sort_mode: RecipeSortMode::NameAsc,
        };
        view_state.reload_data(cx);
        view_state
    }

    pub fn reload_data(&mut self, cx: &mut Context<Self>) {
        self.cached_recipes = self.services.recipes.list_recipes().unwrap_or_default();
        self.cached_items = self.services.items.list_items().unwrap_or_default();
        if let Some(id) = self.selected_recipe_id {
            self.cached_cost = self.services.recipes.estimate_cost(id).ok();
        } else {
            self.cached_cost = None;
        }

        let recipe_list = self.recipe_list.clone();
        let cached_recipes = self.cached_recipes.clone();
        let group_mode = self.group_mode;
        let sort_mode = self.sort_mode;

        cx.defer(move |cx| {
            recipe_list.update(cx, |list, cx| {
                list.delegate_mut().recipes = cached_recipes;
                list.delegate_mut().group_mode = group_mode;
                list.delegate_mut().sort_mode = sort_mode;

                let query = list.delegate().query.clone();
                list.delegate_mut().prepare(query);
                cx.notify();
            });
        });
        cx.notify();
    }

    pub fn delete_recipe(&mut self, recipe_id: RecipeId, cx: &mut Context<Self>) {
        match self.services.recipes.delete_recipe(recipe_id) {
            Ok(_) => {
                self.status_msg = "Recipe deleted successfully".to_string();
                if self.selected_recipe_id == Some(recipe_id) {
                    self.selected_recipe_id = None;
                }
            }
            Err(e) => {
                self.status_msg = format!("Error deleting recipe: {}", e);
            }
        }
        self.reload_data(cx);
    }

    pub fn open_create_recipe_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let items = self.services.items.list_items().unwrap_or_default();
        let first_item = items.first().map(|i| i.id);

        self.editing_recipe_id = None;
        self.recipe_form_name = String::new();
        self.recipe_form_servings = dec!(4);
        self.recipe_form_instructions = String::new();
        self.recipe_form_yields = Vec::new();
        self.recipe_form_ingredients = Vec::new();
        self.recipe_form_meal_type = "Dinner".to_string();

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

        self.show_recipe_dialog(window, cx);
    }

    pub fn open_edit_recipe_modal(&mut self, recipe: &Recipe, window: &mut Window, cx: &mut Context<Self>) {
        let items = self.services.items.list_items().unwrap_or_default();
        let first_item = items.first().map(|i| i.id);

        self.editing_recipe_id = Some(recipe.id);
        self.recipe_form_name = recipe.name.clone();
        self.recipe_form_servings = recipe.servings;
        self.recipe_form_instructions = recipe.instructions.clone();
        self.recipe_form_yields = recipe.yields.clone();
        self.recipe_form_ingredients = recipe.ingredients.clone();
        self.recipe_form_meal_type = recipe.meal_type.clone().unwrap_or_else(|| "Dinner".to_string());

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

        self.show_recipe_dialog(window, cx);
    }

    fn show_recipe_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let view = cx.entity().clone();
        window.open_dialog(cx, move |dialog, _, _| {
            let view = view.clone();
            dialog
                .w(px(650.))
                .content(move |content, _, cx| {
                    let view_read = view.read(cx);
                    let is_edit = view_read.editing_recipe_id.is_some();
                    let title = if is_edit { "Edit Recipe Definition" } else { "Create New Recipe" };

                    let items = view_read.services.items.list_items().unwrap_or_default();
                    let item_options: Vec<SelectOption> = items
                        .iter()
                        .map(|i| SelectOption::new(i.id.0.to_string(), i.name.clone()))
                        .collect();

                    let recipes = view_read.services.recipes.list_recipes().unwrap_or_default();
                    let recipe_options: Vec<SelectOption> = recipes
                        .iter()
                        .filter(|r| Some(r.id) != view_read.editing_recipe_id)
                        .map(|r| SelectOption::new(r.id.0.to_string(), format!("Recipe: {}", r.name)))
                        .collect();

                    let mut sub_options = vec![SelectOption::new("none", "None (No Substitute)")];
                    for i in &items {
                        sub_options.push(SelectOption::new(i.id.0.to_string(), i.name.clone()));
                    }

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
                    ];

                    let target_type_options = vec![
                        SelectOption::new("item", "Raw Ingredient Item"),
                        SelectOption::new("recipe", "Sub-Recipe"),
                    ];

                    let meal_type_options = vec![
                        SelectOption::new("Breakfast", "Breakfast"),
                        SelectOption::new("Lunch", "Lunch"),
                        SelectOption::new("Dinner", "Dinner"),
                        SelectOption::new("Dessert", "Dessert"),
                        SelectOption::new("Snack", "Snack"),
                        SelectOption::new("Base / Component", "Base / Component"),
                    ];

                    let form_name = view_read.recipe_form_name.clone();
                    let form_servings = view_read.recipe_form_servings;
                    let form_instructions = view_read.recipe_form_instructions.clone();
                    let form_yields = view_read.recipe_form_yields.clone();
                    let form_ingredients = view_read.recipe_form_ingredients.clone();
                    let form_meal_type = view_read.recipe_form_meal_type.clone();

                    let ing_is_recipe = view_read.ing_target_is_recipe;
                    let ing_target_item_id = view_read.ing_target_item_id;
                    let ing_target_recipe_id = view_read.ing_target_recipe_id;
                    let ing_amount = view_read.ing_amount;
                    let ing_unit = view_read.ing_unit.clone();
                    let ing_required = view_read.ing_required;
                    let ing_sub_id = view_read.ing_substitute_id;

                    let yield_item_id = view_read.yield_item_id;
                    let yield_amount = view_read.yield_amount;
                    let yield_unit = view_read.yield_unit.clone();

                    let v_servings = view.clone();
                    let v_meal_type = view.clone();
                    let v_y_item = view.clone();
                    let v_y_amt = view.clone();
                    let v_y_unit = view.clone();
                    let v_y_add = view.clone();
                    let v_y_rem = view.clone();

                    let v_ing_type = view.clone();
                    let v_target_item = view.clone();
                    let v_target_rec = view.clone();
                    let v_amount = view.clone();
                    let v_unit = view.clone();
                    let v_req = view.clone();
                    let v_sub = view.clone();
                    let v_add_edge = view.clone();
                    let v_rem_edge = view.clone();
                    let v_save = view.clone();

                    let items_map: HashMap<ItemId, String> = items.iter().map(|i| (i.id, i.name.clone())).collect();
                    let recipes_map: HashMap<RecipeId, String> = recipes.iter().map(|r| (r.id, r.name.clone())).collect();

                    content
                        .child(
                            DialogHeader::new()
                                .child(DialogTitle::new().child(title))
                                .child(DialogDescription::new().child("Configure recipe yields, ingredient edges, sub-recipes, substitutes, and instructions")),
                        )
                        .child(
                            div()
                                .py_4()
                                .flex()
                                .flex_col()
                                .gap_3()
                                .child(
                                    FormInput::new("input-recipe-name")
                                        .label("Recipe Name")
                                        .placeholder("e.g. Homemade Bolognese Sauce")
                                        .value(form_name),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap_2()
                                        .child(
                                            NumberInput::new("input-recipe-servings", form_servings)
                                                .label("Base Servings Count")
                                                .step(dec!(1))
                                                .on_increment({
                                                    let v = v_servings.clone();
                                                    move |val, _window, cx| {
                                                        v.update(cx, |this, cx| {
                                                            this.recipe_form_servings = *val;
                                                            cx.notify();
                                                        });
                                                    }
                                                })
                                                .on_decrement({
                                                    let v = v_servings.clone();
                                                    move |val, _window, cx| {
                                                        v.update(cx, |this, cx| {
                                                            this.recipe_form_servings = *val;
                                                            cx.notify();
                                                        });
                                                    }
                                                }),
                                        )
                                        .child(
                                            Select::new("select-recipe-meal-type", meal_type_options)
                                                .label("Meal Type / Category")
                                                .selected_id(Some(form_meal_type))
                                                .on_select(move |opt: &SelectOption, _window, cx| {
                                                    let mt = opt.id.clone();
                                                    v_meal_type.update(cx, |this, cx| {
                                                        this.recipe_form_meal_type = mt;
                                                        cx.notify();
                                                    });
                                                }),
                                        ),
                                )
                                .child(
                                    FormInput::new("input-recipe-instructions")
                                        .label("Preparation Instructions")
                                        .placeholder("e.g. Sauté onions, brown meat, simmer for 45 mins")
                                        .value(form_instructions),
                                )
                                // Section: Recipe Yields (Produces)
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_2()
                                        .p_3()
                                        .bg(cx.theme().muted)
                                        .rounded_md()
                                        .child(div().text_xs().font_weight(FontWeight::BOLD).child("Recipe Yields (Produced Output)"))
                                        .child(
                                            Select::new("select-yield-item", item_options.clone())
                                                .label("Yield Item")
                                                .selected_id(yield_item_id.map(|id| id.0.to_string()))
                                                .on_select(move |opt: &SelectOption, _window, cx| {
                                                    if let Ok(uuid) = uuid::Uuid::from_str(&opt.id) {
                                                        v_y_item.update(cx, |this, cx| {
                                                            this.yield_item_id = Some(ItemId(uuid));
                                                            cx.notify();
                                                        });
                                                    }
                                                }),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .gap_2()
                                                .child(
                                                    NumberInput::new("input-yield-amount", yield_amount)
                                                        .label("Yield Amount")
                                                        .step(dec!(1))
                                                        .on_increment({
                                                            let v = v_y_amt.clone();
                                                            move |val, _, cx| v.update(cx, |this, cx| { this.yield_amount = *val; cx.notify(); })
                                                        })
                                                        .on_decrement({
                                                            let v = v_y_amt.clone();
                                                            move |val, _, cx| v.update(cx, |this, cx| { this.yield_amount = *val; cx.notify(); })
                                                        }),
                                                )
                                                .child(
                                                    Select::new("select-yield-unit", unit_options.clone())
                                                        .label("Yield Unit")
                                                        .selected_id(Some(format!("{:?}", yield_unit)))
                                                        .on_select(move |opt: &SelectOption, _, cx| {
                                                            let unit = match opt.id.as_str() {
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
                                                            v_y_unit.update(cx, |this, cx| { this.yield_unit = unit; cx.notify(); });
                                                        }),
                                                ),
                                        )
                                        .child(
                                            Button::new("btn-add-yield-to-form")
                                                .secondary()
                                                .label("+ Add Yield Item")
                                                .on_click(move |_, _, cx| {
                                                    v_y_add.update(cx, |this, cx| {
                                                        if let Some(y_id) = this.yield_item_id {
                                                            if let Ok(qty) = Quantity::new(this.yield_amount, this.yield_unit.clone()) {
                                                                this.recipe_form_yields.push((y_id, qty));
                                                            }
                                                        }
                                                        cx.notify();
                                                    });
                                                }),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .gap_1()
                                                .children(form_yields.into_iter().enumerate().map(|(idx, (y_id, qty))| {
                                                    let v_y_del = v_y_rem.clone();
                                                    let item_label = items_map.get(&y_id).cloned().unwrap_or_else(|| "Unknown".to_string());
                                                    div()
                                                        .flex()
                                                        .items_center()
                                                        .justify_between()
                                                        .p_2()
                                                        .bg(cx.theme().background)
                                                        .border_1()
                                                        .border_color(cx.theme().border)
                                                        .rounded_md()
                                                        .child(div().text_xs().child(format!("Yield: {} ({} {})", item_label, qty.amount.normalize(), qty.unit)))
                                                        .child(
                                                            Button::new(format!("btn-rem-yield-{}", idx))
                                                                .ghost()
                                                                .label("Remove")
                                                                .on_click(move |_, _, cx| {
                                                                    v_y_del.update(cx, |this, cx| {
                                                                        if idx < this.recipe_form_yields.len() {
                                                                            this.recipe_form_yields.remove(idx);
                                                                        }
                                                                        cx.notify();
                                                                    });
                                                                }),
                                                        )
                                                })),
                                        ),
                                )
                                // Section: Add Ingredient Edge (Sub-Recipe or Item)
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_2()
                                        .p_3()
                                        .bg(cx.theme().muted)
                                        .rounded_md()
                                        .child(div().text_xs().font_weight(FontWeight::BOLD).child("Add Ingredient Edge (Item or Sub-Recipe)"))
                                        .child(
                                            Select::new("select-ing-target-type", target_type_options)
                                                .label("Target Component Type")
                                                .selected_id(Some(if ing_is_recipe { "recipe".to_string() } else { "item".to_string() }))
                                                .on_select(move |opt: &SelectOption, _, cx| {
                                                    let is_rec = opt.id == "recipe";
                                                    v_ing_type.update(cx, |this, cx| {
                                                        this.ing_target_is_recipe = is_rec;
                                                        cx.notify();
                                                    });
                                                }),
                                        )
                                        .child(
                                            if ing_is_recipe {
                                                Select::new("select-ing-recipe-target", recipe_options)
                                                    .label("Sub-Recipe Target")
                                                    .selected_id(ing_target_recipe_id.map(|id| id.0.to_string()))
                                                    .on_select(move |opt: &SelectOption, _, cx| {
                                                        if let Ok(uuid) = uuid::Uuid::from_str(&opt.id) {
                                                            v_target_rec.update(cx, |this, cx| {
                                                                this.ing_target_recipe_id = Some(RecipeId(uuid));
                                                                cx.notify();
                                                            });
                                                        }
                                                    })
                                            } else {
                                                Select::new("select-ing-item-target", item_options.clone())
                                                    .label("Ingredient Item Target")
                                                    .selected_id(ing_target_item_id.map(|id| id.0.to_string()))
                                                    .on_select(move |opt: &SelectOption, _, cx| {
                                                        if let Ok(uuid) = uuid::Uuid::from_str(&opt.id) {
                                                            v_target_item.update(cx, |this, cx| {
                                                                this.ing_target_item_id = Some(ItemId(uuid));
                                                                cx.notify();
                                                            });
                                                        }
                                                    })
                                            },
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .gap_2()
                                                .child(
                                                    NumberInput::new("input-ing-amount", ing_amount)
                                                        .label("Amount")
                                                        .step(dec!(10))
                                                        .on_increment({
                                                            let v = v_amount.clone();
                                                            move |val, _, cx| v.update(cx, |this, cx| { this.ing_amount = *val; cx.notify(); })
                                                        })
                                                        .on_decrement({
                                                            let v = v_amount.clone();
                                                            move |val, _, cx| v.update(cx, |this, cx| { this.ing_amount = *val; cx.notify(); })
                                                        }),
                                                )
                                                .child(
                                                    Select::new("select-ing-unit", unit_options.clone())
                                                        .label("Unit")
                                                        .selected_id(Some(format!("{:?}", ing_unit)))
                                                        .on_select(move |opt: &SelectOption, _, cx| {
                                                            let unit = match opt.id.as_str() {
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
                                                            v_unit.update(cx, |this, cx| { this.ing_unit = unit; cx.notify(); });
                                                        }),
                                                ),
                                        )
                                        .child(
                                            Select::new("select-ing-substitute", sub_options)
                                                .label("Optional Per-Recipe Substitute")
                                                .selected_id(ing_sub_id.map(|id| id.0.to_string()).or_else(|| Some("none".to_string())))
                                                .on_select(move |opt: &SelectOption, _, cx| {
                                                    let sub = if opt.id == "none" {
                                                        None
                                                    } else {
                                                        uuid::Uuid::from_str(&opt.id).ok().map(ItemId)
                                                    };
                                                    v_sub.update(cx, |this, cx| {
                                                        this.ing_substitute_id = sub;
                                                        cx.notify();
                                                    });
                                                }),
                                        )
                                        .child(
                                            Checkbox::new("cb-ing-required")
                                                .label("Required Ingredient (must not be omitted)")
                                                .checked(ing_required)
                                                .on_click(move |checked, _, cx| {
                                                    v_req.update(cx, |this, cx| {
                                                        this.ing_required = *checked;
                                                        cx.notify();
                                                    });
                                                }),
                                        )
                                        .child(
                                            Button::new("btn-add-edge-to-form")
                                                .secondary()
                                                .label("+ Add Edge to Recipe")
                                                .on_click(move |_, _, cx| {
                                                    v_add_edge.update(cx, |this, cx| {
                                                        let target = if this.ing_target_is_recipe {
                                                            this.ing_target_recipe_id.map(ItemOrRecipeId::Recipe)
                                                        } else {
                                                            this.ing_target_item_id.map(ItemOrRecipeId::Item)
                                                        };

                                                        if let Some(t) = target {
                                                            if let Ok(qty) = Quantity::new(this.ing_amount, this.ing_unit.clone()) {
                                                                this.recipe_form_ingredients.push(IngredientEdge {
                                                                    target: t,
                                                                    quantity: qty,
                                                                    required: this.ing_required,
                                                                    per_recipe_substitute: this.ing_substitute_id,
                                                                    cycle_flag: false,
                                                                });
                                                            }
                                                        }
                                                        cx.notify();
                                                    });
                                                }),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_1()
                                        .child(div().text_xs().font_weight(FontWeight::BOLD).child("Current Ingredients List"))
                                        .children(form_ingredients.into_iter().enumerate().map(|(idx, edge)| {
                                            let v_rem = v_rem_edge.clone();
                                            let target_desc = match edge.target {
                                                ItemOrRecipeId::Item(id) => items_map.get(&id).cloned().unwrap_or_else(|| format!("Item {:?}", id)),
                                                ItemOrRecipeId::Recipe(id) => recipes_map.get(&id).cloned().unwrap_or_else(|| format!("Recipe {:?}", id)),
                                            };
                                            let sub_desc = edge.per_recipe_substitute.and_then(|sid| items_map.get(&sid)).map(|s| format!(" [Sub: {}]", s)).unwrap_or_default();
                                            let req_desc = if edge.required { "" } else { " (Optional)" };
                                            div()
                                                .flex()
                                                .items_center()
                                                .justify_between()
                                                .p_2()
                                                .bg(cx.theme().background)
                                                .border_1()
                                                .border_color(cx.theme().border)
                                                .rounded_md()
                                                .child(div().text_xs().child(format!("{} - {} {}{}{}", target_desc, edge.quantity.amount.normalize(), edge.quantity.unit, req_desc, sub_desc)))
                                                .child(
                                                    Button::new(format!("btn-rem-edge-{}", idx))
                                                        .ghost()
                                                        .label("Remove")
                                                        .on_click(move |_, _, cx| {
                                                            v_rem.update(cx, |this, cx| {
                                                                if idx < this.recipe_form_ingredients.len() {
                                                                    this.recipe_form_ingredients.remove(idx);
                                                                }
                                                                cx.notify();
                                                            });
                                                        }),
                                                )
                                        })),
                                ),
                        )
                        .child(
                            DialogFooter::new()
                                .child(
                                    Button::new("btn-cancel-recipe-modal")
                                        .secondary()
                                        .label("Cancel")
                                        .on_click(|_, window, cx| {
                                            window.close_dialog(cx);
                                        }),
                                )
                                .child(
                                    Button::new("btn-save-recipe-modal")
                                        .primary()
                                        .label("Save Recipe")
                                        .on_click(move |_, window, cx| {
                                            v_save.update(cx, |this, cx| {
                                                this.save_recipe(cx);
                                            });
                                            window.close_dialog(cx);
                                        }),
                                ),
                        )
                })
        });
    }

    pub fn save_recipe(&mut self, cx: &mut Context<Self>) {
        if self.recipe_form_name.trim().is_empty() {
            self.status_msg = "Error: Recipe name cannot be empty".to_string();
            cx.notify();
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
        recipe.meal_type = Some(self.recipe_form_meal_type.clone());

        match self.services.recipes.save_recipe(recipe) {
            Ok(saved) => {
                self.status_msg = format!("Saved recipe: {}", saved.name);
            }
            Err(e) => {
                self.status_msg = format!("Error saving recipe: {}", e);
            }
        }
        self.reload_data(cx);
    }

    pub fn open_make_recipe_modal(&mut self, recipe: &Recipe, window: &mut Window, cx: &mut Context<Self>) {
        self.make_batches = dec!(1.0);
        self.make_selected_yield = recipe.yields.first().map(|y| y.0);
        self.make_status = String::new();

        let recipe_clone = recipe.clone();
        let view = cx.entity().clone();

        window.open_dialog(cx, move |dialog, _, _| {
            let view = view.clone();
            let rec = recipe_clone.clone();

            dialog.w(px(500.)).content(move |content, _, cx| {
                let view_read = view.read(cx);
                let batches = view_read.make_batches;
                let status = view_read.make_status.clone();
                let v_dec = view.clone();
                let v_inc = view.clone();
                let v_exec = view.clone();
                let rec_exec = rec.clone();

                content
                    .child(
                        DialogHeader::new()
                            .child(DialogTitle::new().child(format!("Make Recipe: {}", rec.name)))
                            .child(DialogDescription::new().child("Scale batches, consume pantry ingredients, and produce yields")),
                    )
                    .child(
                        div()
                            .py_4()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .child(div().text_sm().font_weight(FontWeight::BOLD).child("Batch Multiplier:"))
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(
                                                Button::new("btn-dec-batch")
                                                    .secondary()
                                                    .label("- 0.5")
                                                    .on_click(move |_, _, cx| {
                                                        v_dec.update(cx, |this, cx| {
                                                            if this.make_batches > dec!(0.5) {
                                                                this.make_batches -= dec!(0.5);
                                                                cx.notify();
                                                            }
                                                        });
                                                    }),
                                            )
                                            .child(Tag::new().child(format!("{}x", batches.normalize())))
                                            .child(
                                                Button::new("btn-inc-batch")
                                                    .secondary()
                                                    .label("+ 0.5")
                                                    .on_click(move |_, _, cx| {
                                                        v_inc.update(cx, |this, cx| {
                                                            this.make_batches += dec!(0.5);
                                                            cx.notify();
                                                        });
                                                    }),
                                            ),
                                    ),
                            )
                            .when(!status.is_empty(), |this| {
                                this.child(Alert::new("make-status-alert", status.clone()))
                            }),
                    )
                    .child(
                        DialogFooter::new()
                            .child(
                                Button::new("btn-close-make-modal")
                                    .secondary()
                                    .label("Close")
                                    .on_click(|_, window, cx| {
                                        window.close_dialog(cx);
                                    }),
                            )
                            .child(
                                Button::new("btn-confirm-make-recipe")
                                    .primary()
                                    .label("Cook & Deduct Pantry")
                                    .on_click(move |_, window, cx| {
                                        let r = rec_exec.clone();
                                        v_exec.update(cx, |this, cx| {
                                            this.execute_make_recipe(&r, cx);
                                        });
                                        window.close_dialog(cx);
                                    }),
                            ),
                    )
            })
        });
    }

    pub fn execute_make_recipe(&mut self, recipe: &Recipe, cx: &mut Context<Self>) {
        let items_map: HashMap<ItemId, Item> =
            self.cached_items.iter().map(|i| (i.id, i.clone())).collect();
        let recipes_map: HashMap<RecipeId, Recipe> =
            self.cached_recipes.iter().map(|r| (r.id, r.clone())).collect();

        let mut config = MakeRecipeConfig::default();
        config.batches = self.make_batches;
        config.selected_yield_item = self.make_selected_yield;

        match evaluate_make_recipe_full(recipe, &config, &items_map, &recipes_map) {
            Ok(execution) => {
                for (item_id, qty) in execution.ingredients_to_consume {
                    let _ = self.services.pantry.consume_pantry_item(item_id, qty.amount, qty.unit);
                }
                for (yield_id, qty) in execution.yields_produced {
                    let _ = self.services.pantry.add_pantry_entry(yield_id, qty.amount, qty.unit, None);
                }
                self.status_msg = format!("Cooked {}x '{}'! Ingredients deducted & yield stored.", self.make_batches.normalize(), recipe.name);
            }
            Err(e) => {
                self.status_msg = format!("Make Recipe Error: {}", e);
            }
        }
        self.reload_data(cx);
    }
}

impl Render for RecipesView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selected_recipe = self
            .selected_recipe_id
            .and_then(|id| self.cached_recipes.iter().find(|r| r.id == id).cloned());

        let group_options = vec![
            SelectOption::new("MealType", "Group by Meal Type"),
            SelectOption::new("Alphabetical", "Group Alphabetically"),
            SelectOption::new("Servings", "Group by Servings Scale"),
        ];

        let sort_options = vec![
            SelectOption::new("NameAsc", "Sort A-Z"),
            SelectOption::new("NameDesc", "Sort Z-A"),
            SelectOption::new("ServingsDesc", "Sort by Servings (High to Low)"),
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
                                    .child("Recipe Matrix & Sub-Recipe DAG"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Manage multi-level recipe nesting, sourdough starter cycle awareness, and batch cook execution"),
                            ),
                    )
                    .child(Alert::new("recipe-status-alert", format!("Status: {}", self.status_msg))),
            )
            // Controls & Filter Bar
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Select::new("select-recipe-group-mode", group_options)
                                    .selected_id(Some(format!("{:?}", self.group_mode)))
                                    .on_select(cx.listener(|this, opt: &SelectOption, _window, cx| {
                                        this.group_mode = match opt.id.as_str() {
                                            "Alphabetical" => RecipeGroupMode::Alphabetical,
                                            "Servings" => RecipeGroupMode::Servings,
                                            _ => RecipeGroupMode::MealType,
                                        };
                                        this.reload_data(cx);
                                    })),
                            )
                            .child(
                                Select::new("select-recipe-sort-mode", sort_options)
                                    .selected_id(Some(format!("{:?}", self.sort_mode)))
                                    .on_select(cx.listener(|this, opt: &SelectOption, _window, cx| {
                                        this.sort_mode = match opt.id.as_str() {
                                            "NameDesc" => RecipeSortMode::NameDesc,
                                            "ServingsDesc" => RecipeSortMode::ServingsDesc,
                                            _ => RecipeSortMode::NameAsc,
                                        };
                                        this.reload_data(cx);
                                    })),
                            ),
                    )
                    .child(
                        Button::new("btn-open-create-recipe")
                            .primary()
                            .label("+ Create New Recipe")
                            .on_click(cx.listener(|this, _event, window, cx| {
                                this.open_create_recipe_modal(window, cx);
                            })),
                    ),
            )
            // Main Grid Layout: Virtualized Recipe List on Left, Detail / Costing on Right
            .child(
                div()
                    .flex()
                    .gap_4()
                    .flex_1()
                    .child(
                        div()
                            .w_1_2()
                            .h_full()
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_xl()
                            .overflow_hidden()
                            .child(List::new(&self.recipe_list)),
                    )
                    .child(
                        div()
                            .w_1_2()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .child(
                                if let Some(recipe) = selected_recipe {
                                    let cost_display = if let Some(cost) = &self.cached_cost {
                                        format!("Est. Cost: ${} / batch (${} / serving)", cost.price_per_batch, cost.price_per_serving)
                                    } else {
                                        "Cost: Unknown / Missing Store Packages".to_string()
                                    };

                                    div()
                                        .flex()
                                        .flex_col()
                                        .gap_4()
                                        .p_4()
                                        .bg(cx.theme().muted)
                                        .border_1()
                                        .border_color(cx.theme().border)
                                        .rounded_xl()
                                        .child(
                                            div()
                                                .flex()
                                                .items_center()
                                                .justify_between()
                                                .child(
                                                    div()
                                                        .text_lg()
                                                        .font_weight(FontWeight::BOLD)
                                                        .child(format!("Selected: {}", recipe.name)),
                                                )
                                                .child(Badge::new().child(recipe.meal_type.clone().unwrap_or_else(|| "Dinner".to_string()))),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::BOLD)
                                                .text_color(cx.theme().foreground)
                                                .child(cost_display),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(format!("Instructions: {}", if recipe.instructions.is_empty() { "None" } else { &recipe.instructions })),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .font_weight(FontWeight::BOLD)
                                                .child("Ingredients / Component Edges:"),
                                        )
                                        .child(
                                            div()
                                                .flex()
                                                .flex_col()
                                                .gap_1()
                                                .children(recipe.ingredients.iter().map(|edge| {
                                                    let is_cycle = edge.cycle_flag;
                                                    let is_opt = !edge.required;
                                                    div()
                                                        .flex()
                                                        .items_center()
                                                        .justify_between()
                                                        .text_xs()
                                                        .child(format!("{:?} - {} {}", edge.target, edge.quantity.amount.normalize(), edge.quantity.unit))
                                                        .child(
                                                            div()
                                                                .flex()
                                                                .gap_1()
                                                                .when(is_opt, |this| this.child(Badge::new().child("Optional")))
                                                                .when(is_cycle, |this| this.child(Badge::new().child("Cycle"))),
                                                        )
                                                })),
                                        )
                                } else {
                                    div()
                                        .p_6()
                                        .bg(cx.theme().muted)
                                        .border_1()
                                        .border_color(cx.theme().border)
                                        .rounded_xl()
                                        .child(
                                            div()
                                                .text_sm()
                                                .text_color(cx.theme().muted_foreground)
                                                .child("Select a recipe from the list to view nested expansion breakdown, costing analysis, or to initiate a batch make."),
                                        )
                                },
                            ),
                    ),
            )
    }
}
