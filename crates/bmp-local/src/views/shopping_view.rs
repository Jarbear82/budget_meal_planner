use bmp_services::AppServices;
use gpui::*;
use gpui_component::button::{Button, ButtonVariants};
use gpui_component::ActiveTheme;

pub struct ShoppingView {
    pub services: AppServices,
}

impl ShoppingView {
    pub fn new(services: AppServices) -> Self {
        Self { services }
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
                        Button::new("btn-reconcile-receipt")
                            .primary()
                            .label("Checkout & Reconcile Receipt"),
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
                    .child(div().text_sm().text_color(cx.theme().muted_foreground).child("Store Selection Filter: All Stores")),
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
