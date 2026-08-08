use bmp_services::AppServices;
use chrono::Utc;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::ActiveTheme;
use rust_decimal_macros::dec;

pub struct ShoppingView {
    pub services: AppServices,
    pub status_msg: String,
}

impl ShoppingView {
    pub fn new(services: AppServices) -> Self {
        Self {
            services,
            status_msg: "Ready".to_string(),
        }
    }

    pub fn generate_shopping_list(&mut self, cx: &mut Context<Self>) {
        match self.services.shopping.generate_shopping_list(Vec::new(), None, None) {
            Ok(list) => {
                self.status_msg = format!("Shopping list generated! Total items: {}", list.items.len());
            }
            Err(e) => {
                self.status_msg = format!("Error: {}", e);
            }
        }
        cx.notify();
    }

    pub fn record_receipt(&mut self, cx: &mut Context<Self>) {
        match self.services.analytics.record_receipt(None, dec!(45.50), Utc::now()) {
            Ok(id) => {
                self.status_msg = format!("Recorded receipt ID: {}. Actual total: $45.50", id);
            }
            Err(e) => {
                self.status_msg = format!("Error: {}", e);
            }
        }
        cx.notify();
    }
}

impl Render for ShoppingView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
                            .child("Shopping List & In-Store Checklist"),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                Button::new("btn-gen-list")
                                    .secondary()
                                    .label("Generate List")
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.generate_shopping_list(cx);
                                    })),
                            )
                            .child(
                                Button::new("btn-reconcile-receipt")
                                    .primary()
                                    .label("Checkout & Reconcile Receipt")
                                    .on_click(cx.listener(|this, _event, _window, cx| {
                                        this.record_receipt(cx);
                                    })),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_4()
                    .p_4()
                    .bg(cx.theme().background)
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded_lg()
                    .child(div().text_sm().text_color(cx.theme().muted_foreground).child(format!("Status: {}", self.status_msg))),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_4()
                    .bg(cx.theme().background)
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded_lg()
                    .child(div().text_xs().font_weight(FontWeight::BOLD).text_color(cx.theme().muted_foreground).child("DYNAMIC SHOPPING ITEMS"))
                    .child(div().text_sm().text_color(cx.theme().muted_foreground).child("Generate a shopping list from scheduled calendar meals to view store package counts and costs.")),
            )
    }
}
