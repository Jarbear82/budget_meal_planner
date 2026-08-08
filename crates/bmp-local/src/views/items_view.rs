use bmp_domain::*;
use bmp_services::AppServices;
use gpui::*;
use gpui_component::badge::Badge;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::tag::Tag;
use gpui_component::ActiveTheme;
use rust_decimal_macros::dec;

pub struct ItemsView {
    pub services: AppServices,
    pub _search_query: String,
    pub status_msg: String,
}

impl ItemsView {
    pub fn new(services: AppServices) -> Self {
        Self {
            services,
            _search_query: String::new(),
            status_msg: "Ready".to_string(),
        }
    }

    pub fn add_sample_item(&mut self, cx: &mut Context<Self>) {
        let count = self.services.items.list_items().map(|l| l.len()).unwrap_or(0);
        let name = format!("New Custom Item {}", count + 1);
        match self.services.items.create_item(&name, Some(dec!(1.0)), Some("Pantry")) {
            Ok(item) => {
                self.status_msg = format!("Added item: {}", item.name);
            }
            Err(e) => {
                self.status_msg = format!("Error: {}", e);
            }
        }
        cx.notify();
    }

    pub fn toggle_purchase_mode(&mut self, item_id: ItemId, cx: &mut Context<Self>) {
        if let Ok(mut items) = self.services.items.list_items() {
            if let Some(item) = items.iter_mut().find(|i| i.id == item_id) {
                item.preferred_purchase_mode = match item.preferred_purchase_mode {
                    PurchaseMode::BuyFinished => PurchaseMode::PreferMake,
                    PurchaseMode::PreferMake => PurchaseMode::AskEveryTime,
                    PurchaseMode::AskEveryTime => PurchaseMode::BuyFinished,
                };
                if self.services.items.update_item(item).is_ok() {
                    self.status_msg = format!("Updated purchase mode for {}", item.name);
                }
            }
        }
        cx.notify();
    }
}

impl Render for ItemsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let items = self.services.items.list_items().unwrap_or_default();
        let stores = self.services.items.list_stores().unwrap_or_default();

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
                            .child("Items & Store Packages Matrix"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(Badge::new().child(format!("Total Items: {}", items.len())))
                            .child(
                                Button::new("btn-add-item")
                                    .primary()
                                    .label("+ Add Item")
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.add_sample_item(cx);
                                    })),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_3()
                    .p_3()
                    .bg(cx.theme().muted)
                    .rounded_lg()
                    .child(
                        div()
                            .flex_1()
                            .p_2()
                            .bg(cx.theme().background)
                            .border_1()
                            .border_color(cx.theme().border)
                            .rounded_md()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(format!("Status: {}", self.status_msg)),
                    ),
            )
            .child(
                div()
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
                            .flex()
                            .justify_between()
                            .pb_2()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(cx.theme().muted_foreground)
                            .child(div().w_1_4().child("Item Name"))
                            .child(div().w_1_4().child("Density (g/ml)"))
                            .child(div().w_1_4().child("Purchase Mode (Click to Toggle)"))
                            .child(div().w_1_4().child("Category")),
                    )
                    .children(items.into_iter().map(|item| {
                        let item_id = item.id;
                        let density_str = item
                            .density
                            .map(|d| format!("{} g/ml", d.g_per_ml.normalize()))
                            .unwrap_or_else(|| "Missing Density".to_string());
                        let category_str = item.category.unwrap_or_else(|| "Uncategorized".to_string());
                        let mode_str = format!("{:?}", item.preferred_purchase_mode);

                        div()
                            .flex()
                            .justify_between()
                            .items_center()
                            .py_2()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .text_sm()
                            .text_color(cx.theme().foreground)
                            .child(div().w_1_4().font_weight(FontWeight::BOLD).child(item.name))
                            .child(div().w_1_4().child(Tag::new().child(density_str)))
                            .child(
                                div().w_1_4().child(
                                    Button::new(format!("btn-toggle-mode-{}", item_id))
                                        .secondary()
                                        .label(mode_str)
                                        .on_click(cx.listener(move |this, _event, _window, cx| {
                                            this.toggle_purchase_mode(item_id, cx);
                                        })),
                                ),
                            )
                            .child(div().w_1_4().text_color(cx.theme().muted_foreground).child(category_str))
                    })),
            )
            .child(
                div()
                    .flex()
                    .gap_4()
                    .child(
                        div()
                            .flex_1()
                            .p_4()
                            .bg(cx.theme().muted)
                            .rounded_lg()
                            .child(div().text_sm().font_weight(FontWeight::BOLD).text_color(cx.theme().foreground).child("Stores Registry"))
                            .child(div().text_xs().text_color(cx.theme().muted_foreground).child(format!("{} Registered Stores", stores.len()))),
                    ),
            )
    }
}
