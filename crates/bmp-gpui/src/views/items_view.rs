use crate::components::*;
use bmp_domain::*;
use bmp_services::AppServices;
use gpui::prelude::*;
use gpui::*;
use gpui_component::WindowExt;
use gpui_component::alert::Alert;
use gpui_component::badge::Badge;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::checkbox::Checkbox;
use gpui_component::dialog::{DialogDescription, DialogFooter, DialogHeader, DialogTitle};
use gpui_component::list::{List, ListDelegate, ListItem, ListState};
use gpui_component::tag::Tag;
use gpui_component::{ActiveTheme, IndexPath, Selectable};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemGroupMode {
    Category,
    PurchaseMode,
    DensityStatus,
    StoreCoverage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemSortMode {
    NameAsc,
    NameDesc,
    DensityDesc,
}

pub struct ItemSection {
    pub title: String,
    pub items: Vec<Item>,
}

#[derive(IntoElement)]
pub struct DomainListItem {
    pub base: ListItem,
    pub item: Item,
    pub selected: bool,
    pub view: WeakEntity<ItemsView>,
}

impl Selectable for DomainListItem {
    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    fn is_selected(&self) -> bool {
        self.selected
    }
}

impl RenderOnce for DomainListItem {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let density_str = self
            .item
            .density
            .map(|d| format!("{} g/ml", d.g_per_ml.normalize()))
            .unwrap_or_else(|| "Missing Density".to_string());
        let mode_str = format!("{:?}", self.item.preferred_purchase_mode);
        let item_clone = self.item.clone();
        let item_id = self.item.id;
        let flags = self.item.dietary_flags.clone();
        let view_edit = self.view.clone();
        let view_delete = self.view.clone();

        self.base.py_2().px_2().rounded_md().child(
            div()
                .flex()
                .justify_between()
                .items_center()
                .w_full()
                // Name & Category & Dietary Flags
                .child(
                    div()
                        .w_1_4()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_sm()
                                .text_color(cx.theme().foreground)
                                .child(self.item.name.clone()),
                        )
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(
                                    self.item
                                        .category
                                        .clone()
                                        .unwrap_or_else(|| "Uncategorized".to_string()),
                                ),
                        )
                        .when(!flags.is_empty(), |this| {
                            this.child(
                                div().flex().gap_1().mt_1().children(
                                    flags.into_iter().map(|f| Tag::new().child(f.as_str())),
                                ),
                            )
                        }),
                )
                // Density
                .child(div().w_1_4().child(Tag::new().child(density_str)))
                // Purchase Mode
                .child(div().w_1_4().child(Tag::new().child(mode_str)))
                // Action Buttons
                .child(
                    div()
                        .w_1_4()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(
                            Button::new(format!("btn-edit-{}", item_id))
                                .secondary()
                                .label("Edit")
                                .on_click(move |_, window, cx| {
                                    if let Some(view) = view_edit.upgrade() {
                                        view.update(cx, |this, cx| {
                                            this.open_edit_item_modal(&item_clone, window, cx);
                                        });
                                    }
                                }),
                        )
                        .child(
                            Button::new(format!("btn-delete-{}", item_id))
                                .ghost()
                                .label("🗑")
                                .on_click(move |_, _, cx| {
                                    if let Some(view) = view_delete.upgrade() {
                                        view.update(cx, |this, cx| {
                                            this.delete_item(item_id, cx);
                                        });
                                    }
                                }),
                        ),
                ),
        )
    }
}

pub struct ItemListDelegate {
    pub items: Vec<Item>,
    pub sections: Vec<ItemSection>,
    pub selected_index: Option<IndexPath>,
    pub query: String,
    pub group_mode: ItemGroupMode,
    pub sort_mode: ItemSortMode,
    pub dietary_filter: Option<DietaryFlag>,
    pub view: WeakEntity<ItemsView>,
}

impl ItemListDelegate {
    pub fn prepare(&mut self, query: String) {
        self.query = query;
        let q = self.query.to_lowercase();

        let filtered: Vec<Item> = self
            .items
            .iter()
            .filter(|item| {
                if !q.is_empty() {
                    let matches_name = item.name.to_lowercase().contains(&q);
                    let matches_cat = item
                        .category
                        .as_ref()
                        .map(|c| c.to_lowercase().contains(&q))
                        .unwrap_or(false);
                    if !matches_name && !matches_cat {
                        return false;
                    }
                }
                if let Some(flag) = self.dietary_filter {
                    if !item.dietary_flags.contains(&flag) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect();

        let mut groups: BTreeMap<String, Vec<Item>> = BTreeMap::new();
        for item in filtered {
            let key = match self.group_mode {
                ItemGroupMode::Category => item
                    .category
                    .clone()
                    .unwrap_or_else(|| "Uncategorized".to_string()),
                ItemGroupMode::PurchaseMode => format!("{:?}", item.preferred_purchase_mode),
                ItemGroupMode::DensityStatus => {
                    if item.density.is_some() {
                        "Configured Density (g/ml)".to_string()
                    } else {
                        "Missing Density".to_string()
                    }
                }
                ItemGroupMode::StoreCoverage => "All Domain Ingredients".to_string(),
            };
            groups.entry(key).or_default().push(item);
        }

        self.sections = groups
            .into_iter()
            .map(|(title, mut items)| {
                match self.sort_mode {
                    ItemSortMode::NameAsc => {
                        items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
                    }
                    ItemSortMode::NameDesc => {
                        items.sort_by(|a, b| b.name.to_lowercase().cmp(&a.name.to_lowercase()))
                    }
                    ItemSortMode::DensityDesc => items.sort_by(|a, b| {
                        let da = a.density.map(|d| d.g_per_ml).unwrap_or(Decimal::ZERO);
                        let db = b.density.map(|d| d.g_per_ml).unwrap_or(Decimal::ZERO);
                        db.cmp(&da)
                    }),
                }
                ItemSection { title, items }
            })
            .collect();
    }
}

impl ListDelegate for ItemListDelegate {
    type Item = DomainListItem;

    fn sections_count(&self, _: &App) -> usize {
        self.sections.len()
    }

    fn items_count(&self, section: usize, _: &App) -> usize {
        self.sections
            .get(section)
            .map(|s| s.items.len())
            .unwrap_or(0)
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
                        .child(format!("{} items", sec.items.len())),
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
        let selected_item_id = ix.and_then(|idx| {
            self.sections
                .get(idx.section)
                .and_then(|s| s.items.get(idx.row).map(|i| i.id))
        });

        if let Some(view) = self.view.upgrade() {
            view.update(cx, |this, cx| {
                this.selected_item_id = selected_item_id;
                this.reload_packages(cx);
                cx.notify();
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
        let item = self.sections.get(ix.section)?.items.get(ix.row)?;
        Some(DomainListItem {
            base: ListItem::new(format!("item-row-{}", item.id)).selected(selected),
            item: item.clone(),
            selected,
            view: self.view.clone(),
        })
    }
}

pub struct ItemsView {
    pub services: AppServices,
    pub items_list: Entity<ListState<ItemListDelegate>>,
    pub status_msg: String,

    pub cached_items: Vec<Item>,
    pub cached_stores: Vec<Store>,
    pub cached_packages: Vec<Package>,
    pub selected_item_id: Option<ItemId>,

    // Item Form State
    pub editing_item_id: Option<ItemId>,
    pub item_form_name: String,
    pub item_form_density: Decimal,
    pub item_form_category: String,
    pub item_form_mode: PurchaseMode,
    pub item_form_dietary_flags: HashSet<DietaryFlag>,
    pub item_form_calories: Option<Decimal>,
    pub item_form_protein: Option<Decimal>,
    pub item_form_carbs: Option<Decimal>,
    pub item_form_fat: Option<Decimal>,
    pub item_form_fiber: Option<Decimal>,
    pub item_form_sodium: Option<Decimal>,

    // Package Form State
    pub pkg_form_item_id: Option<ItemId>,
    pub pkg_form_store_id: Option<StoreId>,
    pub pkg_form_amount: Decimal,
    pub pkg_form_unit: Unit,
    pub pkg_form_price: Decimal,
    pub pkg_form_preferred: bool,

    // Store Form State
    pub store_form_name: String,

    // Bridge Form State
    pub bridge_form_item_id: Option<ItemId>,
    pub bridge_from_amount: Decimal,
    pub bridge_from_unit: Unit,
    pub bridge_to_amount: Decimal,
    pub bridge_to_unit: Unit,

    // Grouping & Sorting state
    pub group_mode: ItemGroupMode,
    pub sort_mode: ItemSortMode,
    pub dietary_filter: Option<DietaryFlag>,
}

impl ItemsView {
    pub fn new(services: AppServices, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let view = cx.entity().downgrade();
        let delegate = ItemListDelegate {
            items: Vec::new(),
            sections: Vec::new(),
            selected_index: None,
            query: String::new(),
            group_mode: ItemGroupMode::Category,
            sort_mode: ItemSortMode::NameAsc,
            dietary_filter: None,
            view,
        };

        let items_list = cx.new(|cx| ListState::new(delegate, window, cx).searchable(true));

        let mut view_state = Self {
            services,
            items_list,
            status_msg: "Items matrix ready".to_string(),
            cached_items: Vec::new(),
            cached_stores: Vec::new(),
            cached_packages: Vec::new(),
            selected_item_id: None,

            editing_item_id: None,
            item_form_name: String::new(),
            item_form_density: dec!(1.0),
            item_form_category: "Pantry".to_string(),
            item_form_mode: PurchaseMode::BuyFinished,
            item_form_dietary_flags: HashSet::new(),
            item_form_calories: None,
            item_form_protein: None,
            item_form_carbs: None,
            item_form_fat: None,
            item_form_fiber: None,
            item_form_sodium: None,

            pkg_form_item_id: None,
            pkg_form_store_id: None,
            pkg_form_amount: dec!(500),
            pkg_form_unit: Unit::Gram,
            pkg_form_price: dec!(4.99),
            pkg_form_preferred: true,

            store_form_name: String::new(),

            bridge_form_item_id: None,
            bridge_from_amount: dec!(1),
            bridge_from_unit: Unit::Each,
            bridge_to_amount: dec!(150),
            bridge_to_unit: Unit::Gram,

            group_mode: ItemGroupMode::Category,
            sort_mode: ItemSortMode::NameAsc,
            dietary_filter: None,
        };

        view_state.reload_data(cx);
        view_state
    }

    pub fn reload_data(&mut self, cx: &mut Context<Self>) {
        self.cached_items = self.services.items.list_items().unwrap_or_default();
        self.cached_stores = self.services.items.list_stores().unwrap_or_default();
        self.reload_packages(cx);

        let items_list = self.items_list.clone();
        let cached_items = self.cached_items.clone();
        let group_mode = self.group_mode;
        let sort_mode = self.sort_mode;
        let dietary_filter = self.dietary_filter;

        cx.defer(move |cx| {
            items_list.update(cx, |list, cx| {
                list.delegate_mut().items = cached_items;
                list.delegate_mut().group_mode = group_mode;
                list.delegate_mut().sort_mode = sort_mode;
                list.delegate_mut().dietary_filter = dietary_filter;

                let query = list.delegate().query.clone();
                list.delegate_mut().prepare(query);
                cx.notify();
            });
        });
        cx.notify();
    }

    pub fn reload_packages(&mut self, cx: &mut Context<Self>) {
        if let Some(id) = self.selected_item_id {
            self.cached_packages = self
                .services
                .items
                .get_packages_for_item(id)
                .unwrap_or_default();
        } else {
            self.cached_packages.clear();
        }
        cx.notify();
    }

    pub fn open_create_item_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.editing_item_id = None;
        self.item_form_name = String::new();
        self.item_form_density = dec!(1.0);
        self.item_form_category = "General".to_string();
        self.item_form_mode = PurchaseMode::BuyFinished;
        self.item_form_dietary_flags = HashSet::new();
        self.item_form_calories = None;
        self.item_form_protein = None;
        self.item_form_carbs = None;
        self.item_form_fat = None;
        self.item_form_fiber = None;
        self.item_form_sodium = None;
        self.show_item_dialog(window, cx);
    }

    pub fn open_edit_item_modal(
        &mut self,
        item: &Item,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.editing_item_id = Some(item.id);
        self.item_form_name = item.name.clone();
        self.item_form_density = item.density.map(|d| d.g_per_ml).unwrap_or(dec!(1.0));
        self.item_form_category = item
            .category
            .clone()
            .unwrap_or_else(|| "General".to_string());
        self.item_form_mode = item.preferred_purchase_mode;
        self.item_form_dietary_flags = item.dietary_flags.iter().cloned().collect();
        if let Some(nut) = &item.nutrition {
            self.item_form_calories = nut.calories;
            self.item_form_protein = nut.protein_g;
            self.item_form_carbs = nut.net_carbs_g;
            self.item_form_fat = nut.fat_g;
            self.item_form_fiber = nut.fiber_g;
            self.item_form_sodium = nut.sodium_mg;
        } else {
            self.item_form_calories = None;
            self.item_form_protein = None;
            self.item_form_carbs = None;
            self.item_form_fat = None;
            self.item_form_fiber = None;
            self.item_form_sodium = None;
        }
        self.show_item_dialog(window, cx);
    }

    fn show_item_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let view = cx.entity().clone();
        window.open_dialog(cx, move |dialog, _, _| {
            let view = view.clone();
            dialog.w(px(550.)).content(move |content, _, cx| {
                let view_read = view.read(cx);
                let is_edit = view_read.editing_item_id.is_some();
                let title = if is_edit {
                    "Edit Domain Item"
                } else {
                    "Add Domain Item"
                };

                let mode_options = vec![
                    SelectOption::new("BuyFinished", "Buy Finished Package"),
                    SelectOption::new("PreferMake", "Prefer Make / Expand"),
                    SelectOption::new("AskEveryTime", "Ask Every Time"),
                ];

                let form_name = view_read.item_form_name.clone();
                let form_category = view_read.item_form_category.clone();
                let form_density = view_read.item_form_density;
                let form_mode = view_read.item_form_mode;
                let form_flags = view_read.item_form_dietary_flags.clone();

                let form_cal = view_read.item_form_calories.unwrap_or(Decimal::ZERO);
                let form_prot = view_read.item_form_protein.unwrap_or(Decimal::ZERO);
                let form_carb = view_read.item_form_carbs.unwrap_or(Decimal::ZERO);
                let form_fat = view_read.item_form_fat.unwrap_or(Decimal::ZERO);

                let v_num = view.clone();
                let v_mode = view.clone();
                let v_save = view.clone();
                let v_flags = view.clone();
                let v_cal = view.clone();
                let v_prot = view.clone();
                let v_carb = view.clone();
                let v_fat = view.clone();

                let common_flags = DietaryFlag::all().to_vec();

                content
                    .child(
                        DialogHeader::new()
                            .child(DialogTitle::new().child(title))
                            .child(DialogDescription::new().child(
                                "Configure density (g/ml), category, dietary flags, and nutrition",
                            )),
                    )
                    .child(
                        div()
                            .py_4()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .child(
                                FormInput::new("input-item-name")
                                    .label("Item Name")
                                    .placeholder("e.g. Extra Virgin Olive Oil")
                                    .value(form_name),
                            )
                            .child(
                                FormInput::new("input-item-category")
                                    .label("Category")
                                    .placeholder("e.g. Oils & Fats, Pantry, Dairy")
                                    .value(form_category),
                            )
                            .child(
                                NumberInput::new("input-item-density", form_density)
                                    .label("Density (g/ml)")
                                    .step(dec!(0.05))
                                    .unit("g/ml")
                                    .on_increment({
                                        let v = v_num.clone();
                                        move |val, _window, cx| {
                                            v.update(cx, |this, cx| {
                                                this.item_form_density = *val;
                                                cx.notify();
                                            });
                                        }
                                    })
                                    .on_decrement({
                                        let v = v_num.clone();
                                        move |val, _window, cx| {
                                            v.update(cx, |this, cx| {
                                                this.item_form_density = *val;
                                                cx.notify();
                                            });
                                        }
                                    }),
                            )
                            .child(
                                Select::new("select-item-purchase-mode", mode_options)
                                    .label("Preferred Purchase Mode")
                                    .selected_id(Some(format!("{:?}", form_mode)))
                                    .on_select(move |opt: &SelectOption, _window, cx| {
                                        let mode = match opt.id.as_str() {
                                            "PreferMake" => PurchaseMode::PreferMake,
                                            "AskEveryTime" => PurchaseMode::AskEveryTime,
                                            _ => PurchaseMode::BuyFinished,
                                        };
                                        v_mode.update(cx, |this, cx| {
                                            this.item_form_mode = mode;
                                            cx.notify();
                                        });
                                    }),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::BOLD)
                                            .child("Dietary Tags"),
                                    )
                                    .child(div().flex().flex_wrap().gap_2().children(
                                        common_flags.into_iter().map(|flag| {
                                            let is_checked = form_flags.contains(&flag);
                                            let vf = v_flags.clone();
                                            Checkbox::new(format!("cb-flag-{}", flag.as_str()))
                                                .label(flag.as_str())
                                                .checked(is_checked)
                                                .on_click(move |checked, _window, cx| {
                                                    let f = flag.clone();
                                                    let is_set = *checked;
                                                    vf.update(cx, |this, cx| {
                                                        if is_set {
                                                            this.item_form_dietary_flags.insert(f);
                                                        } else {
                                                            this.item_form_dietary_flags.remove(&f);
                                                        }
                                                        cx.notify();
                                                    });
                                                })
                                        }),
                                    )),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_xs()
                                            .font_weight(FontWeight::BOLD)
                                            .child("Nutritional Info (Optional per 100g)"),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .gap_2()
                                            .child(
                                                NumberInput::new("input-cal", form_cal)
                                                    .label("Calories")
                                                    .step(dec!(10))
                                                    .on_increment({
                                                        let v = v_cal.clone();
                                                        move |val, _, cx| {
                                                            v.update(cx, |this, cx| {
                                                                this.item_form_calories =
                                                                    Some(*val);
                                                                cx.notify();
                                                            })
                                                        }
                                                    })
                                                    .on_decrement({
                                                        let v = v_cal.clone();
                                                        move |val, _, cx| {
                                                            v.update(cx, |this, cx| {
                                                                this.item_form_calories =
                                                                    Some(*val);
                                                                cx.notify();
                                                            })
                                                        }
                                                    }),
                                            )
                                            .child(
                                                NumberInput::new("input-prot", form_prot)
                                                    .label("Protein (g)")
                                                    .step(dec!(1))
                                                    .on_increment({
                                                        let v = v_prot.clone();
                                                        move |val, _, cx| {
                                                            v.update(cx, |this, cx| {
                                                                this.item_form_protein = Some(*val);
                                                                cx.notify();
                                                            })
                                                        }
                                                    })
                                                    .on_decrement({
                                                        let v = v_prot.clone();
                                                        move |val, _, cx| {
                                                            v.update(cx, |this, cx| {
                                                                this.item_form_protein = Some(*val);
                                                                cx.notify();
                                                            })
                                                        }
                                                    }),
                                            )
                                            .child(
                                                NumberInput::new("input-carb", form_carb)
                                                    .label("Carbs (g)")
                                                    .step(dec!(1))
                                                    .on_increment({
                                                        let v = v_carb.clone();
                                                        move |val, _, cx| {
                                                            v.update(cx, |this, cx| {
                                                                this.item_form_carbs = Some(*val);
                                                                cx.notify();
                                                            })
                                                        }
                                                    })
                                                    .on_decrement({
                                                        let v = v_carb.clone();
                                                        move |val, _, cx| {
                                                            v.update(cx, |this, cx| {
                                                                this.item_form_carbs = Some(*val);
                                                                cx.notify();
                                                            })
                                                        }
                                                    }),
                                            )
                                            .child(
                                                NumberInput::new("input-fat", form_fat)
                                                    .label("Fat (g)")
                                                    .step(dec!(1))
                                                    .on_increment({
                                                        let v = v_fat.clone();
                                                        move |val, _, cx| {
                                                            v.update(cx, |this, cx| {
                                                                this.item_form_fat = Some(*val);
                                                                cx.notify();
                                                            })
                                                        }
                                                    })
                                                    .on_decrement({
                                                        let v = v_fat.clone();
                                                        move |val, _, cx| {
                                                            v.update(cx, |this, cx| {
                                                                this.item_form_fat = Some(*val);
                                                                cx.notify();
                                                            })
                                                        }
                                                    }),
                                            ),
                                    ),
                            ),
                    )
                    .child(
                        DialogFooter::new()
                            .child(
                                Button::new("btn-cancel-item-modal")
                                    .secondary()
                                    .label("Cancel")
                                    .on_click(|_, window, cx| {
                                        window.close_dialog(cx);
                                    }),
                            )
                            .child(
                                Button::new("btn-save-item-modal")
                                    .primary()
                                    .label("Save Item")
                                    .on_click(move |_, window, cx| {
                                        v_save.update(cx, |this, cx| {
                                            this.save_item(cx);
                                        });
                                        window.close_dialog(cx);
                                    }),
                            ),
                    )
            })
        });
    }

    pub fn save_item(&mut self, cx: &mut Context<Self>) {
        if self.item_form_name.trim().is_empty() {
            self.status_msg = "Error: Item name cannot be empty".to_string();
            cx.notify();
            return;
        }

        let density_opt = if self.item_form_density > Decimal::ZERO {
            Some(self.item_form_density)
        } else {
            None
        };

        let nutrition = if self.item_form_calories.is_some() || self.item_form_protein.is_some() {
            Some(NutritionalInfo {
                calories: self.item_form_calories,
                protein_g: self.item_form_protein,
                net_carbs_g: self.item_form_carbs,
                fat_g: self.item_form_fat,
                fiber_g: self.item_form_fiber,
                sodium_mg: self.item_form_sodium,
            })
        } else {
            None
        };

        if let Some(item_id) = self.editing_item_id {
            if let Ok(mut items) = self.services.items.list_items() {
                if let Some(item) = items.iter_mut().find(|i| i.id == item_id) {
                    item.name = self.item_form_name.trim().to_string();
                    item.category = Some(self.item_form_category.trim().to_string());
                    item.preferred_purchase_mode = self.item_form_mode;
                    item.dietary_flags = self.item_form_dietary_flags.iter().cloned().collect();
                    item.nutrition = nutrition;
                    if let Some(d) = density_opt {
                        if let Ok(den) = Density::new(d) {
                            item.density = Some(den);
                        }
                    } else {
                        item.density = None;
                    }

                    match self.services.items.update_item(item) {
                        Ok(_) => {
                            self.status_msg = format!("Updated item: {}", item.name);
                        }
                        Err(e) => {
                            self.status_msg = format!("Error updating item: {}", e);
                        }
                    }
                }
            }
        } else {
            match self.services.items.create_item(
                self.item_form_name.trim(),
                density_opt,
                Some(self.item_form_category.trim()),
            ) {
                Ok(mut item) => {
                    item.preferred_purchase_mode = self.item_form_mode;
                    item.dietary_flags = self.item_form_dietary_flags.iter().cloned().collect();
                    item.nutrition = nutrition;
                    let _ = self.services.items.update_item(&item);
                    self.status_msg = format!("Created item: {}", item.name);
                }
                Err(e) => {
                    self.status_msg = format!("Error creating item: {}", e);
                }
            }
        }
        self.reload_data(cx);
    }

    pub fn delete_item(&mut self, item_id: ItemId, cx: &mut Context<Self>) {
        match self.services.items.delete_item(item_id) {
            Ok(_) => {
                self.status_msg = "Deleted item successfully".to_string();
                if self.selected_item_id == Some(item_id) {
                    self.selected_item_id = None;
                }
            }
            Err(e) => {
                self.status_msg = format!("Error deleting item: {}", e);
            }
        }
        self.reload_data(cx);
    }

    pub fn open_add_store_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.store_form_name = String::new();
        let view = cx.entity().clone();
        window.open_dialog(cx, move |dialog, _, _| {
            let view = view.clone();
            dialog.w(px(400.)).content(move |content, _, cx| {
                let view_read = view.read(cx);
                let form_name = view_read.store_form_name.clone();
                let v_save = view.clone();

                content
                    .child(
                        DialogHeader::new()
                            .child(DialogTitle::new().child("Register Store"))
                            .child(
                                DialogDescription::new()
                                    .child("Add local grocery store for package tracking"),
                            ),
                    )
                    .child(
                        div().py_4().child(
                            FormInput::new("input-store-name")
                                .label("Store Name")
                                .placeholder("e.g. Trader Joe's, Costco")
                                .value(form_name),
                        ),
                    )
                    .child(
                        DialogFooter::new()
                            .child(
                                Button::new("btn-cancel-store-modal")
                                    .secondary()
                                    .label("Cancel")
                                    .on_click(|_, window, cx| {
                                        window.close_dialog(cx);
                                    }),
                            )
                            .child(
                                Button::new("btn-save-store-modal")
                                    .primary()
                                    .label("Save Store")
                                    .on_click(move |_, window, cx| {
                                        v_save.update(cx, |this, cx| {
                                            this.save_store(cx);
                                        });
                                        window.close_dialog(cx);
                                    }),
                            ),
                    )
            })
        });
    }

    pub fn save_store(&mut self, cx: &mut Context<Self>) {
        if self.store_form_name.trim().is_empty() {
            self.status_msg = "Error: Store name cannot be empty".to_string();
            cx.notify();
            return;
        }
        match self.services.items.add_store(self.store_form_name.trim()) {
            Ok(store) => {
                self.status_msg = format!("Registered store: {}", store.name);
            }
            Err(e) => {
                self.status_msg = format!("Error registering store: {}", e);
            }
        }
        self.reload_data(cx);
    }
}

impl Render for ItemsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let stores = self.cached_stores.clone();
        let selected_item = self
            .selected_item_id
            .and_then(|id| self.cached_items.iter().find(|i| i.id == id).cloned());
        let _has_selected_item = selected_item.is_some();
        let _selected_item_packages = self.cached_packages.clone();

        let group_options = vec![
            SelectOption::new("Category", "Group by Category"),
            SelectOption::new("PurchaseMode", "Group by Purchase Mode"),
            SelectOption::new("DensityStatus", "Group by Density Status"),
        ];

        let sort_options = vec![
            SelectOption::new("NameAsc", "Sort A-Z"),
            SelectOption::new("NameDesc", "Sort Z-A"),
            SelectOption::new("DensityDesc", "Sort by Density (g/ml)"),
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
                                    .child("Domain Items & Density Matrix"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Manage physical conversion densities, store packages, and mass-per-each bridges"),
                            ),
                    )
                    .child(Alert::new("items-status-alert", format!("Status: {}", self.status_msg))),
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
                                Select::new("select-item-group-mode", group_options)
                                    .selected_id(Some(format!("{:?}", self.group_mode)))
                                    .on_select(cx.listener(|this, opt: &SelectOption, _window, cx| {
                                        this.group_mode = match opt.id.as_str() {
                                            "PurchaseMode" => ItemGroupMode::PurchaseMode,
                                            "DensityStatus" => ItemGroupMode::DensityStatus,
                                            _ => ItemGroupMode::Category,
                                        };
                                        this.reload_data(cx);
                                    })),
                            )
                            .child(
                                Select::new("select-item-sort-mode", sort_options)
                                    .selected_id(Some(format!("{:?}", self.sort_mode)))
                                    .on_select(cx.listener(|this, opt: &SelectOption, _window, cx| {
                                        this.sort_mode = match opt.id.as_str() {
                                            "NameDesc" => ItemSortMode::NameDesc,
                                            "DensityDesc" => ItemSortMode::DensityDesc,
                                            _ => ItemSortMode::NameAsc,
                                        };
                                        this.reload_data(cx);
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::new("btn-open-create-item")
                                    .primary()
                                    .label("+ Add Ingredient / Item")
                                    .on_click(cx.listener(|this, _event, window, cx| {
                                        this.open_create_item_modal(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("btn-open-add-store")
                                    .secondary()
                                    .label("+ Add Store")
                                    .on_click(cx.listener(|this, _event, window, cx| {
                                        this.open_add_store_modal(window, cx);
                                    })),
                            ),
                    ),
            )
            // Main Grid Layout: Items Virtualized List on Left, Detail / Package Panel on Right
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
                            .child(List::new(&self.items_list)),
                    )
                    .child(
                        div()
                            .w_1_2()
                            .flex()
                            .flex_col()
                            .gap_4()
                            .child(
                                if let Some(item) = selected_item {
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
                                                        .child(format!("Selected: {}", item.name)),
                                                )
                                                .child(Badge::new().child(format!("{:?}", item.preferred_purchase_mode))),
                                        )
                                        .child(
                                            div()
                                                .text_xs()
                                                .text_color(cx.theme().muted_foreground)
                                                .child(format!(
                                                    "Density: {} | Category: {}",
                                                    item.density.map(|d| format!("{} g/ml", d.g_per_ml.normalize())).unwrap_or_else(|| "Not set".to_string()),
                                                    item.category.as_deref().unwrap_or("Uncategorized")
                                                )),
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
                                                .child("Select an ingredient from the list to view and configure store packages or count bridges."),
                                        )
                                },
                            )
                            // Registered Stores Summary
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .p_4()
                                    .bg(cx.theme().background)
                                    .border_1()
                                    .border_color(cx.theme().border)
                                    .rounded_xl()
                                    .child(div().text_sm().font_weight(FontWeight::BOLD).child("Registered Grocery Stores"))
                                    .child(
                                        if stores.is_empty() {
                                            div().text_xs().text_color(cx.theme().muted_foreground).child("No stores added yet. Click '+ Add Store' above.")
                                        } else {
                                            div()
                                                .flex()
                                                .flex_wrap()
                                                .gap_2()
                                                .children(stores.into_iter().map(|s| {
                                                    Badge::new().child(s.name)
                                                }))
                                        }
                                    ),
                            ),
                    ),
            )
    }
}
