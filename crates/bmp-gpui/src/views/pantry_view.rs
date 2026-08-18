use crate::components::*;
use bmp_domain::*;
use bmp_services::AppServices;
use chrono::{Local, NaiveDate};
use gpui::prelude::*;
use gpui::*;
use gpui_component::WindowExt;
use gpui_component::alert::Alert;
use gpui_component::badge::Badge;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::dialog::{DialogDescription, DialogFooter, DialogHeader, DialogTitle};
use gpui_component::list::{List, ListDelegate, ListItem, ListState};
use gpui_component::tag::Tag;
use gpui_component::{ActiveTheme, IndexPath, Selectable};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PantryGroupMode {
    ExpirationStatus,
    Category,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PantrySortMode {
    ExpirationDate,
    NameAsc,
    QuantityDesc,
}

pub struct PantrySection {
    pub title: String,
    pub entries: Vec<PantryEntry>,
}

#[derive(IntoElement)]
pub struct PantryListItem {
    pub base: ListItem,
    pub entry: PantryEntry,
    pub item_name: String,
    pub selected: bool,
    pub view: WeakEntity<PantryView>,
}

impl Selectable for PantryListItem {
    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    fn is_selected(&self) -> bool {
        self.selected
    }
}

impl RenderOnce for PantryListItem {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let entry_id = self.entry.id;
        let today = Local::now().date_naive();
        let (exp_str, exp_status) = match self.entry.expiration {
            Some(exp) => {
                let days = (exp - today).num_days();
                if days < 0 {
                    (format!("Expired ({})", exp), "Expired")
                } else if days <= 3 {
                    (format!("Expires in {}d ({})", days, exp), "Expires Soon")
                } else {
                    (format!("Expires {}", exp), "Fresh")
                }
            }
            None => ("No Expiration Date".to_string(), "Fresh"),
        };

        let view_dec = self.view.clone();
        let view_inc = self.view.clone();
        let view_del = self.view.clone();

        self.base.py_2().px_2().rounded_md().child(
            div()
                .flex()
                .items_center()
                .justify_between()
                .w_full()
                .child(
                    div()
                        .w_1_4()
                        .font_weight(FontWeight::BOLD)
                        .text_sm()
                        .text_color(cx.theme().foreground)
                        .child(self.item_name),
                )
                .child(div().w_1_4().child(Tag::new().child(format!(
                    "{} {}",
                    self.entry.quantity.amount.normalize(),
                    self.entry.quantity.unit
                ))))
                .child(
                    div()
                        .w_1_4()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(Badge::new().child(exp_status))
                        .child(
                            div()
                                .text_xs()
                                .text_color(cx.theme().muted_foreground)
                                .child(exp_str),
                        ),
                )
                .child(
                    div()
                        .w_1_4()
                        .flex()
                        .items_center()
                        .gap_1()
                        .child(
                            Button::new(format!("btn-dec-pantry-{}", entry_id))
                                .secondary()
                                .label("- 100")
                                .on_click(move |_, _, cx| {
                                    if let Some(view) = view_dec.upgrade() {
                                        view.update(cx, |this, cx| {
                                            this.update_quantity(entry_id, dec!(-100), cx);
                                        });
                                    }
                                }),
                        )
                        .child(
                            Button::new(format!("btn-inc-pantry-{}", entry_id))
                                .secondary()
                                .label("+ 100")
                                .on_click(move |_, _, cx| {
                                    if let Some(view) = view_inc.upgrade() {
                                        view.update(cx, |this, cx| {
                                            this.update_quantity(entry_id, dec!(100), cx);
                                        });
                                    }
                                }),
                        )
                        .child(
                            Button::new(format!("btn-del-pantry-{}", entry_id))
                                .ghost()
                                .label("🗑")
                                .on_click(move |_, _, cx| {
                                    if let Some(view) = view_del.upgrade() {
                                        view.update(cx, |this, cx| {
                                            this.delete_entry(entry_id, cx);
                                        });
                                    }
                                }),
                        ),
                ),
        )
    }
}

pub struct PantryListDelegate {
    pub entries: Vec<PantryEntry>,
    pub items: Vec<Item>,
    pub sections: Vec<PantrySection>,
    pub selected_index: Option<IndexPath>,
    pub query: String,
    pub group_mode: PantryGroupMode,
    pub sort_mode: PantrySortMode,
    pub view: WeakEntity<PantryView>,
}

impl PantryListDelegate {
    pub fn prepare(&mut self, query: String) {
        self.query = query;
        let q = self.query.to_lowercase();
        let today = Local::now().date_naive();

        let items_map: HashMap<ItemId, Item> =
            self.items.iter().map(|i| (i.id, i.clone())).collect();

        let filtered: Vec<PantryEntry> = self
            .entries
            .iter()
            .filter(|e| {
                if q.is_empty() {
                    true
                } else {
                    items_map
                        .get(&e.item_id)
                        .map(|item| item.name.to_lowercase().contains(&q))
                        .unwrap_or(false)
                }
            })
            .cloned()
            .collect();

        let mut groups: BTreeMap<String, Vec<PantryEntry>> = BTreeMap::new();
        for entry in filtered {
            let key = match self.group_mode {
                PantryGroupMode::ExpirationStatus => match entry.expiration {
                    Some(exp) => {
                        let days = (exp - today).num_days();
                        if days < 0 {
                            "1. Expired".to_string()
                        } else if days <= 3 {
                            "2. Expires Soon (≤ 3 Days)".to_string()
                        } else {
                            "3. Fresh Inventory".to_string()
                        }
                    }
                    None => "4. No Expiration Date".to_string(),
                },
                PantryGroupMode::Category => items_map
                    .get(&entry.item_id)
                    .and_then(|i| i.category.clone())
                    .unwrap_or_else(|| "Uncategorized".to_string()),
            };
            groups.entry(key).or_default().push(entry);
        }

        self.sections = groups
            .into_iter()
            .map(|(title, mut entries)| {
                match self.sort_mode {
                    PantrySortMode::ExpirationDate => entries.sort_by(|a, b| {
                        let ea = a.expiration.unwrap_or(NaiveDate::MAX);
                        let eb = b.expiration.unwrap_or(NaiveDate::MAX);
                        ea.cmp(&eb)
                    }),
                    PantrySortMode::NameAsc => entries.sort_by(|a, b| {
                        let na = items_map
                            .get(&a.item_id)
                            .map(|i| i.name.as_str())
                            .unwrap_or("");
                        let nb = items_map
                            .get(&b.item_id)
                            .map(|i| i.name.as_str())
                            .unwrap_or("");
                        na.to_lowercase().cmp(&nb.to_lowercase())
                    }),
                    PantrySortMode::QuantityDesc => {
                        entries.sort_by(|a, b| b.quantity.amount.cmp(&a.quantity.amount))
                    }
                }
                PantrySection { title, entries }
            })
            .collect();
    }
}

impl ListDelegate for PantryListDelegate {
    type Item = PantryListItem;

    fn sections_count(&self, _: &App) -> usize {
        self.sections.len()
    }

    fn items_count(&self, section: usize, _: &App) -> usize {
        self.sections
            .get(section)
            .map(|s| s.entries.len())
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
                        .child(format!("{} entries", sec.entries.len())),
                ),
        )
    }

    fn set_selected_index(
        &mut self,
        ix: Option<IndexPath>,
        _: &mut Window,
        _cx: &mut Context<ListState<Self>>,
    ) {
        self.selected_index = ix;
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        _: &mut Window,
        _: &mut Context<ListState<Self>>,
    ) -> Option<Self::Item> {
        let selected = Some(ix) == self.selected_index;
        let entry = self.sections.get(ix.section)?.entries.get(ix.row)?;
        let item_name = self
            .items
            .iter()
            .find(|i| i.id == entry.item_id)
            .map(|i| i.name.clone())
            .unwrap_or_else(|| "Unknown Item".to_string());

        Some(PantryListItem {
            base: ListItem::new(format!("pantry-row-{}", entry.id)).selected(selected),
            entry: entry.clone(),
            item_name,
            selected,
            view: self.view.clone(),
        })
    }
}

pub struct PantryView {
    pub services: AppServices,
    pub pantry_list: Entity<ListState<PantryListDelegate>>,
    pub status_msg: String,

    pub cached_entries: Vec<PantryEntry>,
    pub cached_items: Vec<Item>,

    // Add Stock Form State
    pub add_item_id: Option<ItemId>,
    pub add_amount: Decimal,
    pub add_unit: Unit,
    pub add_expiration: Option<NaiveDate>,
    pub add_has_expiration: bool,

    // Grouping & Sorting state
    pub group_mode: PantryGroupMode,
    pub sort_mode: PantrySortMode,

    pub group_mode_select: Entity<SelectState<Vec<SelectOption>>>,
    pub sort_mode_select: Entity<SelectState<Vec<SelectOption>>>,
}

impl PantryView {
    pub fn new(services: AppServices, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let view = cx.entity().downgrade();
        let delegate = PantryListDelegate {
            entries: Vec::new(),
            items: Vec::new(),
            sections: Vec::new(),
            selected_index: None,
            query: String::new(),
            group_mode: PantryGroupMode::ExpirationStatus,
            sort_mode: PantrySortMode::ExpirationDate,
            view,
        };

        let pantry_list = cx.new(|cx| ListState::new(delegate, window, cx).searchable(true));

        let group_options = vec![
            SelectOption::new("ExpirationStatus", "Group by Expiration Status"),
            SelectOption::new("Category", "Group by Item Category"),
        ];
        let sort_options = vec![
            SelectOption::new("ExpirationDate", "Sort by Expiration Date"),
            SelectOption::new("NameAsc", "Sort A-Z"),
            SelectOption::new("QuantityDesc", "Sort by Quantity"),
        ];
        let group_mode_select = cx.new(|cx| {
            SelectState::new(group_options, Some(IndexPath::default().row(0)), window, cx)
        });
        let sort_mode_select = cx.new(|cx| {
            SelectState::new(sort_options, Some(IndexPath::default().row(0)), window, cx)
        });

        cx.subscribe_in(
            &group_mode_select,
            window,
            |this, _, ev: &SelectEvent<_>, _window, cx| {
                if let SelectEvent::Confirm(Some(id)) = ev {
                    this.group_mode = match id.as_str() {
                        "Category" => PantryGroupMode::Category,
                        _ => PantryGroupMode::ExpirationStatus,
                    };
                    this.reload_data(cx);
                }
            },
        )
        .detach();

        cx.subscribe_in(
            &sort_mode_select,
            window,
            |this, _, ev: &SelectEvent<_>, _window, cx| {
                if let SelectEvent::Confirm(Some(id)) = ev {
                    this.sort_mode = match id.as_str() {
                        "NameAsc" => PantrySortMode::NameAsc,
                        "QuantityDesc" => PantrySortMode::QuantityDesc,
                        _ => PantrySortMode::ExpirationDate,
                    };
                    this.reload_data(cx);
                }
            },
        )
        .detach();

        let mut view_state = Self {
            services,
            pantry_list,
            status_msg: "Pantry inventory ready".to_string(),

            cached_entries: Vec::new(),
            cached_items: Vec::new(),

            add_item_id: None,
            add_amount: dec!(500),
            add_unit: Unit::Gram,
            add_expiration: Some(Local::now().date_naive() + chrono::Duration::days(14)),
            add_has_expiration: true,

            group_mode: PantryGroupMode::ExpirationStatus,
            sort_mode: PantrySortMode::ExpirationDate,
            group_mode_select,
            sort_mode_select,
        };
        view_state.reload_data(cx);
        view_state
    }

    pub fn reload_data(&mut self, cx: &mut Context<Self>) {
        self.cached_entries = self.services.pantry.get_pantry().unwrap_or_default();
        self.cached_items = self.services.items.list_items().unwrap_or_default();

        let pantry_list = self.pantry_list.clone();
        let cached_entries = self.cached_entries.clone();
        let cached_items = self.cached_items.clone();
        let group_mode = self.group_mode;
        let sort_mode = self.sort_mode;

        cx.defer(move |cx| {
            pantry_list.update(cx, |list, cx| {
                list.delegate_mut().entries = cached_entries;
                list.delegate_mut().items = cached_items;
                list.delegate_mut().group_mode = group_mode;
                list.delegate_mut().sort_mode = sort_mode;

                let query = list.delegate().query.clone();
                list.delegate_mut().prepare(query);
                cx.notify();
            });
        });
        cx.notify();
    }

    pub fn open_add_modal(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let items = &self.cached_items;
        let today = Local::now().date_naive();
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
            SelectOption::new("Ounce", "Ounce (oz)"),
            SelectOption::new("Pound", "Pound (lb)"),
            SelectOption::new("Each", "Each (count)"),
        ];

        let item_select = cx.new(|cx| {
            SelectState::new(
                item_options,
                if items.is_empty() {
                    None
                } else {
                    Some(IndexPath::default().row(0))
                },
                window,
                cx,
            )
            .searchable(true)
        });
        let unit_select = cx.new(|cx| {
            SelectState::new(unit_options, Some(IndexPath::default().row(0)), window, cx)
        });
        let exp_picker_state = cx.new(|cx| {
            let mut picker = DatePickerState::new(window, cx);
            picker.set_date(today + chrono::Duration::days(14), window, cx);
            picker
        });

        let view = cx.entity().clone();
        window.open_dialog(cx, move |dialog, _, _| {
            let view = view.clone();
            let i_in = item_select.clone();
            let u_in = unit_select.clone();
            let exp_in = exp_picker_state.clone();

            dialog.w(px(500.)).content(move |content, _, cx| {
                let view_read = view.read(cx);
                let add_amount = view_read.add_amount;
                let add_has_exp = view_read.add_has_expiration;

                let v_num_amt = view.clone();
                let v_cb_exp = view.clone();
                let v_save = view.clone();
                let i_save = i_in.clone();
                let u_save = u_in.clone();
                let exp_save = exp_in.clone();

                content
                    .child(
                        DialogHeader::new()
                            .child(DialogTitle::new().child("Add Pantry Inventory Stock"))
                            .child(
                                DialogDescription::new()
                                    .child("Deposit stock into your pantry inventory"),
                            ),
                    )
                    .child(
                        div()
                            .py_4()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .child(select_field("Item Name", Select::new(&i_in)))
                            .child(
                                NumberInput::new("input-pantry-amount", add_amount)
                                    .label("Stock Quantity Amount")
                                    .step(dec!(50))
                                    .on_increment({
                                        let v = v_num_amt.clone();
                                        move |val, _window, cx| {
                                            v.update(cx, |this, cx| {
                                                this.add_amount = *val;
                                                cx.notify();
                                            });
                                        }
                                    })
                                    .on_decrement({
                                        let v = v_num_amt.clone();
                                        move |val, _window, cx| {
                                            v.update(cx, |this, cx| {
                                                this.add_amount = *val;
                                                cx.notify();
                                            });
                                        }
                                    }),
                            )
                            .child(select_field("Unit", Select::new(&u_in)))
                            .child(
                                Checkbox::new("cb-has-expiration")
                                    .label("Set Expiration Date")
                                    .checked(add_has_exp)
                                    .on_click(move |checked, _window, cx| {
                                        v_cb_exp.update(cx, |this, cx| {
                                            this.add_has_expiration = *checked;
                                            cx.notify();
                                        });
                                    }),
                            )
                            .when(add_has_exp, |this| {
                                this.child(date_picker_field(
                                    "Expiration Date",
                                    DatePicker::new(&exp_in),
                                ))
                            }),
                    )
                    .child(
                        DialogFooter::new()
                            .child(
                                Button::new("btn-cancel-pantry-modal")
                                    .secondary()
                                    .label("Cancel")
                                    .on_click(|_, window, cx| {
                                        window.close_dialog(cx);
                                    }),
                            )
                            .child(
                                Button::new("btn-save-pantry-modal")
                                    .primary()
                                    .label("Deposit to Pantry")
                                    .on_click(move |_, window, cx| {
                                        let i_uuid = i_save
                                            .read(cx)
                                            .selected_value()
                                            .and_then(|s| uuid::Uuid::from_str(&s).ok())
                                            .map(ItemId);
                                        let u_str = u_save.read(cx).selected_value().cloned();
                                        let unit = match u_str.as_deref() {
                                            Some("Kilogram") => Unit::Kilogram,
                                            Some("Milliliter") => Unit::Milliliter,
                                            Some("Liter") => Unit::Liter,
                                            Some("Cup") => Unit::Cup,
                                            Some("Ounce") => Unit::Ounce,
                                            Some("Pound") => Unit::Pound,
                                            Some("Each") => Unit::Each,
                                            _ => Unit::Gram,
                                        };
                                        let exp_date = match exp_save.read(cx).date() {
                                            Date::Single(Some(d)) => Some(d),
                                            Date::Range(Some(d), _) => Some(d),
                                            _ => None,
                                        };
                                        v_save.update(cx, |this, cx| {
                                            this.add_item_id = i_uuid;
                                            this.add_unit = unit;
                                            if this.add_has_expiration {
                                                this.add_expiration = exp_date;
                                            } else {
                                                this.add_expiration = None;
                                            }
                                            this.save_pantry_entry(cx);
                                        });
                                        window.close_dialog(cx);
                                    }),
                            ),
                    )
            })
        });
    }

    pub fn save_pantry_entry(&mut self, cx: &mut Context<Self>) {
        let item_id = match self.add_item_id {
            Some(id) => id,
            None => {
                self.status_msg = "Error: Select an item to add to pantry".to_string();
                cx.notify();
                return;
            }
        };

        match self.services.pantry.add_pantry_entry(
            item_id,
            self.add_amount,
            self.add_unit.clone(),
            self.add_expiration,
        ) {
            Ok(entry) => {
                self.status_msg = format!(
                    "Added stock: {} {}",
                    entry.quantity.amount, entry.quantity.unit
                );
            }
            Err(e) => {
                self.status_msg = format!("Error adding pantry stock: {}", e);
            }
        }
        self.reload_data(cx);
    }

    pub fn consume_entry(
        &mut self,
        entry_id: PantryEntryId,
        amount: Decimal,
        unit: Unit,
        cx: &mut Context<Self>,
    ) {
        if let Some(entry) = self.cached_entries.iter().find(|e| e.id == entry_id) {
            match self
                .services
                .pantry
                .consume_pantry_item(entry.item_id, amount, unit.clone())
            {
                Ok(_) => {
                    self.status_msg = format!("Consumed stock: {} {}", amount, unit);
                }
                Err(e) => {
                    self.status_msg = format!("Error consuming stock: {}", e);
                }
            }
            self.reload_data(cx);
        }
    }

    pub fn update_quantity(
        &mut self,
        entry_id: PantryEntryId,
        delta: Decimal,
        cx: &mut Context<Self>,
    ) {
        let entries = &self.cached_entries;
        if let Some(entry) = entries.iter().find(|e| e.id == entry_id) {
            let next_amount = entry.quantity.amount + delta;
            if next_amount <= Decimal::ZERO {
                let _ = self.services.pantry.delete_pantry_entry(entry_id);
                self.status_msg = "Depleted stock entry removed".to_string();
            } else {
                let _ = self.services.pantry.update_quantity(entry_id, next_amount);
                self.status_msg = format!("Updated stock quantity to {}", next_amount.normalize());
            }
        }
        self.reload_data(cx);
    }

    pub fn delete_entry(&mut self, entry_id: PantryEntryId, cx: &mut Context<Self>) {
        match self.services.pantry.delete_pantry_entry(entry_id) {
            Ok(_) => {
                self.status_msg = "Deleted pantry stock entry".to_string();
            }
            Err(e) => {
                self.status_msg = format!("Error deleting entry: {}", e);
            }
        }
        self.reload_data(cx);
    }
}

impl Render for PantryView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
                                    .child("Pantry Inventory & Stock Tracking"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("Track available quantities, unit measurements, and expiration dates"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(Badge::new().child(format!(
                                "Pantry Entries: {}",
                                self.cached_entries.len()
                            )))
                            .child(
                                Button::new("btn-add-pantry-stock")
                                    .primary()
                                    .label("+ Add Stock")
                                    .on_click(cx.listener(|this, _event, window, cx| {
                                        this.open_add_modal(window, cx);
                                    })),
                            ),
                    ),
            )
            // Grouping & Sorting Control Bar
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .p_3()
                    .bg(cx.theme().muted)
                    .rounded_lg()
                    .child(
                        div().w_56().child(select_field("Section Grouping", Select::new(&self.group_mode_select))),
                    )
                    .child(
                        div().w_48().child(select_field("Sorting", Select::new(&self.sort_mode_select))),
                    )
                    .child(Alert::new("pantry-status-alert", format!("Status: {}", self.status_msg))),
            )
            // Inventory Table List with Sticky Section Headers
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_5()
                    .bg(cx.theme().background)
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded_lg()
                    .flex_1()
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .pb_2()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(cx.theme().muted_foreground)
                            .child(div().w_1_4().child("Item Name"))
                            .child(div().w_1_4().child("Stock Quantity"))
                            .child(div().w_1_4().child("Expiration Status"))
                            .child(div().w_1_4().child("Actions")),
                    )
                    .child(
                        List::new(&self.pantry_list)
                            .flex_1()
                            .w_full(),
                    ),
            )
    }
}
