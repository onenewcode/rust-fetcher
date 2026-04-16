use anyhow::Context as _;
use clap::{Parser, Subcommand};
use common::config::AppConfig;
use common::logging::init_tracing;
use service::app_paths::AppPaths;
use service::commands::{SendImCommand, StartBulkImCommand, StartLiveCommand};
use service::service::AppService;
use std::env;

#[derive(Parser)]
#[command(name = "fetcher-cli")]
#[command(about = "Douyin Fetcher CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Live room data fetcher
    Live {
        /// Room ID to fetch (overrides config.yaml)
        #[arg(short, long)]
        room_id: Option<String>,
        /// Cookie string (overrides config.yaml)
        #[arg(short, long)]
        cookie: Option<String>,
    },
    /// IM message sender
    Im {
        /// Receiver ID (overrides config.yaml)
        #[arg(short, long)]
        receiver_id: Option<String>,
        /// Message text (overrides config.yaml)
        #[arg(short, long)]
        message: Option<String>,
        /// Cookie string (overrides config.yaml)
        #[arg(short, long)]
        cookie: Option<String>,
        /// Bulk sending from CSV file
        #[arg(short, long)]
        bulk: Option<std::path::PathBuf>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let project_root = env::current_dir()?;
    let app_paths = AppPaths::new(project_root.clone());
    let config_path = app_paths.config_file();

    // Load config
    let mut config =
        AppConfig::load(&config_path).context("Failed to load application configuration")?;
    let _guard = init_tracing("info", Some(app_paths.logs_dir()))?;

    match cli.command {
        Commands::Live { room_id, cookie } => {
            if let Some(r) = room_id {
                config.live.id = r;
            }
            if let Some(c) = cookie {
                config.live.cookie = c;
            }

            let mut service = AppService::new(project_root.clone());
            service
                .start_live(StartLiveCommand::new(config))
                .await
                .context("Failed to run live fetcher")?;
        }
        Commands::Im {
            receiver_id,
            message,
            cookie,
            bulk,
        } => {
            if let Some(c) = cookie {
                config.im.cookie = c;
            }
            if let Some(r) = receiver_id {
                config.im.receiver_id = Some(r);
            }
            if let Some(m) = message {
                config.im.message_text = Some(m);
            }

            let service = AppService::new(project_root.clone());
            if let Some(csv_path) = bulk {
                service
                    .start_bulk_im(StartBulkImCommand { csv_path, config })
                    .await
                    .context("Failed to perform bulk IM sending")?;
            } else {
                service
                    .send_im(SendImCommand { config })
                    .await
                    .context("Failed to send IM message")?;
            }
        }
    }

    Ok(())
}
