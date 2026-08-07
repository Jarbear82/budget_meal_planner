use bmp_services::AppServices;
use gpui::*;

pub struct ShoppingView {
    pub services: AppServices,
}

impl ShoppingView {
    pub fn new(services: AppServices) -> Self {
        Self { services }
    }
}

impl Render for ShoppingView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_4()
            .size_full()
            .p_6()
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .text_2xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xf4f4f5))
                            .child("Shopping List & In-Store Checklist"),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .bg(rgb(0x10b981))
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x18181b))
                            .child("Checkout & Reconcile Receipt"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap_4()
                    .p_4()
                    .bg(rgb(0x18181b))
                    .border_1()
                    .border_color(rgb(0x27272a))
                    .rounded_lg()
                    .child(div().text_sm().text_color(rgb(0xa1a1aa)).child("Store Selection Filter: All Stores")),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .p_4()
                    .bg(rgb(0x18181b))
                    .border_1()
                    .border_color(rgb(0x27272a))
                    .rounded_lg()
                    .child(div().text_xs().font_weight(FontWeight::BOLD).text_color(rgb(0xa1a1aa)).child("DYNAMIC SHOPPING ITEMS"))
                    .child(div().text_sm().text_color(rgb(0xa1a1aa)).child("Generate a shopping list from scheduled calendar meals to view store package counts and costs.")),
            )
    }
}
