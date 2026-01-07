use axum::{
    Router,
    routing::get,
    extract::Path,
    response::{IntoResponse, Response},
    http::StatusCode,
    body::Body,
};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

static SCHEME_INDEX: Lazy<SchemeIndex> = Lazy::new(|| {
    SchemeIndex::load().expect("Failed to load scheme index")
});

#[derive(Debug)]
struct SchemeInfo {
    name: String,
    path: String,
    system: String,
}

struct SchemeIndex {
    schemes: HashMap<String, SchemeInfo>,
    names: Vec<String>,
}

impl SchemeIndex {
    fn load() -> std::io::Result<Self> {
        let mut schemes = HashMap::new();
        let base16_dir = std::path::Path::new("data/schemes/base16");
        let base24_dir = std::path::Path::new("data/schemes/base24");

        for (dir, system) in [(base16_dir, "base16"), (base24_dir, "base24")] {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            let name = stem.to_lowercase();
                            schemes.insert(name.clone(), SchemeInfo {
                                name: name.clone(),
                                path: path.to_string_lossy().to_string(),
                                system: system.to_string(),
                            });
                        }
                    }
                }
            }
        }

        let names: Vec<String> = schemes.keys().cloned().collect();
        tracing::info!("Loaded {} schemes into index", schemes.len());

        Ok(SchemeIndex { schemes, names })
    }

    fn find_exact(&self, name: &str) -> Option<&SchemeInfo> {
        self.schemes.get(&name.to_lowercase())
    }

    fn find_fuzzy(&self, query: &str, threshold: f64) -> Option<&SchemeInfo> {
        let query_lower = query.to_lowercase();
        let mut best_match: Option<(&String, f64)> = None;

        for name in &self.names {
            let similarity = strsim::jaro_winkler(&query_lower, name);
            if similarity >= threshold {
                if let Some((_, best_sim)) = best_match {
                    if similarity > best_sim {
                        best_match = Some((name, similarity));
                    }
                } else {
                    best_match = Some((name, similarity));
                }
            }
        }

        best_match.and_then(|(name, _)| self.schemes.get(name))
    }
}

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

async fn handle_scheme(Path(SchemePath { scheme }): Path<SchemePath>) -> Response {
    let scheme = sanitize_name(&scheme);

    let scheme_info = SCHEME_INDEX.find_exact(&scheme)
        .or_else(|| SCHEME_INDEX.find_fuzzy(&scheme, 0.8));

    match scheme_info {
        Some(info) => {
            match File::open(&info.path).await {
                Ok(file) => {
                    let stream = ReaderStream::new(file);
                    let body = Body::from_stream(stream);
                    Response::builder()
                        .header("content-type", "application/yaml")
                        .header("x-scheme-name", &info.name)
                        .header("x-scheme-system", &info.system)
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

    Lazy::force(&SCHEME_INDEX);

    let app = Router::new()
        .route("/", get(|| async { "base16.sh server" }))
        .route("/{scheme}/{template}", get(handle_scheme_template))
        .route("/{scheme}", get(handle_scheme));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
