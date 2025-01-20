use crate::config::UsersConfig;
use crate::web_server::api_error::ApiError;
use argon2::password_hash::SaltString;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use headers::{Cookie, HeaderMapExt};
use http::HeaderMap;
use jsonwebtoken::{DecodingKey, EncodingKey, Validation};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::Permissions;
use std::path::Path;
use std::time::{Duration, SystemTime};

pub const AUTH_COOKIE_NAME: &str = "media_server_access_token";
pub const AUTH_TOKEN_LIFETIME: Duration = Duration::from_secs(60 * 60 * 24 * 365); // 1 year

pub struct AuthManager {
	users: HashMap<String, User>,
	username_to_id: HashMap<String, String>,
	secrets: AuthSecrets,
}

pub struct AuthSecrets {
	jwt_key: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct JwtClaims {
	sub: String,
	exp: u64,
}

impl AuthManager {
	pub fn from_config(users_config: UsersConfig, secrets: AuthSecrets) -> Self {
		let argon = Argon2::default();
		
		let users: HashMap<String, User> = users_config.users.into_iter()
			.map(|cfg| {
				let salt = SaltString::generate(&mut OsRng);
				let password_hash = argon.hash_password(cfg.password.as_bytes(), &salt).unwrap();
				
				let user = User {
					id: cfg.id.clone(),
					display_name: cfg.display_name,
					username: cfg.username,
					allowed_libraries: cfg.allowed_libraries,
					
					password_hash: password_hash.to_string(),
				};
				
				(cfg.id, user)
			})
			.collect();
		
		let username_to_id = users.values()
			.map(|user| (user.username.clone(), user.id.clone()))
			.collect();
		
		Self {
			users,
			username_to_id,
			secrets,
		}
	}
	
	pub fn iter_users(&self) -> impl Iterator<Item = &User> {
		self.users.values()
	}
	
	pub fn get_user_by_id(&self, id: &str) -> Option<&User> {
		self.users.get(id)
	}
	
	pub fn decode_token(&self, token: &str) -> anyhow::Result<&User> {
		let claims = jsonwebtoken::decode::<JwtClaims>(
			token,
			&DecodingKey::from_secret(&self.secrets.jwt_key),
			&Validation::default()
		)?.claims;
		
		let user_id = claims.sub;
		
		self.get_user_by_id(&user_id).ok_or_else(|| anyhow::anyhow!("Unknown user id"))
	}
	
	pub fn generate_token(&self, user: &User) -> String {
		let expire_time = SystemTime::now() + AUTH_TOKEN_LIFETIME;
		
		let expire_time_unix = expire_time.duration_since(std::time::UNIX_EPOCH)
			.expect("Time went backwards")
			.as_secs();
		
		let claims = JwtClaims {
			sub: user.id.to_owned(),
			exp: expire_time_unix,
		};
		
		jsonwebtoken::encode(
			&jsonwebtoken::Header::default(),
			&claims,
			&EncodingKey::from_secret(&self.secrets.jwt_key)
		).expect("Failed to generate JWT token")
	}
	
	pub fn login(&self, username: &str, password: &str) -> anyhow::Result<&User> {
		let user_id = self.username_to_id.get(username)
			.ok_or_else(|| anyhow::anyhow!("Unknown username"))?;
		
		let user = &self.users[user_id];
		
		if !user.verify_password(password) {
			return Err(anyhow::anyhow!("Invalid password"));
		}
		
		Ok(user)
	}
	
	pub fn lookup_from_headers(&self, headers: &HeaderMap) -> Result<&User, ApiError> {
		let cookies = headers.typed_get::<Cookie>();
		
		cookies.as_ref()
			.and_then(|cookies| cookies.get(AUTH_COOKIE_NAME))
			.and_then(|auth_token| self.decode_token(auth_token).ok())
			.ok_or(ApiError::Unauthorized)
	}
}

pub struct User {
	pub id: String,
	pub display_name: String,
	pub username: String,
	pub allowed_libraries: Vec<String>,
	
	password_hash: String,
}

impl User {
	pub fn verify_password(&self, password: &str) -> bool {
		let password_hash = PasswordHash::new(&self.password_hash).unwrap();
		
		Argon2::default().verify_password(password.as_bytes(), &password_hash).is_ok()
	}
	
	pub fn can_see_library(&self, library_id: &str) -> bool {
		self.allowed_libraries.iter().any(|s| s == library_id)
	}
}

impl AuthSecrets {
	pub fn generate() -> Self {
		let mut jwt_key = vec![0u8; 32];
		OsRng.fill_bytes(&mut jwt_key);
		
		Self {
			jwt_key,
		}
	}
	
	pub async fn load_from_file(path: &Path) -> anyhow::Result<Self> {
		if let Ok(file_data) = tokio::fs::read(path).await {
			let secrets: AuthSecretsSerialized = serde_json::from_slice(&file_data)?;
			
			Ok(Self {
				jwt_key: hex::decode(&secrets.jwt_key)?,
			})
		} else {
			let secrets = Self::generate();
			
			let secrets_ser = AuthSecretsSerialized {
				jwt_key: hex::encode(&secrets.jwt_key),
			};
			
			tokio::fs::write(path, serde_json::to_vec_pretty(&secrets_ser)?).await?;
			
			#[cfg(unix)] {
				use std::os::unix::fs::PermissionsExt;
				
				tokio::fs::set_permissions(&path, Permissions::from_mode(0o600)).await?;
			}
			
			Ok(secrets)
		}
	}
}

#[derive(Serialize, Deserialize)]
struct AuthSecretsSerialized {
	jwt_key: String,
}

#[cfg(test)]
mod tests {
	use crate::config::{UserConfig, UsersConfig};
	use crate::web_server::auth::{AuthManager, AuthSecrets, User};
	use argon2::password_hash::SaltString;
	use argon2::{Argon2, PasswordHasher};
	use http::header::COOKIE;
	use http::HeaderMap;
	use rand::rngs::OsRng;
	
	fn create_test_user() -> User {
		let salt = SaltString::generate(&mut OsRng);
		let password_hash = Argon2::default().hash_password(b"hunter42", &salt).unwrap();
		
		User {
			id: "joe".to_string(),
			display_name: "Joe".to_string(),
			username: "joe".to_string(),
			allowed_libraries: vec!["lib_a".to_string(), "lib_b".to_string()],
			password_hash: password_hash.to_string(),
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
		
		assert!(user.can_see_library("lib_a"));
		
		assert!(user.can_see_library("lib_b"));
		
		assert!(!user.can_see_library("lib_c"));
	}
	
	fn create_test_user_config() -> UsersConfig {
		UsersConfig {
			users: vec![
				UserConfig {
					id: "joe".to_string(),
					display_name: "Joe Moe".to_string(),
					username: "joemoe".to_string(),
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
		let secrets = AuthSecrets::generate();
		let auth_manager = AuthManager::from_config(create_test_user_config(), secrets);
		
		let joe = &auth_manager.users["joe"];
		let bob = &auth_manager.users["bob"];
		
		assert_eq!(joe.id, "joe");
		assert!(joe.verify_password("hunter42"));
		
		assert_eq!(bob.id, "bob");
		assert!(bob.verify_password("hfudsfh8ffhuuihufu9"));
		
		assert_eq!(auth_manager.get_user_by_id("joe").unwrap().id, "joe");
		assert_eq!(auth_manager.get_user_by_id("bob").unwrap().id, "bob");
		assert!(auth_manager.get_user_by_id("rick").is_none());
		assert!(auth_manager.get_user_by_id("joe ").is_none());
		
		assert!(auth_manager.login("zoe", "dnjsdnjsanj").is_err());
		assert!(auth_manager.login("bob", "dnjsdnjsanj").is_err());
		assert!(auth_manager.login("joemoe", "dnjsdnjsanj").is_err());
		assert!(auth_manager.login("bob", "hunter42").is_err());
		assert!(auth_manager.login("fsdfdss", "hunter42").is_err());
		assert!(auth_manager.login("joemoe", "hunter43").is_err());
		
		let joe_login = auth_manager.login("joemoe", "hunter42").unwrap();
		assert_eq!(joe_login.id, "joe");
		
		let joe_token = auth_manager.generate_token(joe_login);
		
		assert_eq!(auth_manager.decode_token(&joe_token).unwrap().id, "joe");
		assert!(auth_manager.decode_token("ababbababababababbabababbbbabababbabababbabbababbabababa").is_err());
		assert!(auth_manager.decode_token("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiJqb2UiLCJpYXQiOjE1MTYyMzkwMjJ9.lQlGe-9e923jmy7mx6unjwWX-erhJ7KsNW0H3H8shcA").is_err());
		
		let mut headers = HeaderMap::new();
		headers.insert(COOKIE, format!("media_server_access_token={}", joe_token).parse().unwrap());
		
		assert_eq!(auth_manager.lookup_from_headers(&headers).unwrap().id, "joe");
	}
}