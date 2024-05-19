use std::path::PathBuf;

use argh::FromArgs;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::config::ServerConfig;

mod config;
mod web_server;
mod utils;
mod services;
mod transcoding;

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
	ffmpeg_the_third::log::set_level(ffmpeg_the_third::log::Level::Error);
	
	info!("Starting server");
	
	let config = ServerConfig::load(args.config_dir).await.expect("Loading config");
	let web_ui_dir = get_web_ui_assets_dir();
	
	if !tokio::fs::try_exists(&web_ui_dir).await.unwrap_or(false) {
		panic!("Web UI assets directory does not exist");
	}
	
	web_server::run(config, web_ui_dir).await;
}

fn setup_logging() {
	let subscriber = FmtSubscriber::builder()
		.with_max_level(Level::INFO)
		.finish();
	
	tracing::subscriber::set_global_default(subscriber)
		.expect("Setting default subscriber failed");
}

fn get_web_ui_assets_dir() -> PathBuf {
	std::env::var_os("WEB_UI_DIR")
		.map(PathBuf::from)
		.unwrap_or_else(|| {
			let exe_path = std::env::current_exe().expect("Could not get location of current executable");
			exe_path.parent().unwrap().join("web-ui")
		})
}