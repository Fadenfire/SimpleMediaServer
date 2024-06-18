use std::convert::Infallible;
use std::fs::Permissions;
use std::io::Cursor;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use anyhow::anyhow;
use futures_util::future::join_all;

use http::{Response, StatusCode};
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn;
use tokio::net::TcpListener;
use tokio_rustls::{rustls, TlsAcceptor};
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
pub(crate) mod media_backend_factory;
mod services;

pub async fn run(server_config: ServerConfig, web_ui_dir: PathBuf) {
	let server_config2 = server_config.clone();
	
	let server_state = Arc::new(ServerState::init(server_config, web_ui_dir).await
		.expect("Error initializing server state"));
	
	let mut servers = Vec::new();
	
	let http_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), server_config2.main_config.server.http_port);
	servers.push(serve(http_addr, server_state.clone(), None));
	
	if server_config2.main_config.server.enable_https {
		let tls_acceptor = create_tls_acceptor(&server_config2.paths.data_dir).await.expect("Creating TLS");
		let http_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), server_config2.main_config.server.https_port);
		
		servers.push(serve(http_addr, server_state.clone(), Some(tls_acceptor)));
	}
	
	join_all(servers).await
		.into_iter()
		.collect::<Result<Vec<_>, _>>()
		.expect("Server failed");
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

async fn serve(socket_addr: SocketAddr, server_state: Arc<ServerState>, tls_acceptor: Option<TlsAcceptor>) -> anyhow::Result<()> {
	let listener = TcpListener::bind(socket_addr).await?;
	
	loop {
		let (socket, _remote_addr) = listener.accept().await?;
		let server_state = server_state.clone();
		let tls_acceptor = tls_acceptor.clone();
		
		tokio::spawn(async move {
			let connection_builder = conn::auto::Builder::new(TokioExecutor::new());
			let service = service_fn(move |request| handle_request(request, server_state.clone()));
			
			let result = if let Some(tls_acceptor) = tls_acceptor {
				match tls_acceptor.accept(socket).await {
					Ok(tls_stream) => connection_builder.serve_connection(TokioIo::new(tls_stream), service).await,
					Err(err) => {
						debug!("Error completing TLS handshake: {:?}", err);
						return;
					}
				}
			} else {
				connection_builder.serve_connection(TokioIo::new(socket), service).await
			};
			
			if let Err(err) = result {
				debug!("Error serving connection: {:?}", err);
			}
		});
	}
}

async fn create_tls_acceptor(data_dir: &Path) -> anyhow::Result<TlsAcceptor> {
	let certs_dir = data_dir.join("certs");
	
	tokio::fs::create_dir_all(&certs_dir).await?;
	
	let cert_path = certs_dir.join("certs.pem");
	let private_key_path = certs_dir.join("key.pem");
	
	if !tokio::fs::try_exists(&cert_path).await? || !tokio::fs::try_exists(&private_key_path).await? {
		let cert = rcgen::generate_simple_self_signed(&["localhost".to_owned()])?;
		
		tokio::fs::write(&cert_path, cert.cert.pem()).await?;
		tokio::fs::write(&private_key_path, cert.key_pair.serialize_pem()).await?;
		
		#[cfg(unix)]
		tokio::fs::set_permissions(&private_key_path, Permissions::from_mode(0o600)).await?;
	}
	
	let mut cert_reader = Cursor::new(tokio::fs::read(&cert_path).await?);
	let mut private_key_reader = Cursor::new(tokio::fs::read(&private_key_path).await?);
	
	let certs = rustls_pemfile::certs(&mut cert_reader).collect::<Result<Vec<_>, _>>()?;
	let private_key = rustls_pemfile::private_key(&mut private_key_reader)?
		.ok_or_else(|| anyhow!("Key file contains no private keys"))?;
	
	let tls_config = rustls::ServerConfig::builder()
		.with_no_client_auth()
		.with_single_cert(certs, private_key)?;
	
	Ok(TlsAcceptor::from(Arc::new(tls_config)))
}