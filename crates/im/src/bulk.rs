use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::time::Duration;

use crate::client::IMSender;
use crate::error::Result;
use crate::models::IMSendConfig;
use common::export::CommentUserCsvExporter;
use rand::Rng;
use tokio::time::sleep;
use tracing::{error, info};

pub async fn bulk_send(
    sender: &IMSender,
    csv_path: &Path,
    base_config: &IMSendConfig,
) -> Result<()> {
    let records = CommentUserCsvExporter::read_comments(csv_path)
        .map_err(|e| crate::error::IMError::Message(e.to_string()))?;
    info!(
        "Starting bulk send to {} users from {:?}",
        records.len(),
        csv_path
    );

    let failed_path = csv_path.with_extension("failed.csv");
    let mut failed_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&failed_path)
        .map_err(|e| crate::error::IMError::Fetcher(e.into()))?;

    // Write header if file is empty
    if failed_file.metadata().map(|m| m.len()).unwrap_or(0) == 0 {
        writeln!(
            failed_file,
            "comment_time,comment_content,user_id,user_name,profile_url,error_reason"
        )
        .map_err(|e| crate::error::IMError::Fetcher(e.into()))?;
    }

    let mut rng = rand::rng();

    for (i, record) in records.iter().enumerate() {
        info!(
            "[{}/{}] Sending message to {} (id: {})",
            i + 1,
            records.len(),
            record.user_name,
            record.user_id
        );

        let mut config = base_config.clone();
        config.receiver_id = Some(record.user_id.clone());

        match sender.send(&config).await {
            Ok(_) => {
                info!("Successfully sent message to {}", record.user_name);
            }
            Err(e) => {
                error!("Failed to send message to {}: {}", record.user_name, e);
                if let Err(write_err) = writeln!(
                    failed_file,
                    "{},{},{},{},{},\"{}\"",
                    record.comment_time,
                    record.comment_content,
                    record.user_id,
                    record.user_name,
                    record.profile_url,
                    e.to_string().replace('"', "\"\"")
                ) {
                    error!("Failed to write to failed log: {}", write_err);
                }
            }
        }

        // Random delay between 5 and 15 seconds
        if i < records.len() - 1 {
            let delay_secs = rng.random_range(5..=15);
            info!("Sleeping for {} seconds before next message...", delay_secs);
            sleep(Duration::from_secs(delay_secs)).await;
        }
    }

    info!(
        "Bulk send completed. Check {:?} for any failures.",
        failed_path
    );
    Ok(())
}
