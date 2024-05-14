use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;

use warp::{Filter, Reply};
use warp::filters::BoxedFilter;
use warp::path::Tail;

use crate::config::ServerConfig;
use crate::web_server::state::ServerState;

mod state;
mod api_routes;

pub async fn serve(server_config: ServerConfig) {
	let general_config = server_config.general_config.clone();
	let server_state = Arc::new(ServerState::new(server_config));
	let routes = build_routes(server_state.clone());
	
	warp::serve(routes)
		.bind((IpAddr::V4(Ipv4Addr::UNSPECIFIED), general_config.server_http_port))
		.await;
}

fn build_routes(server_state: Arc<ServerState>) -> BoxedFilter<(impl Reply, )> {
	let api_routes = warp::path("api")
		.and(api_routes::build_routes(server_state));
	
	let route = warp::get()
		.and(warp::path::tail())
		.map(|tail: Tail| format!("test path: {}", tail.as_str()));
	
	api_routes
		.or(route)
		.boxed()
}