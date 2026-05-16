use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use codescope::file_search;
use codescope::content_search;
use codescope::utils;
use std::fs;
use tempfile::TempDir;

/// Create a temporary directory with test files for benchmarking.
fn setup_test_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir_all(src_dir.join("auth")).unwrap();
    fs::create_dir_all(src_dir.join("handlers")).unwrap();

    // Create realistic Rust source files
    let files = vec![
        ("src/main.rs", MAIN_RS),
        ("src/lib.rs", LIB_RS),
        ("src/auth/mod.rs", AUTH_MOD_RS),
        ("src/auth/handler.rs", AUTH_HANDLER_RS),
        ("src/auth/middleware.rs", AUTH_MIDDLEWARE_RS),
        ("src/handlers/user.rs", USER_HANDLER_RS),
        ("src/handlers/config.rs", CONFIG_HANDLER_RS),
        ("src/utils.rs", UTILS_RS),
        ("src/types.rs", TYPES_RS),
        ("src/errors.rs", ERRORS_RS),
        ("Cargo.toml", CARGO_TOML),
        ("README.md", README_MD),
    ];

    for (path, content) in &files {
        fs::write(dir.path().join(path), content).unwrap();
    }

    dir
}

const MAIN_RS: &str = r#"
use auth::authenticate;
use handlers::{handle_request, Route};
use types::Config;

fn main() {
    let config = Config::load("config.toml").unwrap();
    let server = Server::new(config);
    server.run();
}

pub struct Server {
    config: Config,
    routes: Vec<Route>,
}

impl Server {
    pub fn new(config: Config) -> Self {
        let routes = handlers::build_routes();
        Server { config, routes }
    }

    pub fn run(&self) {
        println!("Server running on port {}", self.config.port);
        for route in &self.routes {
            println!("  {} {}", route.method, route.path);
        }
    }

    pub fn handle_connection(&self, conn: Connection) -> Result<Response, Error> {
        let auth = authenticate(&conn.token, &self.config.jwt_secret)?;
        let response = handle_request(&conn.request, &auth, &self.routes)?;
        Ok(response)
    }
}
"#;

const LIB_RS: &str = r#"
pub mod auth;
pub mod handlers;
pub mod utils;
pub mod types;
pub mod errors;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
"#;

const AUTH_MOD_RS: &str = r#"
use errors::AuthError;
use types::{User, Token, Claims};

pub fn authenticate(token: &str, secret: &str) -> Result<Claims, AuthError> {
    if token.is_empty() {
        return Err(AuthError::MissingToken);
    }
    let claims = Token::decode(token, secret)
        .map_err(AuthError::InvalidToken)?;

    if claims.is_expired() {
        return Err(AuthError::TokenExpired);
    }

    Ok(claims)
}

pub fn authorize(claims: &Claims, required_role: &str) -> Result<(), AuthError> {
    if !claims.roles.contains(&required_role.to_string()) {
        return Err(AuthError::Unauthorized);
    }
    Ok(())
}

pub fn refresh_token(token: &str, secret: &str) -> Result<String, AuthError> {
    let mut claims = authenticate(token, secret)?;
    claims.refresh();
    let new_token = Token::encode(&claims, secret);
    Ok(new_token)
}
"#;

const AUTH_HANDLER_RS: &str = r#"
use auth::{authenticate, authorize};
use types::{Request, Response, Claims};
use errors::HandlerError;

pub async fn login_handler(req: &Request) -> Result<Response, HandlerError> {
    let username = req.body.get("username")
        .ok_or(HandlerError::MissingField("username"))?;
    let password = req.body.get("password")
        .ok_or(HandlerError::MissingField("password"))?;

    let user = find_user(username, password)?;
    let token = create_session_token(&user)?;

    Ok(Response::json(serde_json::json!({
        "token": token,
        "user": user,
    })))
}

pub async fn logout_handler(claims: &Claims) -> Result<Response, HandlerError> {
    revoke_session(&claims.session_id)?;
    Ok(Response::json(serde_json::json!({"status": "ok"})))
}

fn find_user(username: &str, password: &str) -> Result<User, HandlerError> {
    let db = get_database_connection()?;
    let user = db.query_user(username)
        .map_err(HandlerError::DatabaseError)?;

    if !verify_password(password, &user.password_hash)? {
        return Err(HandlerError::InvalidCredentials);
    }

    Ok(user)
}
"#;

const AUTH_MIDDLEWARE_RS: &str = r#"
use auth::authenticate;
use types::{Request, Response, Middleware, Next};
use errors::AuthError;

pub struct AuthMiddleware {
    jwt_secret: String,
    optional: bool,
}

impl AuthMiddleware {
    pub fn new(secret: &str) -> Self {
        Self { jwt_secret: secret.to_string(), optional: false }
    }

    pub fn optional(secret: &str) -> Self {
        Self { jwt_secret: secret.to_string(), optional: true }
    }
}

impl Middleware for AuthMiddleware {
    fn handle(&self, req: &mut Request, next: Next) -> Result<Response, errors::HandlerError> {
        let token = req.headers.get("Authorization")
            .and_then(|v| v.strip_prefix("Bearer "));

        let claims = match token {
            Some(t) => Some(authenticate(t, &self.jwt_secret)?),
            None if self.optional => None,
            None => return Err(AuthError::MissingToken.into()),
        };

        req.extensions.insert(claims);
        next.run(req)
    }
}
"#;

const USER_HANDLER_RS: &str = r#"
use types::{Request, Response, User};
use errors::HandlerError;
use auth::authorize;

pub async fn get_user(req: &Request) -> Result<Response, HandlerError> {
    let claims = req.extensions.get::<Claims>()
        .ok_or(HandlerError::Unauthorized)?;
    authorize(claims, "admin")?;

    let user_id = req.params.get("id")
        .ok_or(HandlerError::MissingField("id"))?;

    let user = get_database_connection()?
        .query_user_by_id(user_id)
        .map_err(HandlerError::DatabaseError)?;

    Ok(Response::json(serde_json::to_value(&user)?))
}

pub async fn list_users(req: &Request) -> Result<Response, HandlerError> {
    let page: usize = req.query.get("page")
        .and_then(|p| p.parse().ok())
        .unwrap_or(1);

    let users = get_database_connection()?
        .query_users_page(page, 50)
        .map_err(HandlerError::DatabaseError)?;

    Ok(Response::json(serde_json::json!({
        "users": users,
        "page": page,
    })))
}

pub async fn create_user(req: &Request) -> Result<Response, HandlerError> {
    let claims = req.extensions.get::<Claims>()
        .ok_or(HandlerError::Unauthorized)?;
    authorize(claims, "admin")?;

    let user: User = serde_json::from_value(req.body.clone())?;
    let created = get_database_connection()?
        .insert_user(&user)
        .map_err(HandlerError::DatabaseError)?;

    Ok(Response::json(serde_json::to_value(&created)?))
}
"#;

const CONFIG_HANDLER_RS: &str = r#"
use types::{Request, Response, Config};
use errors::HandlerError;

pub async fn get_config(req: &Request) -> Result<Response, HandlerError> {
    let claims = req.extensions.get::<Claims>()
        .ok_or(HandlerError::Unauthorized)?;
    auth::authorize(claims, "admin")?;

    let config = Config::load("config.toml")
        .map_err(HandlerError::ConfigError)?;

    Ok(Response::json(serde_json::to_value(&config)?))
}

pub async fn update_config(req: &Request) -> Result<Response, HandlerError> {
    let claims = req.extensions.get::<Claims>()
        .ok_or(HandlerError::Unauthorized)?;
    auth::authorize(claims, "admin")?;

    let new_config: Config = serde_json::from_value(req.body.clone())?;
    new_config.save("config.toml")
        .map_err(HandlerError::ConfigError)?;

    Ok(Response::json(serde_json::json!({"status": "updated"})))
}
"#;

const UTILS_RS: &str = r#"
use std::time::{SystemTime, UNIX_EPOCH};

pub fn timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

pub fn generate_id() -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    timestamp().hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

pub fn parse_duration(s: &str) -> Result<std::time::Duration, String> {
    let num: u64 = s.chars().take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .map_err(|_| format!("Invalid duration: {}", s))?;

    let unit = s.chars().skip_while(|c| c.is_ascii_digit()).collect::<String>();
    match unit.as_str() {
        "s" | "" => Ok(std::time::Duration::from_secs(num)),
        "m" => Ok(std::time::Duration::from_secs(num * 60)),
        "h" => Ok(std::time::Duration::from_secs(num * 3600)),
        _ => Err(format!("Unknown duration unit: {}", unit)),
    }
}

pub fn truncate(s: &str, max_len: usize) -> &str {
    if s.len() <= max_len {
        s
    } else {
        &s[..max_len]
    }
}
"#;

const TYPES_RS: &str = r#"
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub port: u16,
    pub host: String,
    pub database_url: String,
    pub jwt_secret: String,
    pub log_level: String,
    pub max_connections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
    pub created_at: String,
    pub password_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub sub: String,
    pub exp: u64,
    pub iat: u64,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: u64,
    pub iat: u64,
    pub roles: Vec<String>,
    pub session_id: String,
}

#[derive(Debug, Clone)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub headers: std::collections::HashMap<String, String>,
    pub query: std::collections::HashMap<String, String>,
    pub params: std::collections::HashMap<String, String>,
    pub body: serde_json::Value,
    pub extensions: std::collections::HashMap<String, Box<dyn std::any::Any>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub status: u16,
    pub body: serde_json::Value,
    pub headers: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct Route {
    pub method: String,
    pub path: String,
    pub handler: String,
}

pub struct Next<'a> {
    pub request: &'a mut Request,
}

pub trait Middleware: Send + Sync {
    fn handle(&self, req: &mut Request, next: Next) -> Result<Response, errors::HandlerError>;
}

impl Response {
    pub fn json(body: serde_json::Value) -> Self {
        Self { status: 200, body, headers: std::collections::HashMap::new() }
    }
}
"#;

const ERRORS_RS: &str = r#"
use std::fmt;

#[derive(Debug)]
pub enum AuthError {
    MissingToken,
    InvalidToken(String),
    TokenExpired,
    Unauthorized,
    InternalError(String),
}

#[derive(Debug)]
pub enum HandlerError {
    Unauthorized,
    MissingField(&'static str),
    InvalidCredentials,
    DatabaseError(String),
    ConfigError(String),
    NotFound(String),
    InternalError(String),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::MissingToken => write!(f, "Authentication token is missing"),
            AuthError::InvalidToken(e) => write!(f, "Invalid token: {}", e),
            AuthError::TokenExpired => write!(f, "Token has expired"),
            AuthError::Unauthorized => write!(f, "Unauthorized access"),
            AuthError::InternalError(e) => write!(f, "Internal auth error: {}", e),
        }
    }
}

impl fmt::Display for HandlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HandlerError::Unauthorized => write!(f, "Unauthorized"),
            HandlerError::MissingField(f) => write!(f, "Missing required field: {}", f),
            HandlerError::InvalidCredentials => write!(f, "Invalid credentials"),
            HandlerError::DatabaseError(e) => write!(f, "Database error: {}", e),
            HandlerError::ConfigError(e) => write!(f, "Config error: {}", e),
            HandlerError::NotFound(e) => write!(f, "Not found: {}", e),
            HandlerError::InternalError(e) => write!(f, "Internal error: {}", e),
        }
    }
}

impl From<AuthError> for HandlerError {
    fn from(e: AuthError) -> Self {
        HandlerError::Unauthorized
    }
}
"#;

const CARGO_TOML: &str = r#"
[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
tokio = { version = "1", features = ["full"] }
"#;

const README_MD: &str = r#"
# Test Project

This is a test project for benchmarking CodeScope.

## Getting Started

```bash
cargo build
cargo test
```

## API Endpoints

- POST /auth/login
- POST /auth/logout
- GET /users
- GET /users/:id
- POST /users
- GET /config
- PUT /config
"#;


fn bench_file_search(c: &mut Criterion) {
    let dir = setup_test_repo();
    let path = dir.path().to_string_lossy().to_string();

    let mut group = c.benchmark_group("file_search");

    group.bench_function("fuzzy_match_short", |b| {
        b.iter(|| {
            let _ = file_search::search_files(
                black_box("main"),
                black_box(&path),
                None,
                None,
                false,
                true,
                false,
                None,
                20,
                false,
            );
        })
    });

    group.bench_function("fuzzy_match_long", |b| {
        b.iter(|| {
            let _ = file_search::search_files(
                black_box("handler"),
                black_box(&path),
                None,
                None,
                false,
                true,
                false,
                None,
                20,
                false,
            );
        })
    });

    group.bench_function("extension_filter", |b| {
        b.iter(|| {
            let _ = file_search::search_files(
                black_box(""),
                black_box(&path),
                None,
                Some(&["rs"]),
                false,
                true,
                false,
                None,
                20,
                false,
            );
        })
    });

    group.bench_function("collect_results", |b| {
        b.iter(|| {
            let _ = file_search::collect_file_results(
                black_box(""),
                black_box(&path),
                None,
                None,
                false,
                true,
                false,
                None,
            );
        })
    });

    group.finish();
}

fn bench_content_search(c: &mut Criterion) {
    let dir = setup_test_repo();
    let path = dir.path().to_string_lossy().to_string();

    let mut group = c.benchmark_group("content_search");

    group.bench_function("fuzzy_content", |b| {
        b.iter(|| {
            let _ = content_search::search_content(
                black_box("authenticate"),
                black_box(&path),
                None,
                content_search::MatchMode::Fuzzy,
                None,
                true,
                false,
                false,
                0,
                None,
                20,
                false,
                false,
            );
        })
    });

    group.bench_function("exact_content", |b| {
        b.iter(|| {
            let _ = content_search::search_content(
                black_box("fn authenticate"),
                black_box(&path),
                None,
                content_search::MatchMode::Exact,
                None,
                true,
                false,
                false,
                0,
                None,
                20,
                false,
                false,
            );
        })
    });

    group.bench_function("regex_content", |b| {
        b.iter(|| {
            let _ = content_search::search_content(
                black_box(r"pub\s+(async\s+)?fn\s+\w+"),
                black_box(&path),
                None,
                content_search::MatchMode::Regex,
                None,
                true,
                false,
                false,
                0,
                None,
                20,
                false,
                false,
            );
        })
    });

    group.bench_function("content_with_context", |b| {
        b.iter(|| {
            let _ = content_search::search_content(
                black_box("authenticate"),
                black_box(&path),
                None,
                content_search::MatchMode::Fuzzy,
                None,
                true,
                false,
                false,
                5,
                None,
                20,
                false,
                false,
            );
        })
    });

    group.bench_function("content_invert", |b| {
        b.iter(|| {
            let _ = content_search::search_content(
                black_box("TODO"),
                black_box(&path),
                None,
                content_search::MatchMode::Exact,
                None,
                true,
                false,
                false,
                0,
                None,
                20,
                false,
                true,
            );
        })
    });

    group.finish();
}

fn bench_stats(c: &mut Criterion) {
    let dir = setup_test_repo();
    let path = dir.path().to_string_lossy().to_string();

    let mut group = c.benchmark_group("stats");

    group.bench_function("project_stats", |b| {
        b.iter(|| {
            let _ = codescope::stats::compute_stats(black_box(&path), None, None);
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_file_search,
    bench_content_search,
    bench_stats,
);
criterion_main!(benches);
