use crate::app_paths::AppPaths;
use crate::config::AppConfig;
use crate::config_store::validate_live_config;
use crate::event::{LiveEvent, LiveLog, LiveStatus};
use anyhow::{Result, bail};
use live::app;
use std::path::PathBuf;
use std::thread;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

pub struct LiveRuntime {
    project_root: PathBuf,
    status: LiveStatus,
    task: Option<JoinHandle<()>>,
    event_tx: mpsc::UnboundedSender<LiveEvent>,
    event_rx: Option<mpsc::UnboundedReceiver<LiveEvent>>,
}

impl LiveRuntime {
    pub fn new(project_root: PathBuf) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        Self {
            project_root,
            status: LiveStatus::Idle,
            task: None,
            event_tx,
            event_rx: Some(event_rx),
        }
    }

    pub fn status(&self) -> LiveStatus {
        self.status
    }

    pub fn event_tx(&self) -> mpsc::UnboundedSender<LiveEvent> {
        self.event_tx.clone()
    }

    pub fn take_event_receiver(&mut self) -> Option<mpsc::UnboundedReceiver<LiveEvent>> {
        self.event_rx.take()
    }

    pub async fn start(&mut self, config: AppConfig) -> Result<()> {
        if self.task.as_ref().is_some_and(|task| !task.is_finished()) {
            bail!("live session is already running");
        }
        self.task = None;

        let app_paths = AppPaths::new(self.project_root.clone());
        validate_live_config(&config)?;
        app_paths.ensure_live_runtime_files()?;

        self.status = LiveStatus::Starting;
        let _ = self.event_tx.send(LiveEvent::StatusChanged(self.status));

        let room_id = config.live.id.clone();
        let project_root = self.project_root.clone();
        let tx = self.event_tx.clone();
        self.task = Some(tokio::task::spawn_blocking(move || {
            let _ = tx.send(LiveEvent::Log(LiveLog {
                level: crate::event::LogLevel::Info,
                message: format!("Starting room {room_id}"),
            }));

            let thread_tx = tx.clone();
            let result = thread::scope(|scope| {
                let handle = scope.spawn(|| {
                    let runtime = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .expect("failed to create live runtime thread");
                    runtime.block_on(app::run_live(&project_root, config))
                });
                handle.join().unwrap_or_else(|_| {
                    Err(live::error::LiveError::Fetcher(
                        common::error::FetcherError::Internal("live thread panicked".to_string()),
                    ))
                })
            });
            let status = if result.is_ok() {
                LiveStatus::Stopped
            } else {
                LiveStatus::Failed
            };
            let _ = thread_tx.send(LiveEvent::StatusChanged(status));
            if let Err(error) = result {
                let _ = thread_tx.send(LiveEvent::Log(LiveLog {
                    level: crate::event::LogLevel::Error,
                    message: format!("Live fetcher error: {error}"),
                }));
            }
        }));

        self.status = LiveStatus::Running;
        let _ = self.event_tx.send(LiveEvent::StatusChanged(self.status));
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<()> {
        let Some(task) = self.task.take() else {
            bail!("no active session to stop");
        };

        self.status = LiveStatus::Stopping;
        let _ = self.event_tx.send(LiveEvent::StatusChanged(self.status));
        task.abort();
        self.status = LiveStatus::Stopped;
        let _ = self.event_tx.send(LiveEvent::StatusChanged(self.status));
        Ok(())
    }
}
