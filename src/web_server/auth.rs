use std::collections::HashMap;

use headers::{Cookie, HeaderMapExt};
use hex::ToHex;
use http::HeaderMap;
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;

use crate::config::UsersConfig;
use crate::web_server::api_routes::error::ApiError;
use crate::web_server::libraries::Library;

pub const AUTH_COOKIE_NAME: &str = "media_server_access_token";
pub const AUTH_TOKEN_LENGTH: usize = 32;

pub struct AuthManager {
	users: HashMap<String, User>,
	token_to_id: HashMap<String, String>,
	username_to_id: HashMap<String, String>,
}

impl AuthManager {
	pub fn from_config(users_config: UsersConfig) -> Self {
		let users: HashMap<String, User> = users_config.users.iter()
			.map(Clone::clone)
			.map(|cfg| {
				let mut auth_token_bytes = [0u8; AUTH_TOKEN_LENGTH];
				pbkdf2_hmac::<Sha256>(format!("{}/{}", cfg.username, cfg.password).as_bytes(), b"isasalt", 100_000, &mut auth_token_bytes);
				let auth_token = auth_token_bytes.encode_hex();
				
				let user = User {
					id: cfg.id.clone(),
					display_name: cfg.display_name,
					username: cfg.username,
					allowed_libraries: cfg.allowed_libraries,
					
					password: cfg.password,
					auth_token,
				};
				
				(cfg.id, user)
			})
			.collect();
		
		let token_to_id = users.values()
			.map(|user| (user.auth_token.clone(), user.id.clone()))
			.collect();
		
		let username_to_id = users.values()
			.map(|user| (user.username.clone(), user.id.clone()))
			.collect();
		
		Self {
			users,
			token_to_id,
			username_to_id,
		}
	}
	
	pub fn iter_users(&self) -> impl Iterator<Item = &User> {
		self.users.values()
	}
	
	pub fn get_user_by_id(&self, id: &str) -> Option<&User> {
		self.users.get(id)
	}
	
	pub fn lookup_token(&self, token: &str) -> Option<&User> {
		let user_id = self.token_to_id.get(token)?;
		self.get_user_by_id(user_id)
	}
	
	pub fn login(&self, username: &str, password: &str) -> Option<&User> {
		let user_id = self.username_to_id.get(username)?;
		
		self.get_user_by_id(user_id).filter(|user| user.verify_password(password))
	}
	
	pub fn lookup_from_headers(&self, headers: &HeaderMap) -> Result<&User, ApiError> {
		let cookies = headers.typed_get::<Cookie>();
		
		cookies.as_ref()
			.and_then(|cookies| cookies.get(AUTH_COOKIE_NAME))
			.and_then(|auth_token| self.lookup_token(auth_token))
			.ok_or(ApiError::Unauthorized)
	}
}

pub struct User {
	pub id: String,
	pub display_name: String,
	pub username: String,
	pub allowed_libraries: Vec<String>,
	
	pub auth_token: String,
	password: String,
}

impl User {
	pub fn verify_password(&self, password: &str) -> bool {
		self.password == password
	}
	
	pub fn can_see_library(&self, library: &Library) -> bool {
		self.allowed_libraries.contains(&library.id)
	}
}

#[cfg(test)]
mod tests {
	use http::header::COOKIE;
	use http::HeaderMap;
	
	use crate::config::{UserConfig, UsersConfig};
	use crate::web_server::auth::{AuthManager, User};
	use crate::web_server::libraries::Library;
	
	fn create_test_user() -> User {
		User {
			id: "joe".to_string(),
			display_name: "Joe".to_string(),
			username: "joe".to_string(),
			allowed_libraries: vec!["lib_a".to_string(), "lib_b".to_string()],
			auth_token: "abcd".to_string(),
			password: "hunter42".to_string(),
		}
	}
	
	#[test]
	fn test_verify_password() {
		let user = create_test_user();
		
		assert!(user.verify_password("hunter42"));
		assert!(!user.verify_password("ddasdada"));
		assert!(!user.verify_password("h4i3hv8478*&YGY*"));
		assert!(!user.verify_password(" hunter42"));
		assert!(!user.verify_password("hunter42  "));
		assert!(!user.verify_password("  hunter42   "));
	}
	
	#[test]
	fn test_can_see_library() {
		let user = create_test_user();
		
		assert!(user.can_see_library(&Library {
			id: "lib_a".to_string(),
			display_name: "Lib fds".to_string(),
			..Default::default()
		}));
		
		assert!(user.can_see_library(&Library {
			id: "lib_b".to_string(),
			display_name: "Lib fjgf".to_string(),
			..Default::default()
		}));
		
		assert!(!user.can_see_library(&Library {
			id: "lib_c".to_string(),
			display_name: "lib_b".to_string(),
			..Default::default()
		}));
	}
	
	fn create_test_user_config() -> UsersConfig {
		UsersConfig {
			users: vec![
				UserConfig {
					id: "joe".to_string(),
					display_name: "Joe Moe".to_string(),
					username: "joe".to_string(),
					password: "hunter42".to_string(),
					allowed_libraries: vec!["lib_a".to_string(), "lib_b".to_string()],
				},
				UserConfig {
					id: "bob".to_string(),
					display_name: "Bob Kleuksi".to_string(),
					username: "bobk".to_string(),
					password: "hfudsfh8ffhuuihufu9".to_string(),
					allowed_libraries: vec!["lib_c".to_string()],
				},
			],
		}
	}
	
	#[test]
	fn test_auth_manager_init() {
		let auth_manager = AuthManager::from_config(create_test_user_config());
		
		let joe = &auth_manager.users["joe"];
		let bob = &auth_manager.users["bob"];
		
		assert_eq!(joe.id, "joe");
		assert_eq!(joe.password, "hunter42");
		assert_eq!(joe.auth_token, "fb27cd4780b060dfb19ac00d27222fa66d8a30c2eb1841a5b74dbf9f2280d014");
		
		assert_eq!(bob.id, "bob");
		assert_eq!(bob.password, "hfudsfh8ffhuuihufu9");
		assert_eq!(bob.auth_token, "0757dfa7bc5293d296cb48a26df087f9ad1356956d055959e7cbead30b44c18a");
		
		assert_eq!(auth_manager.get_user_by_id("joe").unwrap().id, "joe");
		assert_eq!(auth_manager.get_user_by_id("bob").unwrap().id, "bob");
		assert!(auth_manager.get_user_by_id("rick").is_none());
		assert!(auth_manager.get_user_by_id("joe ").is_none());
		
		assert_eq!(auth_manager.lookup_token("fb27cd4780b060dfb19ac00d27222fa66d8a30c2eb1841a5b74dbf9f2280d014").unwrap().id, "joe");
		assert_eq!(auth_manager.lookup_token("0757dfa7bc5293d296cb48a26df087f9ad1356956d055959e7cbead30b44c18a").unwrap().id, "bob");
		assert!(auth_manager.lookup_token("ababbababababababbabababbbbabababbabababbabbababbabababa").is_none());
		
		let mut headers = HeaderMap::new();
		headers.insert(COOKIE, "media_server_access_token=fb27cd4780b060dfb19ac00d27222fa66d8a30c2eb1841a5b74dbf9f2280d014".parse().unwrap());
		
		assert_eq!(auth_manager.lookup_from_headers(&headers).unwrap().id, "joe");
	}
}