use crate::components::*;
use bmp_domain::*;
use bmp_services::AppServices;
use gpui::prelude::*;
use gpui::*;
use gpui_component::alert::Alert;
use gpui_component::badge::Badge;
use gpui_component::button::{Button, ButtonVariants};
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
    pub view: Entity<RecipesView>,
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
                                    view_make.update(cx, |this, cx| {
                                        this.open_make_recipe_modal(&rec_make, window, cx);
                                    });
                                }),
                        )
                        .child(
                            Button::new(format!("btn-edit-rec-{}", recipe_id))
                                .secondary()
                                .label("Edit")
                                .on_click(move |_, window, cx| {
                                    view_edit.update(cx, |this, cx| {
                                        this.open_edit_recipe_modal(&rec_edit, window, cx);
                                    });
                                }),
                        )
                        .child(
                            Button::new(format!("btn-del-rec-{}", recipe_id))
                                .ghost()
                                .label("🗑")
                                .on_click(move |_, _, cx| {
                                    view_delete.update(cx, |this, cx| {
                                        this.delete_recipe(recipe_id, cx);
                                    });
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
    pub view: Entity<RecipesView>,
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
                }
            })
            .cloned()
            .collect();

        let mut groups: BTreeMap<String, Vec<Recipe>> = BTreeMap::new();
        for recipe in filtered {
            let key = match self.group_mode {
                RecipeGroupMode::MealType => recipe
                    .meal_type
                    .clone()
                    .unwrap_or_else(|| "General Recipes".to_string()),
                RecipeGroupMode::Alphabetical => {
                    let first_char = recipe
                        .name
                        .chars()
                        .next()
                        .unwrap_or('A')
                        .to_uppercase()
                        .to_string();
                    format!("Letter {}", first_char)
                }
                RecipeGroupMode::Servings => {
                    format!("{} Servings", recipe.servings.normalize())
                }
            };
            groups.entry(key).or_default().push(recipe);
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

        self.view.update(cx, |view, cx| {
            view.selected_recipe_id = selected_recipe_id;
            view.reload_data(cx);
        });
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
        let view = cx.entity().clone();
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

    pub fn open_create_recipe_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let items = self.services.items.list_items().unwrap_or_default();
        let first_item = items.first().map(|i| i.id);

        self.editing_recipe_id = None;
        self.recipe_form_name = String::new();
        self.recipe_form_servings = dec!(4);
        self.recipe_form_instructions = "Mix ingredients thoroughly and cook as directed.".to_string();
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
                .w(px(600.))
                .content(move |content, _, cx| {
                    let view_read = view.read(cx);
                    let is_edit = view_read.editing_recipe_id.is_some();
                    let title = if is_edit { "Edit Recipe Definition" } else { "Create New Recipe" };

                    let items = view_read.services.items.list_items().unwrap_or_default();
                    let item_options: Vec<SelectOption> = items
                        .iter()
                        .map(|i| SelectOption::new(i.id.0.to_string(), i.name.clone()))
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
                    ];

                    let form_name = view_read.recipe_form_name.clone();
                    let form_servings = view_read.recipe_form_servings;
                    let form_instructions = view_read.recipe_form_instructions.clone();
                    let form_ingredients = view_read.recipe_form_ingredients.clone();

                    let ing_target_item_id = view_read.ing_target_item_id;
                    let ing_amount = view_read.ing_amount;
                    let ing_unit = view_read.ing_unit.clone();
                    let ing_required = view_read.ing_required;

                    let v_target = view.clone();
                    let v_amount = view.clone();
                    let v_unit = view.clone();
                    let v_req = view.clone();
                    let v_add_edge = view.clone();
                    let v_rem_edge = view.clone();
                    let v_save = view.clone();
                    let v_servings = view.clone();

                    content
                        .child(
                            DialogHeader::new()
                                .child(DialogTitle::new().child(title))
                                .child(DialogDescription::new().child("Configure recipe yields, ingredient edges, required/optional flags, and instructions")),
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
                                    FormInput::new("input-recipe-instructions")
                                        .label("Preparation Instructions")
                                        .placeholder("e.g. Sauté onions, brown meat, simmer for 45 mins")
                                        .value(form_instructions),
                                )
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
                                            Select::new("select-ing-item-target", item_options)
                                                .label("Target Ingredient Item")
                                                .selected_id(ing_target_item_id.map(|id| id.0.to_string()))
                                                .on_select(move |opt: &SelectOption, _window, cx| {
                                                    if let Ok(uuid) = uuid::Uuid::from_str(&opt.id) {
                                                        v_target.update(cx, |this, cx| {
                                                            this.ing_target_item_id = Some(ItemId(uuid));
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
                                                    NumberInput::new("input-ing-amount", ing_amount)
                                                        .label("Amount")
                                                        .step(dec!(10))
                                                        .on_increment({
                                                            let v = v_amount.clone();
                                                            move |val, _window, cx| {
                                                                v.update(cx, |this, cx| {
                                                                    this.ing_amount = *val;
                                                                    cx.notify();
                                                                });
                                                            }
                                                        })
                                                        .on_decrement({
                                                            let v = v_amount.clone();
                                                            move |val, _window, cx| {
                                                                v.update(cx, |this, cx| {
                                                                    this.ing_amount = *val;
                                                                    cx.notify();
                                                                });
                                                            }
                                                        }),
                                                )
                                                .child(
                                                    Select::new("select-ing-unit", unit_options)
                                                        .label("Unit")
                                                        .selected_id(Some(format!("{:?}", ing_unit)))
                                                        .on_select(move |opt: &SelectOption, _window, cx| {
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
                                                            v_unit.update(cx, |this, cx| {
                                                                this.ing_unit = unit;
                                                                cx.notify();
                                                            });
                                                        }),
                                                ),
                                        )
                                        .child(
                                            Checkbox::new("cb-ing-required")
                                                .label("Required Ingredient (must not be omitted)")
                                                .checked(ing_required)
                                                .on_click(move |checked, _window, cx| {
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
                                                .on_click(move |_, _window, cx| {
                                                    v_add_edge.update(cx, |this, cx| {
                                                        if let Some(target_id) = this.ing_target_item_id {
                                                            if let Ok(qty) = Quantity::new(this.ing_amount, this.ing_unit.clone()) {
                                                                this.recipe_form_ingredients.push(IngredientEdge {
                                                                    target: ItemOrRecipeId::Item(target_id),
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
                                            div()
                                                .flex()
                                                .items_center()
                                                .justify_between()
                                                .p_2()
                                                .bg(cx.theme().background)
                                                .border_1()
                                                .border_color(cx.theme().border)
                                                .rounded_md()
                                                .child(div().text_xs().child(format!("{:?} - {} {}", edge.target, edge.quantity.amount, edge.quantity.unit)))
                                                .child(
                                                    Button::new(format!("btn-rem-edge-{}", idx))
                                                        .ghost()
                                                        .label("Remove")
                                                        .on_click(move |_, _window, cx| {
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
        recipe.meal_type = Some(self.recipe_form_meal_type.trim().to_string());

        match self.services.recipes.save_recipe(recipe) {
            Ok(saved) => {
                self.status_msg = format!("Saved recipe: {}", saved.name);
                self.selected_recipe_id = Some(saved.id);
            }
            Err(e) => {
                self.status_msg = format!("Error saving recipe: {}", e);
            }
        }
        self.reload_data(cx);
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
        self.reload_data(cx);
    }

    pub fn open_make_recipe_modal(&mut self, recipe: &Recipe, window: &mut Window, cx: &mut Context<Self>) {
        self.selected_recipe_id = Some(recipe.id);
        self.make_batches = dec!(1.0);
        self.make_selected_yield = recipe.yields.first().map(|y| y.0);
        self.make_status = format!("Ready to batch cook '{}'", recipe.name);

        let view = cx.entity().clone();
        window.open_dialog(cx, move |dialog, _, _| {
            let view = view.clone();
            dialog
                .w(px(500.))
                .content(move |content, _, cx| {
                    let view_read = view.read(cx);
                    let make_batches = view_read.make_batches;
                    let make_status = view_read.make_status.clone();

                    let v_num = view.clone();
                    let v_exec = view.clone();

                    content
                        .child(
                            DialogHeader::new()
                                .child(DialogTitle::new().child("Execute Make Recipe"))
                                .child(DialogDescription::new().child("Batch cook recipe and update Pantry inventory")),
                        )
                        .child(
                            div()
                                .py_4()
                                .flex()
                                .flex_col()
                                .gap_3()
                                .child(
                                    NumberInput::new("input-make-batches", make_batches)
                                        .label("Batch Multiplier / Scale")
                                        .step(dec!(0.5))
                                        .unit("x")
                                        .on_increment({
                                            let v = v_num.clone();
                                            move |val, _window, cx| {
                                                v.update(cx, |this, cx| {
                                                    this.make_batches = *val;
                                                    cx.notify();
                                                });
                                            }
                                        })
                                        .on_decrement({
                                            let v = v_num.clone();
                                            move |val, _window, cx| {
                                                v.update(cx, |this, cx| {
                                                    this.make_batches = *val;
                                                    cx.notify();
                                                });
                                            }
                                        }),
                                )
                                .when(!make_status.is_empty(), |this| {
                                    this.child(
                                        div()
                                            .p_3()
                                            .bg(cx.theme().accent)
                                            .rounded_md()
                                            .text_xs()
                                            .font_weight(FontWeight::BOLD)
                                            .child(make_status),
                                    )
                                }),
                        )
                        .child(
                            DialogFooter::new()
                                .child(
                                    Button::new("btn-cancel-make-modal")
                                        .secondary()
                                        .label("Close")
                                        .on_click(|_, window, cx| {
                                            window.close_dialog(cx);
                                        }),
                                )
                                .child(
                                    Button::new("btn-confirm-make-cook")
                                        .primary()
                                        .label("🍳 Produce Batch & Update Pantry")
                                        .on_click(move |_, window, cx| {
                                            v_exec.update(cx, |this, cx| {
                                                this.execute_make_recipe(cx);
                                            });
                                            window.close_dialog(cx);
                                        }),
                                ),
                        )
                })
        });
    }

    pub fn execute_make_recipe(&mut self, cx: &mut Context<Self>) {
        let recipe_id = match self.selected_recipe_id {
            Some(id) => id,
            None => return,
        };

        let recipes = self.cached_recipes.clone();
        let recipe = match recipes.into_iter().find(|r| r.id == recipe_id) {
            Some(r) => r,
            None => return,
        };

        let items_list = self.cached_items.clone();
        let items_map: HashMap<ItemId, Item> = items_list.into_iter().map(|i| (i.id, i)).collect();

        let recipes_list = self.cached_recipes.clone();
        let recipes_map: HashMap<RecipeId, Recipe> = recipes_list.into_iter().map(|r| (r.id, r)).collect();

        let mut config = MakeRecipeConfig::default();
        config.batches = self.make_batches;
        config.selected_yield_item = self.make_selected_yield;

        match evaluate_make_recipe_full(&recipe, &config, &items_map, &recipes_map) {
            Ok(exec) => {
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
        self.reload_data(cx);
    }
}

impl Render for RecipesView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let items = self.cached_items.clone();

        let selected_recipe = self
            .selected_recipe_id
            .and_then(|id| self.cached_recipes.iter().find(|r| r.id == id).cloned());
        let has_selected_recipe = selected_recipe.is_some();
        let recipe_cost_estimate = self.cached_cost.clone();

        let group_options = vec![
            SelectOption::new("MealType", "Group by Meal Type"),
            SelectOption::new("Alphabetical", "Group Alphabetically"),
            SelectOption::new("Servings", "Group by Servings"),
        ];

        let sort_options = vec![
            SelectOption::new("NameAsc", "Sort A-Z"),
            SelectOption::new("NameDesc", "Sort Z-A"),
            SelectOption::new("ServingsDesc", "Sort by Servings"),
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
                            .child(Badge::new().child(format!(
                                "Saved Recipes: {}",
                                self.cached_recipes.len()
                            )))
                            .child(
                                Button::new("btn-new-recipe")
                                    .primary()
                                    .label("+ Create Recipe")
                                    .on_click(cx.listener(|this, _event, window, cx| {
                                        this.open_create_recipe_modal(window, cx);
                                    })),
                            ),
                    ),
            )
            // Section Grouping & Sorting Controls Toolbar
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .p_3()
                    .bg(cx.theme().muted)
                    .rounded_lg()
                    .child(
                        div().w_56().child(
                            Select::new("select-recipe-group-mode", group_options)
                                .label("Section Grouping")
                                .selected_id(Some(format!("{:?}", self.group_mode)))
                                .on_select(cx.listener(|this, opt: &SelectOption, _window, cx| {
                                    this.group_mode = match opt.id.as_str() {
                                        "Alphabetical" => RecipeGroupMode::Alphabetical,
                                        "Servings" => RecipeGroupMode::Servings,
                                        _ => RecipeGroupMode::MealType,
                                    };
                                    this.reload_data(cx);
                                })),
                        ),
                    )
                    .child(
                        div().w_48().child(
                            Select::new("select-recipe-sort-mode", sort_options)
                                .label("Sorting")
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
                    .child(Alert::new("recipes-status-alert", format!("Status: {}", self.status_msg))),
            )
            // Split Master-Detail View
            .child(
                div()
                    .flex()
                    .gap_4()
                    .flex_1()
                    // Recipes List Sidebar with Sticky Section Headers
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
                                List::new(&self.recipe_list)
                                    .flex_1()
                                    .w_full(),
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
                                                .on_click(cx.listener(move |this, _event, window, cx| {
                                                    this.open_make_recipe_modal(&recipe_clone, window, cx);
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
    }
}
