use gpui::prelude::*;
use gpui::*;
pub use gpui_component::IndexPath;
use gpui_component::searchable_list::SearchableListItem;
pub use gpui_component::select::{Select, SelectEvent, SelectState};

/// A single option for the Select / Combobox component.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectOption {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
}

impl SelectOption {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            description: None,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

impl SearchableListItem for SelectOption {
    type Value = String;

    fn title(&self) -> SharedString {
        self.label.clone().into()
    }

    fn value(&self) -> &Self::Value {
        &self.id
    }
}

/// A clean form field wrapper pairing a label with a Select dropdown.
pub fn select_field<D>(
    label: impl Into<SharedString>,
    select: Select<D>,
) -> impl IntoElement
where
    D: gpui_component::searchable_list::SearchableListDelegate + 'static,
    <D::Item as gpui_component::searchable_list::SearchableListItem>::Value: PartialEq + Clone,
{
    div()
        .flex()
        .flex_col()
        .gap_1()
        .w_full()
        .child(
            div()
                .text_xs()
                .font_weight(FontWeight::SEMIBOLD)
                .child(label.into()),
        )
        .child(select)
}
