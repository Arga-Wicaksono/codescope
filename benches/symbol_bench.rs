use criterion::{black_box, criterion_group, criterion_main, Criterion};
use codescope::symbol;
use codescope::context;
use std::fs;
use tempfile::TempDir;

/// Create a temporary repository with test files for symbol benchmarking.
fn setup_symbol_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    let src_dir = dir.path().join("src");
    fs::create_dir_all(src_dir.join("auth")).unwrap();
    fs::create_dir_all(src_dir.join("handlers")).unwrap();
    fs::create_dir_all(src_dir.join("models")).unwrap();

    let files = vec![
        ("src/lib.rs", "pub mod auth;\npub mod handlers;\npub mod models;\npub mod utils;\n\npub fn version() -> &'static str { env!(\"CARGO_PKG_VERSION\") }\n"),
        ("src/main.rs", "fn main() {\n    println!(\"Hello\");\n}\n\npub struct App {\n    config: Config,\n}\n\nimpl App {\n    pub fn new() -> Self { Self { config: Config::default() } }\n    pub fn run(&self) { println!(\"Running\"); }\n}\n\npub struct Config {\n    pub port: u16,\n}\n\nimpl Default for Config {\n    fn default() -> Self { Self { port: 8080 } }\n}\n\npub enum State { Running, Stopped, Paused }\n\npub trait Handler {\n    fn handle(&self, req: Request) -> Response;\n    fn name(&self) -> &str;\n}\n\nimpl Handler for App {\n    fn handle(&self, req: Request) -> Response { Response::ok() }\n    fn name(&self) -> &str { \"App\" }\n}\n"),
        ("src/auth/mod.rs", "pub mod middleware;\npub mod service;\n\npub fn authenticate(token: &str) -> Result<User, AuthError> {\n    service::validate_token(token)\n}\n\npub fn authorize(user: &User, role: &str) -> bool {\n    user.roles.contains(&role.to_string())\n}\n\npub struct User {\n    pub id: String,\n    pub username: String,\n    pub roles: Vec<String>,\n}\n\npub struct AuthError(String);\n\nimpl std::fmt::Display for AuthError {\n    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result { write!(f, \"{}\", self.0) }\n}\n"),
        ("src/auth/service.rs", "use crate::models::Token;\n\npub fn validate_token(token: &str) -> Result<crate::auth::User, crate::auth::AuthError> {\n    let parsed = Token::parse(token)?;\n    if parsed.is_expired() {\n        return Err(crate::auth::AuthError(\"Token expired\".into()));\n    }\n    Ok(crate::auth::User { id: parsed.sub, username: parsed.username, roles: parsed.roles })\n}\n\npub fn create_token(user: &crate::auth::User) -> String {\n    Token::new(&user.id, &user.username, &user.roles).encode()\n}\n\npub fn refresh_token(old_token: &str) -> Result<String, crate::auth::AuthError> {\n    let user = validate_token(old_token)?;\n    Ok(create_token(&user))\n}\n"),
        ("src/auth/middleware.rs", "use crate::auth::{authenticate, AuthError};\n\npub struct AuthMiddleware {\n    pub required_role: Option<String>,\n}\n\nimpl AuthMiddleware {\n    pub fn new() -> Self { Self { required_role: None } }\n    pub fn with_role(role: &str) -> Self { Self { required_role: Some(role.to_string()) } }\n    \n    pub fn process(&self, token: &str) -> Result<crate::auth::User, AuthError> {\n        let user = authenticate(token)?;\n        if let Some(role) = &self.required_role {\n            if !crate::auth::authorize(&user, role) {\n                return Err(AuthError(\"Forbidden\".into()));\n            }\n        }\n        Ok(user)\n    }\n}\n"),
        ("src/handlers/mod.rs", "pub mod user;\npub mod config;\npub mod health;\n\npub fn build_routes() -> Vec<Route> {\n    vec![\n        Route::new(\"GET\", \"/health\", health::check),\n        Route::new(\"POST\", \"/auth/login\", user::login),\n        Route::new(\"GET\", \"/users\", user::list),\n        Route::new(\"GET\", \"/config\", config::get),\n    ]\n}\n\npub struct Route {\n    pub method: String,\n    pub path: String,\n    pub handler: fn() -> String,\n}\n\nimpl Route {\n    pub fn new(method: &str, path: &str, handler: fn() -> String) -> Self {\n        Self { method: method.to_string(), path: path.to_string(), handler }\n    }\n}\n"),
        ("src/handlers/user.rs", "use crate::auth::{authenticate, authorize};\n\npub fn login() -> String {\n    \"Login handler\".to_string()\n}\n\npub fn list() -> String {\n    \"User list\".to_string()\n}\n\npub fn create() -> String {\n    \"User created\".to_string()\n}\n\npub fn get_user(id: &str) -> String {\n    format!(\"User {}\", id)\n}\n"),
        ("src/handlers/config.rs", "pub fn get() -> String {\n    \"Config\".to_string()\n}\n\npub fn update() -> String {\n    \"Updated\".to_string()\n}\n"),
        ("src/handlers/health.rs", "pub fn check() -> String {\n    \"OK\".to_string()\n}\n"),
        ("src/models/mod.rs", "pub mod token;\npub mod request;\npub mod response;\n\npub struct AppState {\n    pub db: Database,\n    pub config: Config,\n}\n\npub struct Database {\n    pub url: String,\n}\n"),
        ("src/models/token.rs", "pub struct Token {\n    pub sub: String,\n    pub username: String,\n    pub roles: Vec<String>,\n    pub exp: u64,\n    pub iat: u64,\n}\n\nimpl Token {\n    pub fn new(sub: &str, username: &str, roles: &[String]) -> Self {\n        Self { sub: sub.to_string(), username: username.to_string(), roles: roles.to_vec(), exp: 0, iat: 0 }\n    }\n    \n    pub fn parse(raw: &str) -> Result<Self, String> {\n        Ok(Self { sub: raw.to_string(), username: String::new(), roles: vec![], exp: 0, iat: 0 })\n    }\n    \n    pub fn is_expired(&self) -> bool { false }\n    \n    pub fn encode(&self) -> String { format!(\"{}.{}\", self.sub, self.iat) }\n}\n"),
        ("src/models/request.rs", "pub struct Request {\n    pub method: String,\n    pub path: String,\n    pub body: String,\n}\n\nimpl Request {\n    pub fn new(method: &str, path: &str) -> Self { Self { method: method.to_string(), path: path.to_string(), body: String::new() } }\n}\n"),
        ("src/models/response.rs", "pub struct Response { pub status: u16, pub body: String }\n\nimpl Response {\n    pub fn ok() -> Self { Self { status: 200, body: \"OK\".to_string() } }\n    pub fn json(body: &str) -> Self { Self { status: 200, body: body.to_string() } }\n    pub fn not_found() -> Self { Self { status: 404, body: \"Not Found\".to_string() } }\n}"),
        ("src/utils.rs", "pub fn timestamp() -> u64 { 0 }\npub fn generate_id() -> String { \"id\".to_string() }\npub fn truncate(s: &str, max: usize) -> &str { if s.len() <= max { s } else { &s[..max] } }\n"),
    ];

    for (path, content) in &files {
        fs::write(dir.path().join(path), content).unwrap();
    }

    dir
}

fn bench_symbol_search(c: &mut Criterion) {
    let dir = setup_symbol_repo();
    let path = dir.path().to_string_lossy().to_string();

    let mut group = c.benchmark_group("symbol_search");

    group.bench_function("find_function", |b| {
        b.iter(|| {
            let _ = symbol::run_symbol(
                black_box("authenticate"),
                black_box(&path),
                None, None, None, None, false, None, false,
            );
        })
    });

    group.bench_function("find_struct", |b| {
        b.iter(|| {
            let _ = symbol::run_symbol(
                black_box("User"),
                black_box(&path),
                None, None, None, None, false, None, false,
            );
        })
    });

    group.bench_function("find_impl", |b| {
        b.iter(|| {
            let _ = symbol::run_symbol(
                black_box("Handler"),
                black_box(&path),
                None, None, None, Some("trait"), false, None, false,
            );
        })
    });

    group.bench_function("list_all_symbols", |b| {
        b.iter(|| {
            let _ = symbol::run_symbols(
                black_box(&path),
                None, None, None, None, false, None, Some(200), false,
            );
        })
    });

    group.bench_function("find_refs", |b| {
        b.iter(|| {
            let _ = symbol::run_refs(
                black_box("authenticate"),
                black_box(&path),
                None, None, None, false, None, false,
            );
        })
    });

    group.bench_function("find_callers", |b| {
        b.iter(|| {
            let _ = symbol::run_callers(
                black_box("validate_token"),
                black_box(&path),
                None, None, None, false, None, false,
            );
        })
    });

    group.finish();
}

fn bench_context_engine(c: &mut Criterion) {
    let dir = setup_symbol_repo();
    let path = dir.path().to_string_lossy().to_string();

    let mut group = c.benchmark_group("context_engine");

    group.bench_function("extract_context", |b| {
        b.iter(|| {
            let _ = context::run_context(
                black_box("auth"),
                black_box(&path),
                None, None, None, false, None, Some(20), false,
            );
        })
    });

    group.bench_function("pack_context", |b| {
        b.iter(|| {
            let _ = context::run_pack(
                black_box("authentication flow"),
                black_box(&path),
                None, None, None, false, None, Some(4000), false,
            );
        })
    });

    group.bench_function("trace_symbol", |b| {
        b.iter(|| {
            let _ = context::run_trace(
                black_box("authenticate"),
                black_box(&path),
                None, None, None, false, None, Some(5), false,
            );
        })
    });

    group.finish();
}

fn bench_graph(c: &mut Criterion) {
    let dir = setup_symbol_repo();
    let path = dir.path().to_string_lossy().to_string();

    let mut group = c.benchmark_group("graph");

    group.bench_function("module_graph", |b| {
        b.iter(|| {
            let _ = codescope::graph::run_graph(
                black_box(&path), None, "tree", false, "modules",
            );
        })
    });

    group.bench_function("call_graph", |b| {
        b.iter(|| {
            let _ = codescope::graph::run_graph(
                black_box(&path), None, "flat", false, "calls",
            );
        })
    });

    group.bench_function("impact_analysis", |b| {
        b.iter(|| {
            let _ = codescope::graph::run_impact(
                black_box(&path), black_box("auth"), false,
            );
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_symbol_search,
    bench_context_engine,
    bench_graph,
);
criterion_main!(benches);
