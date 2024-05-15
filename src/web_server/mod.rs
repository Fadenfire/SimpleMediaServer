use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use tower_http::services::{ServeDir, ServeFile};

use crate::config::ServerConfig;
use crate::web_server::state::ServerState;

mod state;
mod api_routes;

pub async fn run(server_config: ServerConfig, web_ui_dir: PathBuf) {
	let general_config = server_config.general_config.clone();
	let server_state = Arc::new(ServerState::new(server_config));
	let router = build_router(server_state.clone(), web_ui_dir);
	
	let http_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), general_config.server_http_port);
	
	axum_server::bind(http_addr)
		.serve(router.into_make_service())
		.await
		.unwrap();
}

fn build_router(server_state: Arc<ServerState>, web_ui_dir: PathBuf) -> Router {
	let web_ui_index = ServeFile::new(web_ui_dir.join("index.html"))
		.precompressed_gzip()
		.precompressed_br();
	
	let web_ui = ServeDir::new(web_ui_dir)
		.precompressed_gzip()
		.precompressed_br()
		.fallback(web_ui_index);
	
	let router = Router::new()
		.nest("/api/", api_routes::build_router())
		.fallback_service(web_ui)
		.with_state(server_state);
	
	router
}