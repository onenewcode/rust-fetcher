use crate::components::button::{Button, ButtonVariant};
use crate::components::sidebar::*;
use crate::router::Route;
use crate::state::use_app_state;
use dioxus::prelude::*;
use dioxus_i18n::prelude::i18n;
use dioxus_i18n::t;
use dioxus_primitives::icon;
use unic_langid::langid;

#[component]
fn UserMenu() -> Element {
    let mut is_open = use_signal(|| false);
    let mut state = use_app_state();
    let nav = use_navigator();

    rsx! {
        div { class: "user-menu-wrapper",
            Button {
                variant: ButtonVariant::Ghost,
                class: "user-menu-trigger",
                onclick: move |_| is_open.set(!is_open()),
                span { class: "user-menu-trigger-text", {t!("settings")} }
                icon::Icon {
                    width: "0.75rem",
                    height: "0.75rem",
                    stroke: "currentColor",
                    polyline { points: "6 9 12 15 18 9" }
                }
            }
            if is_open() {
                div { class: "user-menu-dropdown",
                    div { class: "user-menu-list",
                        button {
                            class: "user-menu-item",
                            onclick: move |_| {
                                is_open.set(false);
                                nav.push(Route::Settings {});
                            },
                            span { {t!("settings")} }
                            span { class: "user-menu-shortcut", "Cmd+," }
                        }
                        div { class: "menu-separator" }
                        button {
                            class: "user-menu-item",
                            onclick: move |_| {
                                state.write().toggle_theme();
                                is_open.set(false);
                            },
                            span { {t!("themes")} }
                            span { class: "user-menu-shortcut", "Cmd+K Cmd+T" }
                        }
                        div { class: "menu-separator" }
                        button {
                            class: "user-menu-item",
                            onclick: move |_| {
                                let current = i18n().language();
                                if current == langid!("en-US") {
                                    i18n().set_language(langid!("zh-CN"));
                                    state.write().config.language = "zh-CN".to_string();
                                } else {
                                    i18n().set_language(langid!("en-US"));
                                    state.write().config.language = "en-US".to_string();
                                }
                                is_open.set(false);
                            },
                            span { {t!("language")} }
                            span { class: "user-menu-shortcut",
                                if i18n().language() == langid!("en-US") { "English" } else { "中文" }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn AppLayout() -> Element {
    let nav = use_navigator();
    let mut state = use_app_state();
    let window = dioxus::desktop::use_window();

    dioxus::desktop::use_muda_event_handler(move |event| {
        if event.id() == "settings" {
            nav.push(Route::Settings {});
        } else if event.id() == "lang_en" {
            i18n().set_language(langid!("en-US"));
            state.write().config.language = "en-US".to_string();
        } else if event.id() == "lang_zh" {
            i18n().set_language(langid!("zh-CN"));
            state.write().config.language = "zh-CN".to_string();
        } else if event.id() == "toggle_theme" {
            state.write().toggle_theme();
        }
    });

    rsx! {
        SidebarProvider {
            default_open: true,
            div { class: "app-layout-container",
                // Custom Titlebar Header
                header {
                    class: "app-header-bar",
                    onmousedown: move |_| { window.drag(); },
                    SidebarTrigger {
                        class: "sidebar-trigger-btn",
                    }
                    h1 { class: "app-title-text",
                        {t!("app-title")}
                    }
                    UserMenu {}
                }
                div { class: "app-layout-main",
                    Sidebar {
                        variant: SidebarVariant::Sidebar,
                        collapsible: SidebarCollapsible::Offcanvas,
                        side: SidebarSide::Left,
                        SidebarHeader {
                            div { class: "sidebar-title", {t!("fetcher")} }
                        }
                        SidebarContent {
                            SidebarGroup {
                                SidebarGroupLabel { {t!("fetchers")} }
                                SidebarMenu {
                                    SidebarMenuItem {
                                        SidebarMenuButton {
                                            r#as: move |attrs: Vec<Attribute>| rsx! {
                                                a {
                                                    onclick: move |_| { nav.push(Route::Live {}); },
                                                    ..attrs,
                                                    span { class: "form-item flex-row-center",
                                                        icon::Icon {
                                                            width: "1rem",
                                                            height: "1rem",
                                                            stroke: "currentColor",
                                                            circle { cx: "12", cy: "12", r: "10" }
                                                            circle { cx: "12", cy: "12", r: "3" }
                                                        }
                                                        {t!("live")}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    SidebarMenuItem {
                                        SidebarMenuButton {
                                            r#as: move |attrs: Vec<Attribute>| rsx! {
                                                a {
                                                    onclick: move |_| { nav.push(Route::Im {}); },
                                                    ..attrs,
                                                    span { class: "form-item flex-row-center",
                                                        icon::Icon {
                                                            width: "1rem",
                                                            height: "1rem",
                                                            stroke: "currentColor",
                                                            path { d: "M21 11.5a8.38 8.38 0 0 1-.9 3.8 8.5 8.5 0 0 1-7.6 4.7 8.38 8.38 0 0 1-3.8-.9L3 21l1.9-5.7a8.38 8.38 0 0 1-.9-3.8 8.5 8.5 0 0 1 4.7-7.6 8.38 8.38 0 0 1 3.8-.9h.5a8.48 8.48 0 0 1 8 8v.5z" }
                                                        }
                                                        {t!("im")}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        SidebarFooter {
                            div { class: "sidebar-footer-box",
                                div { class: "sidebar-footer-text", "v0.1.0" }
                            }
                        }
                    }
                    main { class: "app-main-content",
                        Outlet::<Route> {}
                    }
                }
            }
        }
    }
}
