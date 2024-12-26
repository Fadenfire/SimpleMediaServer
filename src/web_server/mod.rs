use std::convert::Infallible;
use std::fs::Permissions;
use std::io::Cursor;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use anyhow::anyhow;
use futures_util::future::join_all;
use http::{Response, StatusCode};
use http_body_util::BodyExt;
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn;
use rcgen::{CertificateParams, DnType, KeyPair};
use tokio::net::TcpListener;
use tokio_rustls::{rustls, TlsAcceptor};
use tower::Service;
use tower_http::services::{ServeDir, ServeFile};
use tracing::{debug, info, instrument, trace};

use server_state::ServerState;

use crate::config::ServerConfig;
use crate::web_server::web_utils::{full_body, HyperRequest, HyperResponse};

mod api_routes;
mod media_metadata;
mod video_locator;
mod libraries;
mod web_utils;
pub(crate) mod media_backend_factory;
mod services;
mod server_state;
mod auth;
mod watch_history;
mod media_connections;
mod api_types;
mod api_error;

#[instrument(skip_all)]
async fn route_request(request: HyperRequest, path: &[&str], server_state: Arc<ServerState>) -> HyperResponse {
	trace!("Request for {}", request.uri().path());
	
	match path {
		["api", tail @ ..] => api_routes::route_request(request, tail, server_state).await,
		
		_ => {
			let fallback = ServeFile::new(server_state.config.paths.web_ui_dir.join("index.html"))
				.precompressed_gzip()
				.precompressed_br();
			
			let mut serve_web_ui = ServeDir::new(&server_state.config.paths.web_ui_dir)
				.precompressed_gzip()
				.precompressed_br()
				.fallback(fallback);
			
			serve_web_ui.call(request)
				.await
				.unwrap()
				.map(|body| body.map_err(anyhow::Error::new).boxed_unsync())
		}
	}
}

pub async fn run(config: ServerConfig) {
	let server_state = Arc::new(ServerState::init(config.clone()).await
		.expect("Error initializing server state"));
	
	let mut servers = Vec::new();
	
	if config.main_config.server.enable_http {
		let http_addr = SocketAddr::new(config.main_config.server.host, config.main_config.server.http_port);
		
		servers.push(serve(http_addr, server_state.clone(), None));
	}
	
	if config.main_config.server.enable_https {
		let tls_acceptor = create_tls_acceptor(&config.paths.data_dir).await.expect("Creating TLS");
		let http_addr = SocketAddr::new(config.main_config.server.host, config.main_config.server.https_port);
		
		servers.push(serve(http_addr, server_state.clone(), Some(tls_acceptor)));
	}
	
	assert!(!servers.is_empty(), "Neither http nor https is enabled");
	
	info!("Started server");
	
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
	
	Ok(route_request(request, &path, server_state).await)
}

async fn create_tls_acceptor(data_dir: &Path) -> anyhow::Result<TlsAcceptor> {
	let certs_dir = data_dir.join("certs");
	
	tokio::fs::create_dir_all(&certs_dir).await?;
	
	let cert_path = certs_dir.join("certs.pem");
	let private_key_path = certs_dir.join("key.pem");
	
	if !tokio::fs::try_exists(&cert_path).await? || !tokio::fs::try_exists(&private_key_path).await? {
		info!("No TLS certificate found, generating self-signed cert");
		
		generate_tls_cert(&cert_path, &private_key_path).await?;
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

async fn generate_tls_cert(cert_path: &Path, private_key_path: &Path) -> anyhow::Result<()> {
	let key_pair = KeyPair::generate()?;
	
	let mut cert_params = CertificateParams::new(&["localhost".to_owned()])?;
	cert_params.distinguished_name.push(DnType::CommonName, "localhost");
	cert_params.distinguished_name.push(DnType::OrganizationName, "Rust Media Server");
	cert_params.distinguished_name.push(DnType::OrganizationalUnitName, "Self Signed");
	
	let cert = cert_params.self_signed(&key_pair)?;
	
	tokio::fs::write(&cert_path, cert.pem()).await?;
	tokio::fs::write(&private_key_path, key_pair.serialize_pem()).await?;
	
	#[cfg(unix)] {
		use std::os::unix::fs::PermissionsExt;
		
		tokio::fs::set_permissions(&private_key_path, Permissions::from_mode(0o600)).await?;
	}
	
	Ok(())
}

async fn serve(socket_addr: SocketAddr, server_state: Arc<ServerState>, tls_acceptor: Option<TlsAcceptor>) -> anyhow::Result<()> {
	info!("Listening on {} {}", socket_addr, if tls_acceptor.is_some() { "with TLS" } else { "" });
	
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
