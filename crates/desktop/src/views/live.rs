use crate::components::button::{Button, ButtonVariant};
use crate::components::card::*;
use crate::components::input::Input;
use crate::components::label::Label;
use crate::controller::use_controller;
use crate::state::use_app_state;
use dioxus::prelude::*;
use dioxus_i18n::t;

fn live_status_label(status: service::event::LiveStatus) -> &'static str {
    match status {
        service::event::LiveStatus::Idle => "Idle",
        service::event::LiveStatus::Starting => "Starting",
        service::event::LiveStatus::Running => "Running",
        service::event::LiveStatus::Stopping => "Stopping",
        service::event::LiveStatus::Stopped => "Stopped",
        service::event::LiveStatus::Failed => "Error",
    }
}

#[component]
pub fn Live() -> Element {
    let mut state = use_app_state();
    let controller = use_controller();
    let ctrl_start = controller.clone();
    let ctrl_stop = controller.clone();
    let status_label = live_status_label(state.read().live_status);
    let message_count = state.read().message_count;

    rsx! {
        div { class: "view-container-sm",
            div { class: "view-header",
                h2 { class: "view-title", {t!("live-title")} }
                p { class: "view-description", {t!("live-description")} }
            }

            Card {
                CardHeader {
                    CardTitle { {t!("configuration")} }
                    CardDescription { {t!("set-room-id")} }
                }
                CardContent {
                    div { class: "form-item",
                        Label { html_for: "live-id", {t!("room-id")} }
                        Input {
                            id: "live-id",
                            placeholder: t!("enter-room-id-placeholder"),
                            value: "{state.read().config.live.id}",
                            oninput: move |e: FormEvent| {
                                state.write().config.live.id = e.value();
                            }
                        }
                    }
                }
            }

            div { class: "action-buttons",
                if state.read().is_running() {
                    Button {
                        variant: ButtonVariant::Destructive,
                        class: "btn-xl",
                        onclick: move |_| {
                            let controller = ctrl_stop.clone();
                            tracing::info!("User clicked stop monitoring");
                            spawn(async move {
                                let mut ctrl = controller.lock().await;
                                let _ = ctrl.stop().await;
                            });
                        },
                        {t!("stop-monitoring")}
                    }
                } else {
                    Button {
                        variant: ButtonVariant::Primary,
                        class: "btn-xl",
                        onclick: move |_| {
                            let controller = ctrl_start.clone();
                            let config = state.read().config.clone();
                            tracing::info!("User clicked start monitoring for room: {}", config.live.id);
                            spawn(async move {
                                let mut ctrl = controller.lock().await;
                                if let Err(e) = ctrl.start(config).await {
                                    let err_msg = e.to_string();
                                    tracing::error!("Failed to start monitoring: {}", err_msg);
                                    state.write().error_message = Some(err_msg);
                                }
                            });
                        },
                        {t!("start-monitoring")}
                    }
                }

                div { class: "status-bar",
                    span { "{t!(\"status\")}: {status_label}" }
                    span { "{t!(\"messages-received\")}: {message_count}" }
                }
            }

            if let Some(error) = state.read().error_message.clone() {
                div { class: "error-display",
                    Card {
                        CardContent {
                            p { class: "text-destructive", "{error}" }
                            Button {
                                variant: ButtonVariant::Secondary,
                                onclick: move |_| {
                                    state.write().error_message = None;
                                },
                                "Clear"
                            }
                        }
                    }
                }
            }
        }
    }
}
