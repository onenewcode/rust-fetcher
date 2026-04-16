use crate::components::button::{Button, ButtonVariant};
use crate::components::card::*;
use crate::components::input::Input;
use crate::components::label::Label;
use crate::components::textarea::Textarea;
use crate::controller::use_controller;
use crate::state::use_app_state;
use dioxus::prelude::*;
use dioxus_i18n::t;

#[component]
pub fn Im() -> Element {
    let mut state = use_app_state();
    let controller = use_controller();

    let ctrl_save = controller.clone();
    let ctrl_send = controller.clone();
    let ctrl_bulk = controller.clone();

    rsx! {
        div { class: "view-container-md",
            div { class: "view-header",
                h2 { class: "view-title", {t!("im-title")} }
                p { class: "view-description", {t!("im-description")} }
            }

            Card {
                CardHeader {
                    CardTitle { {t!("message-details")} }
                    CardDescription { {t!("recipient-content")} }
                }
                CardContent {
                    div { class: "form-item-lg",
                        div { class: "form-item",
                            Label { html_for: "im-receiver-id", {t!("receiver-id")} }
                            Input {
                                id: "im-receiver-id",
                                placeholder: t!("enter-recipient-id"),
                                value: "{state.read().config.im.receiver_id.clone().unwrap_or_default()}",
                                oninput: move |e: FormEvent| {
                                    state.write().config.im.receiver_id = Some(e.value());
                                }
                            }
                        }

                        div { class: "form-item",
                            Label { html_for: "im-message-content", {t!("message-content")} }
                            Textarea {
                                id: "im-message-content",
                                class: "cookie-textarea",
                                placeholder: t!("message-placeholder"),
                                value: "{state.read().config.im.message_text.clone().unwrap_or_default()}",
                                oninput: move |e: FormEvent| {
                                    state.write().config.im.message_text = Some(e.value());
                                }
                            }
                        }
                    }
                }
                CardFooter {
                    div { class: "action-bar",
                        Button {
                            variant: ButtonVariant::Outline,
                            class: "flex-1",
                            onclick: move |_| {
                                let ctrl = ctrl_save.clone();
                                let current_config = state.read().config.clone();
                                spawn(async move {
                                    let ctrl = ctrl.lock().await;
                                    let _ = ctrl.save_config(&current_config);
                                });
                            },
                            {t!("save-draft")}
                        }
                        Button {
                            variant: ButtonVariant::Primary,
                            class: "flex-2",
                            onclick: move |_| {
                                let r = state.read().config.im.receiver_id.clone();
                                let c = state.read().config.im.message_text.clone();
                                if let (Some(r_id), Some(_)) = (r, c) {
                                    tracing::info!("User sending direct message to {}", r_id);
                                    let ctrl = ctrl_send.clone();
                                    let config = state.read().config.clone();
                                    spawn(async move {
                                        let ctrl = ctrl.lock().await;
                                        match ctrl.send_im(config).await {
                                            Ok(_) => {
                                                tracing::info!("Message sent successfully to {}", r_id);
                                            }
                                            Err(e) => {
                                                let err_msg = e.to_string();
                                                tracing::error!("Failed to send message: {}", err_msg);
                                                state.write().error_message = Some(err_msg);
                                            }
                                        }
                                    });
                                } else {
                                    rfd::MessageDialog::new()
                                        .set_title("Validation Error")
                                        .set_description("Please enter both receiver ID and message content.")
                                        .set_level(rfd::MessageLevel::Warning)
                                        .show();
                                }
                            },
                            {t!("send-message")}
                        }
                    }
                }
            }

            Card {
                CardHeader {
                    CardTitle { "批量发送 (Bulk IM)" }
                    CardDescription { "选择抓取的 CSV 文件，自动按顺序发送私信。包含随机延迟保护账号。" }
                }
                CardContent {
                    div { class: "action-buttons",
                        div { class: "bulk-file-selector",
                            Button {
                                variant: ButtonVariant::Outline,
                                onclick: move |_| {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("CSV", &["csv"])
                                        .pick_file() {
                                        tracing::info!("User selected CSV file for bulk IM: {:?}", path);
                                        state.write().bulk_csv_path = Some(path);
                                    }
                                },
                                "选择 CSV 文件"
                            }
                            if let Some(path) = state.read().bulk_csv_path.clone() {
                                span { class: "view-description text-primary", "{path.file_name().unwrap().to_string_lossy()}" }
                            } else {
                                span { class: "view-description", "未选择文件" }
                            }
                        }

                        if state.read().is_bulk_sending || state.read().bulk_status.is_some() {
                            div { class: "bulk-status-box",
                                div { class: "bulk-status-header",
                                    span { "{state.read().bulk_status_text().unwrap_or_default()}" }
                                    if state.read().bulk_total > 0 {
                                        span { "{state.read().bulk_progress} / {state.read().bulk_total}" }
                                    }
                                }
                                if state.read().bulk_total > 0 {
                                    div { class: "bulk-progress-track",
                                        div {
                                            class: "bulk-progress-fill",
                                            width: "{state.read().bulk_progress as f32 / state.read().bulk_total as f32 * 100.0}%",
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                CardFooter {
                    Button {
                        variant: ButtonVariant::Primary,
                        class: "w-full",
                        disabled: state.read().bulk_csv_path.is_none() || state.read().is_bulk_sending,
                        onclick: move |_| {
                            let csv_path = state.read().bulk_csv_path.clone();
                            let config = state.read().config.clone();
                            let ctrl = ctrl_bulk.clone();
                            if let Some(path) = csv_path {
                                tracing::info!("User started bulk IM sending");
                                state.write().is_bulk_sending = true;
                                spawn(async move {
                                    let ctrl = ctrl.lock().await;
                                    if let Err(e) = ctrl.start_bulk_im(path, config) {
                                        let err_msg = e.to_string();
                                        tracing::error!("Bulk IM failed: {}", err_msg);
                                        state.write().error_message = Some(err_msg);
                                        state.write().is_bulk_sending = false;
                                    }
                                });
                            }
                        },
                        if state.read().is_bulk_sending { "正在批量发送..." } else { "开始批量发送" }
                    }
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
