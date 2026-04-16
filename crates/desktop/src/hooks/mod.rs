use crate::controller::AppController;
use crate::state::AppState;
use dioxus::prelude::*;
use service::event::{LiveEvent, LiveLog, LiveStatus, LogLevel};

pub fn use_app_events(state: Signal<AppState>, controller: AppController) {
    use_hook(move || {
        let mut state = state;
        let controller = controller.clone();
        spawn(async move {
            let rx = {
                let mut ctrl = controller.lock().await;
                ctrl.take_event_receiver()
            };

            if let Some(mut rx) = rx {
                while let Some(event) = rx.recv().await {
                    let mut s = state.write();
                    crate::state::reduce_event(&mut s, event.clone());

                    match event {
                        LiveEvent::StatusChanged(_status)
                            if s.live_status == LiveStatus::Failed =>
                        {
                            rfd::MessageDialog::new()
                                .set_title("System Error")
                                .set_description("The live fetcher encountered a critical error and has stopped.")
                                .set_level(rfd::MessageLevel::Error)
                                .show();
                        }
                        LiveEvent::Log(LiveLog {
                            level: LogLevel::Error,
                            message,
                        }) => {
                            rfd::MessageDialog::new()
                                .set_title("Fetcher Error")
                                .set_description(&message)
                                .set_level(rfd::MessageLevel::Error)
                                .show();
                        }
                        _ => {}
                    }
                }
            }
        });
    });
}
