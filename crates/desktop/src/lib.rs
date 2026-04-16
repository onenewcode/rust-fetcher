pub mod components;
pub mod controller;
pub mod hooks;
pub mod layouts;
pub mod router;
pub mod state;
pub mod views;
pub mod widgets;

use crate::controller::{AppController, Controller};
use crate::hooks::use_app_events;
use crate::router::Route;
use crate::state::AppState;
use dioxus::prelude::*;
use dioxus_i18n::prelude::*;
use std::sync::Arc;
use tokio::sync::Mutex;
use unic_langid::langid;

#[component]
pub fn App() -> Element {
    use_init_i18n(|| {
        I18nConfig::new(langid!("zh-CN"))
            .with_locale(Locale::new_static(
                langid!("en-US"),
                include_str!("../locales/en-US.ftl"),
            ))
            .with_locale(Locale::new_static(
                langid!("zh-CN"),
                include_str!("../locales/zh-CN.ftl"),
            ))
    });

    let mut state = use_context_provider(|| Signal::new(AppState::new()));
    let project_root = std::env::current_dir().unwrap_or_default();
    let controller = use_context_provider::<AppController>(|| {
        Arc::new(Mutex::new(Controller::new(project_root)))
    });

    use_app_events(state, controller.clone());

    let mut i18 = i18n();

    // Load initial config
    use_hook(move || {
        spawn(async move {
            let ctrl = controller.lock().await;
            if let Ok(config) = ctrl.load_config() {
                state.write().config = config.clone();
                state.write().selected_theme = config.theme;
                if let Ok(lang) = config.language.parse() {
                    i18.set_language(lang);
                }
            }
        });
    });

    use_effect(move || {
        let theme_val = state.read().selected_theme.as_str();
        let js = format!(
            "document.documentElement.setAttribute('data-theme', '{}')",
            theme_val
        );
        _ = document::eval(&js);
    });

    rsx! {
        document::Link { rel: "stylesheet", href: asset!("/assets/dx-components-theme.css") }
        document::Link { rel: "stylesheet", href: asset!("/assets/custom-themes.css") }
        document::Link { rel: "stylesheet", href: asset!("/assets/views.css") }
        div { class: "min-h-screen bg-[var(--primary-color-2)] text-[var(--secondary-color)] transition-colors duration-200",
            Router::<Route> {}
        }
    }
}
