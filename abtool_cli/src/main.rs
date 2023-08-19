use std::time::{Duration, Instant};
use chrono::Local;
use clap::Parser;
use tracing::{debug, error, Level};
use tracing_subscriber::EnvFilter;
use anyhow::Result;

fn main() -> Result<()> {
    let filter = EnvFilter::from_default_env().add_directive("abtool_cli=trace".parse().unwrap())
        .add_directive("shell=trace".parse().unwrap());
    let collector = tracing_subscriber::fmt().with_max_level(Level::TRACE).with_env_filter(filter).finish();

    tracing::subscriber::set_global_default(collector).expect("Unable to set a global collector");
    let now = Local::now();

    let formatted = now.format("%Y-%m-%d_%H-%M-%S").to_string();

    debug!("starting from {}", formatted.as_str());

    let args = Args::parse();
    debug!("action: {}", args.action);

    let config = match args.config {
        None => {
            panic!("config is None");
        }
        Some(config) => {
            debug!("config file: {}", config);

            config
        }
    };

    let start_time = Instant::now();

    match args.action.as_str() {
        "apk" => {
            debug!("build apk");
            let _apk_path = match shell::build_apk(config, formatted.as_str()) {
                Ok(path) => {
                    debug!("build success, apk path: {}", path);

                }
                Err(e) => {
                    error!("build failed: {}", e);
                    panic!("build failed: {}", e);
                }
            };
        }

        _ => {
            debug!("build aab");
            let _aab_path = match shell::build_aab(config, formatted.as_str()) {
                Ok(path) => {
                    debug!("build success, aab path: {}", path);

                }
                Err(e) => {
                    error!("build failed: {}", e);
                    panic!("build failed: {}", e);
                }
            };
        }
    }
    let end_time = Instant::now();
    let duration = time_diff(start_time, end_time);
    debug!("execution completed, taking: {}ms", duration.as_millis());
    Ok(())
}

fn time_diff(start_time: Instant, end_time: Instant) -> Duration {
    end_time.duration_since(start_time)
}

#[derive(Parser, Debug)]
#[command(long_about = None)]
#[command(name = "abtool_cli")]
#[command(author = "song")]
#[command(version = "0.1.0")]
#[command(about = "abtool_cli about")]
struct Args {
    #[arg(short, long, default_value = "aab")]
    action: String,
    #[arg(short, long, default_value = None)]
    config: Option<String>,

}