use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;

use axum::Router;
use axum::routing::get;

use crate::config::ServerConfig;
use crate::web_server::state::ServerState;

mod state;
mod api_routes;

pub async fn run(server_config: ServerConfig) {
	let general_config = server_config.general_config.clone();
	let server_state = Arc::new(ServerState::new(server_config));
	let router = build_router(server_state.clone());
	
	let listener = tokio::net::TcpListener::bind((IpAddr::V4(Ipv4Addr::UNSPECIFIED), general_config.server_http_port)).await.unwrap();
	axum::serve(listener, router).await.unwrap();
}

fn build_router(server_state: Arc<ServerState>) -> Router {
	let router = Router::new()
		.nest("/api/", api_routes::build_router())
		.route("/", get(|| async { "hjfdjsfbjds" }))
		.fallback(|| async { "Not found" })
		.with_state(server_state);
	
	router
}