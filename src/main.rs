mod config;

use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use clap::Parser;
use crate::config::ServerConfig;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
	/// Path to config directory
	#[arg(long)]
	config_dir: PathBuf,
}

#[tokio::main]
async fn main() {
	let args = Args::parse();
	
	setup_logging();
	
	let config = ServerConfig::load(args.config_dir).await.expect("Loading config");
	
	info!("Starting server");
	
	
}

fn setup_logging() {
	let subscriber = FmtSubscriber::builder()
		.with_max_level(Level::INFO)
		.finish();
	
	tracing::subscriber::set_global_default(subscriber)
		.expect("setting default subscriber failed");
}