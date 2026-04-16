use crate::app_paths::AppPaths;
use crate::config::AppConfig;
use crate::event::{BulkPhase, BulkProgress, LiveEvent};
use anyhow::Result;
use common::export::CommentUserCsvExporter;
use im::client::IMSender;
use im::models::IMSendConfig;
use std::path::PathBuf;
use tokio::sync::mpsc;

pub struct ImRuntime {
    project_root: PathBuf,
    event_tx: mpsc::UnboundedSender<LiveEvent>,
}

impl ImRuntime {
    pub fn new(project_root: PathBuf, event_tx: mpsc::UnboundedSender<LiveEvent>) -> Self {
        Self {
            project_root,
            event_tx,
        }
    }

    pub async fn send(&self, config: AppConfig) -> Result<()> {
        let app_paths = AppPaths::new(self.project_root.clone());
        app_paths.ensure_im_runtime_files()?;

        let sender = IMSender::new(app_paths.im_sign_js())?;
        let im_config = IMSendConfig {
            cookie: config.im.cookie,
            timeout: 20,
            receiver_id: config.im.receiver_id,
            conversation_id: None,
            message_text: config.im.message_text,
        };

        sender.send(&im_config).await?;
        Ok(())
    }

    pub async fn run_bulk_send(&self, csv_path: PathBuf, config: AppConfig) -> Result<()> {
        let app_paths = AppPaths::new(self.project_root.clone());
        app_paths.ensure_im_runtime_files()?;
        let sender = IMSender::new(app_paths.im_sign_js())?; // IMSender created here, on the spawned task.

        let im_config = IMSendConfig {
            cookie: config.im.cookie,
            timeout: 20,
            receiver_id: None,
            conversation_id: None,
            message_text: config.im.message_text,
        };

        let records = CommentUserCsvExporter::read_comments(&csv_path)?;
        let total = records.len();

        let _ = self.event_tx.send(LiveEvent::ImBulkProgress(BulkProgress {
            progress: 0,
            total,
            phase: BulkPhase::Starting,
        }));

        let failed_path = csv_path.with_extension("failed.csv");
        let mut failed_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&failed_path)?;

        if failed_file.metadata()?.len() == 0 {
            use std::io::Write;
            writeln!(
                failed_file,
                "comment_time,comment_content,user_id,user_name,profile_url,error_reason"
            )?;
        }

        let mut success_count = 0;
        let mut fail_count = 0;
        let mut rng = rand::rng();

        for (i, record) in records.iter().enumerate() {
            let _ = self.event_tx.send(LiveEvent::ImBulkProgress(BulkProgress {
                progress: i + 1,
                total,
                phase: BulkPhase::Sending {
                    user_name: record.user_name.clone(),
                },
            }));

            let mut current_config = im_config.clone();
            current_config.receiver_id = Some(record.user_id.clone());

            match sender.send(&current_config).await {
                Ok(_) => {
                    success_count += 1;
                }
                Err(e) => {
                    fail_count += 1;
                    use std::io::Write;
                    let _ = writeln!(
                        failed_file,
                        "{},{},{},{},{},\"{}\"",
                        record.comment_time,
                        record.comment_content,
                        record.user_id,
                        record.user_name,
                        record.profile_url,
                        e.to_string().replace('"', "\"\"")
                    );
                }
            }

            if i < total - 1 {
                use rand::Rng;
                let delay = rng.random_range(5..=15);
                let _ = self.event_tx.send(LiveEvent::ImBulkProgress(BulkProgress {
                    progress: i + 1,
                    total,
                    phase: BulkPhase::Waiting { seconds: delay },
                }));
                tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
            }
        }

        let _ = self.event_tx.send(LiveEvent::ImBulkCompleted {
            success_count,
            fail_count,
        });

        Ok(())
    }
}
