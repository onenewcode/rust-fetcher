use crate::components::button::{Button, ButtonVariant};
use crate::components::card::*;
use crate::components::label::Label;
use crate::components::textarea::Textarea;
use crate::controller::use_controller;
use crate::state::use_app_state;
use dioxus::prelude::*;
use dioxus_i18n::t;

#[component]
pub fn Settings() -> Element {
    let mut state = use_app_state();
    let controller = use_controller();

    rsx! {
        div { class: "view-container",
            div { class: "view-header",
                h2 { class: "view-title", {t!("settings")} }
                p { class: "view-description", {t!("settings-description")} }
            }

            div { class: "form-group",
                Card {
                    CardHeader {
                        CardTitle { {t!("live-cookie")} }
                        CardDescription { "Authentication cookie for live stream monitoring" }
                    }
                    CardContent {
                        div { class: "form-item",
                            Label { html_for: "live-cookie", {t!("cookie-string")} }
                            Textarea {
                                id: "live-cookie",
                                class: "cookie-textarea",
                                placeholder: t!("cookie-placeholder"),
                                value: "{state.read().config.live.cookie}",
                                oninput: move |e: FormEvent| {
                                    state.write().config.live.cookie = e.value();
                                }
                            }
                        }
                    }
                }

                Card {
                    CardHeader {
                        CardTitle { {t!("im-cookie")} }
                        CardDescription { "Authentication cookie for instant messaging" }
                    }
                    CardContent {
                        div { class: "form-item",
                            Label { html_for: "im-cookie", {t!("cookie-string")} }
                            Textarea {
                                id: "im-cookie",
                                class: "cookie-textarea",
                                placeholder: t!("cookie-placeholder"),
                                value: "{state.read().config.im.cookie}",
                                oninput: move |e: FormEvent| {
                                    state.write().config.im.cookie = e.value();
                                }
                            }
                        }
                    }
                }
            }

            div { class: "form-actions",
                Button {
                    variant: ButtonVariant::Primary,
                    class: "btn-save",
                    onclick: move |_| {
                        let controller = controller.clone();
                        let current_config = state.read().config.clone();
                        spawn(async move {
                            let ctrl = controller.lock().await;
                            let _ = ctrl.save_config(&current_config);
                        });
                    },
                    {t!("save-configuration")}
                }
            }
        }
    }
}
