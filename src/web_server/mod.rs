use std::convert::Infallible;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use http::{Response, StatusCode};
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn;
use tokio::net::TcpListener;
use tracing::debug;

use crate::config::ServerConfig;
use crate::web_server::router::ServerState;
use crate::web_server::web_utils::{full_body, HyperRequest, HyperResponse};

mod api_routes;
mod video_metadata;
mod video_locator;
mod libraries;
mod web_utils;
mod router;

pub async fn run(server_config: ServerConfig, web_ui_dir: PathBuf) {
	let general_config = server_config.general_config.clone();
	let server_state = Arc::new(ServerState::init(server_config, web_ui_dir).await.expect("Error initializing server state"));
	
	let http_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), general_config.server_http_port);
	
	serve(http_addr, server_state).await.unwrap();
}

async fn handle_request(request: HyperRequest, server_state: Arc<ServerState>) -> Result<HyperResponse, Infallible> {
	let path_owned = match web_utils::split_path(request.uri().path()) {
		Ok(path) => path,
		Err(_) => return Ok(Response::builder()
			.status(StatusCode::BAD_REQUEST)
			.body(full_body("Invalid path"))
			.unwrap())
	};
	
	let path: Vec<&str> = path_owned.iter().map(String::as_str).collect();
	
	Ok(router::route_request(request, &path, server_state).await)
}

async fn serve(socket_addr: SocketAddr, server_state: Arc<ServerState>) -> anyhow::Result<()> {
	let listener = TcpListener::bind(socket_addr).await?;
	
	loop {
		let (socket, _remote_addr) = listener.accept().await?;
		let server_state = server_state.clone();
		
		tokio::spawn(async move {
			let connection_builder = conn::auto::Builder::new(TokioExecutor::new());
			
			let service = service_fn(move |request| handle_request(request, server_state.clone()));
			let io = TokioIo::new(socket);
			
			if let Err(err) = connection_builder.serve_connection(io, service).await {
				debug!("Error serving connection: {:?}", err);
			}
		});
	}
}