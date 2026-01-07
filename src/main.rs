use axum::{
    Router,
    routing::get,
    extract::Path,
    response::{IntoResponse, Response},
    http::StatusCode,
    body::Body,
};
use serde::Deserialize;
use std::net::SocketAddr;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

fn sanitize_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(255)
        .collect()
}

async fn find_scheme_file(scheme: &str) -> Option<String> {
    let base16_path = format!("data/schemes/base16/{}.yaml", scheme);
    let base24_path = format!("data/schemes/base24/{}.yaml", scheme);

    if tokio::fs::metadata(&base16_path).await.is_ok() {
        Some(base16_path)
    } else if tokio::fs::metadata(&base24_path).await.is_ok() {
        Some(base24_path)
    } else {
        None
    }
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

async fn handle_scheme(Path(SchemePath { scheme }): Path<SchemePath>) -> Response {
    let scheme = sanitize_name(&scheme);

    match find_scheme_file(&scheme).await {
        Some(path) => {
            match File::open(&path).await {
                Ok(file) => {
                    let stream = ReaderStream::new(file);
                    let body = Body::from_stream(stream);
                    Response::builder()
                        .header("content-type", "application/yaml")
                        .body(body)
                        .unwrap()
                }
                Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file").into_response()
            }
        }
        None => (StatusCode::NOT_FOUND, format!("Scheme '{}' not found", scheme)).into_response()
    }
}

async fn handle_scheme_template(
    Path(SchemeTemplatePath { scheme, template }): Path<SchemeTemplatePath>
) -> Response {
    let scheme = sanitize_name(&scheme);
    let template = sanitize_name(&template);
    (StatusCode::NOT_IMPLEMENTED, format!("Template rendering not yet implemented: {}/{}", scheme, template)).into_response()
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
