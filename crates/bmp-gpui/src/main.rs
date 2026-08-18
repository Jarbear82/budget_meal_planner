pub mod app;
pub mod components;
pub mod views;

use app::BudgetMealPlannerApp;
use gpui::*;
#[allow(unused_imports)]
use gpui_component::{Root, TitleBar};

fn main() {
    let app = gpui_platform::application().with_assets(gpui_component_assets::Assets);
    app.run(move |cx| {
        gpui_component::init(cx);

        let options = WindowOptions {
            // window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
            //     None,
            //     size(px(1280.), px(800.)),
            //     cx,
            // ))),
            #[cfg(not(target_os = "linux"))]
            titlebar: Some(TitleBar::title_bar_options()),
            #[cfg(target_os = "linux")]
            window_decorations: Some(WindowDecorations::Server),
            ..Default::default()
        };

        cx.open_window(options, |window, cx| {
            eprintln!(
                "Runtime window decorations = {:?}",
                window.window_decorations()
            );
            let app_view = cx.new(|cx| BudgetMealPlannerApp::new(window, cx));
            cx.new(|cx| Root::new(app_view, window, cx))
        })
        .unwrap();
    });
}
