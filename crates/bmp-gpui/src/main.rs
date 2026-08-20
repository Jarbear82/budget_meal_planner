pub mod app;
pub mod components;
pub mod views;

use app::BudgetMealPlannerApp;
use gpui::*;
#[allow(unused_imports)]
use gpui_component::{ActiveTheme, Root, Theme, ThemeRegistry, TitleBar};
use std::path::PathBuf;

fn main() {
    let app = gpui_platform::application().with_assets(gpui_component_assets::Assets);
    app.run(move |cx| {
        gpui_component::init(cx);

        load_and_watch_themes(cx);

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

fn load_and_watch_themes(cx: &mut App) {
    let themes_dir = PathBuf::from("./themes");
    if !themes_dir.exists() {
        let _ = std::fs::create_dir_all(&themes_dir);
    }

    // Load + watch. Closure runs after initial load and on every change.
    if let Err(err) = ThemeRegistry::watch_dir(themes_dir, cx, move |cx| {
        let (light, dark) = {
            let registry = ThemeRegistry::global(cx);
            (
                registry.themes().get("Molokai Light").cloned(),
                registry.themes().get("Molokai Dark").cloned(),
            )
        };

        if let Some(light) = light {
            Theme::global_mut(cx).light_theme = light;
        }
        if let Some(dark) = dark {
            Theme::global_mut(cx).dark_theme = dark;
        }

        Theme::sync_system_appearance(None, cx);
        cx.refresh_windows();
    }) {
        eprintln!("failed to bind themes file monitor: {:?}", err);
    }
}
