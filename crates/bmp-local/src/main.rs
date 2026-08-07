pub mod app;
pub mod components;
pub mod views;

use app::BudgetMealPlannerApp;
use gpui::*;
use gpui_component::Root;

fn main() {
    let app = gpui_platform::application().with_assets(gpui_component_assets::Assets);
    app.run(move |cx| {
        gpui_component::init(cx);

        let options = WindowOptions::default();
        cx.open_window(options, |window, cx| {
            let app_view = cx.new(|cx| BudgetMealPlannerApp::new(cx));
            cx.new(|cx| Root::new(app_view, window, cx))
        })
        .unwrap();
    });
}
