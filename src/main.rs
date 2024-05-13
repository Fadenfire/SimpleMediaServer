mod config;

use std::path::PathBuf;
use argh::FromArgs;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use crate::config::ServerConfig;

#[derive(FromArgs)]
/// rust based basic media server
struct Args {
	/// path to config directory
	#[argh(option)]
	config_dir: PathBuf,
}

#[tokio::main]
async fn main() {
	let args: Args = argh::from_env();
	
	setup_logging();
	
	let config = ServerConfig::load(args.config_dir).await.expect("Loading config");
	
	info!("Starting server");
	
	println!("{:?}", config.general_config);
}

fn setup_logging() {
	let subscriber = FmtSubscriber::builder()
		.with_max_level(Level::INFO)
		.finish();
	
	tracing::subscriber::set_global_default(subscriber)
		.expect("setting default subscriber failed");
}