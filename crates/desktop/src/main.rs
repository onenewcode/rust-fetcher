use desktop::App;
use dioxus::desktop::muda::{Menu, MenuItem, PredefinedMenuItem, Submenu};
#[cfg(target_os = "macos")]
use dioxus::desktop::tao::platform::macos::WindowBuilderExtMacOS;
use dioxus::desktop::{Config, WindowBuilder};

fn main() {
    let project_root = std::env::current_dir().unwrap_or_default();
    let log_dir = project_root.join("logs");
    let _guard = common::logging::init_tracing("info", Some(log_dir))
        .ok()
        .flatten();

    let mut window = WindowBuilder::new().with_title("Douyin Fetcher");

    #[cfg(target_os = "macos")]
    {
        window = window
            .with_title_hidden(true)
            .with_titlebar_transparent(true)
            .with_fullsize_content_view(true);
    }

    let menu = Menu::new();

    #[cfg(target_os = "macos")]
    {
        let app_menu = Submenu::new("Douyin Fetcher", true);
        let lang_menu = Submenu::new("Language", true);
        lang_menu
            .append_items(&[
                &MenuItem::with_id("lang_en", "English", true, None),
                &MenuItem::with_id("lang_zh", "中文 (Chinese)", true, None),
            ])
            .unwrap();

        app_menu
            .append_items(&[
                &PredefinedMenuItem::about(None, None),
                &PredefinedMenuItem::separator(),
                &MenuItem::with_id("settings", "Preferences...", true, None),
                &PredefinedMenuItem::separator(),
                &lang_menu,
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::hide(None),
                &PredefinedMenuItem::hide_others(None),
                &PredefinedMenuItem::show_all(None),
                &PredefinedMenuItem::separator(),
                &PredefinedMenuItem::quit(None),
            ])
            .unwrap();
        menu.append(&app_menu).unwrap();
    }

    let view_menu = Submenu::new("View", true);
    view_menu
        .append_items(&[&MenuItem::with_id(
            "toggle_theme",
            "Toggle Theme",
            true,
            None,
        )])
        .unwrap();
    menu.append(&view_menu).unwrap();

    let config = Config::new().with_window(window).with_menu(menu);

    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(App);
}
