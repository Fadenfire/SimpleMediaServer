use std::path::PathBuf;

use argh::FromArgs;
use tracing::{info, Level};
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::config::ServerConfig;

mod config;
mod web_server;
mod utils;
mod media_manipulation;

#[derive(FromArgs)]
/// rust based basic media server
struct Args {
	/// path to the directory containing config files and server data
	#[argh(option, default = "PathBuf::from(\"\")")]
	data_dir: PathBuf,
	/// path to the directory containing cache files
	#[argh(option, default = "PathBuf::from(\"cache\")")]
	cache_dir: PathBuf,
}

#[tokio::main]
async fn main() {
	let args: Args = argh::from_env();
	
	setup_logging();
	ffmpeg_next::log::set_level(ffmpeg_next::log::Level::Warning);
	
	// let data = transcode_segment(TranscodingOptions {
	// 	backend_factory: &MediaBackendFactory::new(),
	// 	media_path: PathBuf::from(std::env::args().nth(1).unwrap()),
	// 	time_range: 100..105,
	// 	target_video_height: 1080,
	// 	target_video_framerate: 60,
	// 	video_bitrate: 12_000_000,
	// 	audio_bitrate: 160_000,
	// }).unwrap();
	//
	// tokio::fs::write("output.ts", &data).await.unwrap();
	
	info!("Starting server");
	
	let config = ServerConfig::load(args.data_dir, args.cache_dir).await.expect("Loading config");
	
	web_server::run(config).await;
}

fn setup_logging() {
	let filter = EnvFilter::builder()
		.with_default_directive(Level::INFO.into())
		.with_env_var("SERVER_LOG")
		.from_env_lossy();
	
	let subscriber = FmtSubscriber::builder()
		.with_max_level(Level::DEBUG)
		.with_env_filter(filter)
		.finish();
	
	tracing::subscriber::set_global_default(subscriber)
		.expect("Setting default subscriber failed");
}
