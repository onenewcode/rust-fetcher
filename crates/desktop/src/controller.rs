use anyhow::Result;
use dioxus::prelude::*;
use service::app_paths::AppPaths;
use service::config::AppConfig;
use service::config_store::{load_config, save_config};
use service::open::open_path;
use service::service::AppService;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Controller {
    app_paths: AppPaths,
    service: AppService,
}

impl Controller {
    pub fn new(project_root: PathBuf) -> Self {
        let app_paths = AppPaths::new(project_root.clone());
        let service = AppService::new(project_root);
        Self { app_paths, service }
    }

    pub fn load_config(&self) -> Result<AppConfig> {
        load_config(&self.app_paths.config_file())
    }

    pub fn save_config(&self, config: &AppConfig) -> Result<()> {
        save_config(&self.app_paths.config_file(), config)
    }

    pub async fn start(&mut self, config: AppConfig) -> Result<()> {
        save_config(&self.app_paths.config_file(), &config)?;
        self.service
            .start_live(service::commands::StartLiveCommand::new(config))
            .await
    }

    pub async fn stop(&mut self) -> Result<()> {
        self.service.stop_live().await
    }

    pub fn open_logs(&self) -> Result<()> {
        open_path(&self.app_paths.logs_dir())
    }

    pub fn open_exports(&self) -> Result<()> {
        open_path(&self.app_paths.exports_dir())
    }

    pub fn take_event_receiver(
        &mut self,
    ) -> Option<tokio::sync::mpsc::UnboundedReceiver<service::event::LiveEvent>> {
        self.service.take_event_receiver()
    }

    pub async fn send_im(&self, config: AppConfig) -> Result<()> {
        self.service
            .send_im(service::commands::SendImCommand { config })
            .await
    }

    pub fn start_bulk_im(&self, csv_path: PathBuf, config: AppConfig) -> Result<()> {
        let project_root = self.app_paths.project_root().to_path_buf();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async move {
                let service = AppService::new(project_root);
                if let Err(e) = service
                    .start_bulk_im(service::commands::StartBulkImCommand { csv_path, config })
                    .await
                {
                    let event_tx = service.event_tx();
                    let _ = event_tx.send(service::event::LiveEvent::ImBulkError(
                        service::event::ImBulkError {
                            message: e.to_string(),
                        },
                    ));
                }
            });
        });
        Ok(())
    }
}

pub type AppController = Arc<Mutex<Controller>>;

pub fn use_controller() -> AppController {
    use_context::<AppController>()
}
