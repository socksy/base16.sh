use axum::{
    Router,
    routing::get,
    extract::Path,
};
use serde::Deserialize;
use std::net::SocketAddr;

fn sanitize_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(255)
        .collect()
}

#[derive(Deserialize)]
struct SchemePath {
    scheme: String,
}

#[derive(Deserialize)]
struct SchemeTemplatePath {
    scheme: String,
    template: String,
}

async fn handle_scheme(Path(SchemePath { scheme }): Path<SchemePath>) -> String {
    let scheme = sanitize_name(&scheme);
    format!("Scheme: {}", scheme)
}

async fn handle_scheme_template(
    Path(SchemeTemplatePath { scheme, template }): Path<SchemeTemplatePath>
) -> String {
    let scheme = sanitize_name(&scheme);
    let template = sanitize_name(&template);
    format!("Scheme: {}, Template: {}", scheme, template)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(|| async { "base16.sh server" }))
        .route("/{scheme}/{template}", get(handle_scheme_template))
        .route("/{scheme}", get(handle_scheme));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
