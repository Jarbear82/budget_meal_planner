use gpui::prelude::*;
use gpui::*;
pub use gpui_component::{
    calendar::{Date, Matcher},
    date_picker::{DatePicker, DatePickerEvent, DatePickerState, DateRangePreset},
};

/// A clean form field wrapper pairing a label with a DatePicker component.
pub fn date_picker_field(label: impl Into<SharedString>, picker: DatePicker) -> impl IntoElement {
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
        .child(picker)
}
