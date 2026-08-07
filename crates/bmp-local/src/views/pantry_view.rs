use bmp_services::AppServices;
use gpui::*;

pub struct PantryView {
    pub services: AppServices,
}

impl PantryView {
    pub fn new(services: AppServices) -> Self {
        Self { services }
    }
}

impl Render for PantryView {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let entries = self.services.pantry.get_pantry().unwrap_or_default();

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
                            .child("Pantry Inventory & Stock Manager"),
                    )
                    .child(
                        div()
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .bg(rgb(0x3f3f46))
                            .text_xs()
                            .text_color(rgb(0xf4f4f5))
                            .child(format!("Total Items in Pantry: {}", entries.len())),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .p_4()
                    .bg(rgb(0x18181b))
                    .border_1()
                    .border_color(rgb(0x27272a))
                    .rounded_lg()
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .pb_2()
                            .border_b_1()
                            .border_color(rgb(0x27272a))
                            .text_xs()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0xa1a1aa))
                            .child(div().w_1_3().child("Item ID"))
                            .child(div().w_1_3().child("Quantity Stored"))
                            .child(div().w_1_3().child("Expiration Date")),
                    )
                    .children(entries.iter().map(|e| {
                        let exp_str = e.expiration.map(|d| d.to_string()).unwrap_or_else(|| "No Expiration".to_string());
                        div()
                            .flex()
                            .justify_between()
                            .py_2()
                            .border_b_1()
                            .border_color(rgb(0x27272a))
                            .text_sm()
                            .text_color(rgb(0xf4f4f5))
                            .child(div().w_1_3().child(format!("{}", e.item_id)))
                            .child(div().w_1_3().text_color(rgb(0x10b981)).child(format!("{}", e.quantity)))
                            .child(div().w_1_3().text_color(rgb(0xa1a1aa)).child(exp_str))
                    })),
            )
    }
}
