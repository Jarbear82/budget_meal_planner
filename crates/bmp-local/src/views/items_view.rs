use bmp_services::AppServices;
use gpui::*;
use gpui_component::ActiveTheme;

pub struct ItemsView {
    pub services: AppServices,
    pub _search_query: String,
}

impl ItemsView {
    pub fn new(services: AppServices) -> Self {
        Self {
            services,
            _search_query: String::new(),
        }
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
                            .px_3()
                            .py_1p5()
                            .rounded_md()
                            .bg(cx.theme().muted)
                            .text_xs()
                            .text_color(cx.theme().foreground)
                            .child(format!("Total Items: {}", items.len())),
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
                            .child("Search ingredients by name or category..."),
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
                            .child(div().w_1_4().child("Purchase Mode"))
                            .child(div().w_1_4().child("Category")),
                    )
                    .children(items.into_iter().map(|item| {
                        let density_str = item
                            .density
                            .map(|d| format!("{} g/ml", d.g_per_ml.normalize()))
                            .unwrap_or_else(|| "Missing Density".to_string());
                        let category_str = item.category.unwrap_or_else(|| "Uncategorized".to_string());
                        let mode_str = format!("{:?}", item.preferred_purchase_mode);

                        div()
                            .flex()
                            .justify_between()
                            .py_2()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .text_sm()
                            .text_color(cx.theme().foreground)
                            .child(div().w_1_4().font_weight(FontWeight::BOLD).child(item.name))
                            .child(div().w_1_4().text_color(rgb(0x10b981)).child(density_str))
                            .child(div().w_1_4().text_color(rgb(0x38bdf8)).child(mode_str))
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
