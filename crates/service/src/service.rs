use crate::commands::{SendImCommand, StartBulkImCommand, StartLiveCommand};
use crate::config::AppConfig;
use crate::im_runtime::ImRuntime;
use crate::live_runtime::LiveRuntime;
use anyhow::Result;
use std::path::PathBuf;

pub struct AppService {
    live_runtime: LiveRuntime,
    im_runtime: ImRuntime,
}

impl AppService {
    pub fn new(project_root: PathBuf) -> Self {
        let live_runtime = LiveRuntime::new(project_root.clone());
        let im_runtime = ImRuntime::new(project_root, live_runtime.event_tx());
        Self {
            live_runtime,
            im_runtime,
        }
    }

    pub async fn start_live(&mut self, command: StartLiveCommand) -> Result<()> {
        self.live_runtime.start(command.config).await
    }

    pub async fn stop_live(&mut self) -> Result<()> {
        self.live_runtime.stop().await
    }

    pub async fn send_im(&self, command: SendImCommand) -> Result<()> {
        self.im_runtime.send(command.config).await
    }

    pub async fn start_bulk_im(&self, command: StartBulkImCommand) -> Result<()> {
        self.im_runtime
            .run_bulk_send(command.csv_path, command.config)
            .await
    }

    pub fn load_config(&self, path: &std::path::Path) -> Result<AppConfig> {
        crate::config::load_config(path)
    }

    pub fn save_config(&self, path: &std::path::Path, config: &AppConfig) -> Result<()> {
        crate::config::save_config(path, config)
    }

    pub fn event_tx(&self) -> tokio::sync::mpsc::UnboundedSender<crate::event::LiveEvent> {
        self.live_runtime.event_tx()
    }

    pub fn take_event_receiver(
        &mut self,
    ) -> Option<tokio::sync::mpsc::UnboundedReceiver<crate::event::LiveEvent>> {
        self.live_runtime.take_event_receiver()
    }

    pub fn status(&self) -> crate::event::LiveStatus {
        self.live_runtime.status()
    }
}
