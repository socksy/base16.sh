use axum::{
    Router,
    routing::get,
    extract::{Path, Query},
    response::{IntoResponse, Response},
    http::{StatusCode, HeaderMap},
    body::Body,
};
use mustache::MapBuilder;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

static SCHEME_INDEX: Lazy<SchemeIndex> = Lazy::new(|| {
    SchemeIndex::load().expect("Failed to load scheme index")
});

static TEMPLATE_INDEX: Lazy<TemplateIndex> = Lazy::new(|| {
    TemplateIndex::load().expect("Failed to load template index")
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

#[derive(Debug)]
struct TemplateInfo {
    name: String,
    path: String,
    _repo: String,
}

struct TemplateIndex {
    templates: HashMap<String, TemplateInfo>,
}

impl TemplateIndex {
    fn load() -> std::io::Result<Self> {
        let mut templates = HashMap::new();
        let templates_dir = std::path::Path::new("data/templates");

        if let Ok(entries) = std::fs::read_dir(templates_dir) {
            for entry in entries.flatten() {
                let repo_path = entry.path();
                if !repo_path.is_dir() {
                    continue;
                }
                let repo_name = repo_path.file_name().unwrap().to_str().unwrap();
                let config_path = repo_path.join("templates/config.yaml");

                if let Ok(config_str) = std::fs::read_to_string(&config_path) {
                    if let Ok(config) = serde_yaml::from_str::<HashMap<String, serde_yaml::Value>>(&config_str) {
                        for (template_name, _) in config.iter() {
                            let mustache_file = format!("{}.mustache", template_name);
                            let template_path = repo_path.join(format!("templates/{}", mustache_file));

                            if template_path.exists() {
                                let short_name = format!("{}-{}",
                                    repo_name.trim_start_matches("base16-").trim_start_matches("base24-"),
                                    template_name
                                );
                                templates.insert(short_name.clone(), TemplateInfo {
                                    name: short_name.clone(),
                                    path: template_path.to_string_lossy().to_string(),
                                    _repo: repo_name.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }

        tracing::info!("Loaded {} templates into index", templates.len());

        Ok(TemplateIndex { templates })
    }

    fn find(&self, name: &str) -> Option<&TemplateInfo> {
        self.templates.get(&name.to_lowercase())
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct SchemeYaml {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    system: String,
    name: String,
    author: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    variant: String,
    palette: HashMap<String, String>,
}

#[derive(Deserialize)]
struct FormatQuery {
    #[serde(default)]
    format: Option<String>,
}

#[derive(Serialize)]
struct HelpResponse {
    schemes: Vec<String>,
    templates: Vec<String>,
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .take(255)
        .collect()
}

fn slugify(name: &str) -> String {
    name.to_lowercase().replace(' ', "-")
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

async fn handle_scheme(
    Path(SchemePath { scheme }): Path<SchemePath>,
    Query(query): Query<FormatQuery>,
    headers: HeaderMap,
) -> Response {
    let scheme = sanitize_name(&scheme);

    let scheme_info = SCHEME_INDEX.find_exact(&scheme)
        .or_else(|| SCHEME_INDEX.find_fuzzy(&scheme, 0.8));

    let scheme_info = match scheme_info {
        Some(info) => info,
        None => return (StatusCode::NOT_FOUND, format!("Scheme '{}' not found", scheme)).into_response(),
    };

    let wants_json = query.format.as_deref() == Some("json")
        || headers.get("accept")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.contains("application/json"))
            .unwrap_or(false);

    if wants_json {
        let scheme_yaml_str = match std::fs::read_to_string(&scheme_info.path) {
            Ok(s) => s,
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file").into_response(),
        };

        let scheme_data: SchemeYaml = match serde_yaml::from_str(&scheme_yaml_str) {
            Ok(d) => d,
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to parse scheme YAML").into_response(),
        };

        let json = match serde_json::to_string_pretty(&scheme_data) {
            Ok(j) => j,
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to serialize JSON").into_response(),
        };

        Response::builder()
            .header("content-type", "application/json")
            .header("x-scheme-name", &scheme_info.name)
            .header("x-scheme-system", &scheme_info.system)
            .body(Body::from(json))
            .unwrap()
    } else {
        match File::open(&scheme_info.path).await {
            Ok(file) => {
                let stream = ReaderStream::new(file);
                let body = Body::from_stream(stream);
                Response::builder()
                    .header("content-type", "application/yaml")
                    .header("x-scheme-name", &scheme_info.name)
                    .header("x-scheme-system", &scheme_info.system)
                    .body(body)
                    .unwrap()
            }
            Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file").into_response()
        }
    }
}

async fn handle_help(
    Query(query): Query<FormatQuery>,
    headers: HeaderMap,
) -> Response {
    let mut schemes: Vec<String> = SCHEME_INDEX.schemes.keys().cloned().collect();
    schemes.sort();

    let mut templates: Vec<String> = TEMPLATE_INDEX.templates.keys().cloned().collect();
    templates.sort();

    let help = HelpResponse { schemes, templates };

    let wants_json = query.format.as_deref() == Some("json")
        || headers.get("accept")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.contains("application/json"))
            .unwrap_or(false);

    if wants_json {
        let json = serde_json::to_string_pretty(&help).unwrap();
        Response::builder()
            .header("content-type", "application/json")
            .body(Body::from(json))
            .unwrap()
    } else {
        let mut text = String::from("base16.sh - Base16/Base24 Theme Server\n\n");

        text.push_str("Usage:\n");
        text.push_str("  GET /{scheme}              - get scheme YAML\n");
        text.push_str("  GET /{scheme}?format=json  - get scheme JSON\n");
        text.push_str("  GET /{scheme}/{template}   - render template\n");
        text.push_str("  GET /--help                - this help\n\n");

        text.push_str(&format!("Schemes ({})\n", help.schemes.len()));
        for scheme in &help.schemes {
            text.push_str(&format!("  {}\n", scheme));
        }
        text.push_str(&format!("\nTemplates ({})\n", help.templates.len()));
        for template in &help.templates {
            text.push_str(&format!("  {}\n", template));
        }

        Response::builder()
            .header("content-type", "text/plain; charset=utf-8")
            .body(Body::from(text))
            .unwrap()
    }
}

async fn handle_scheme_template(
    Path(SchemeTemplatePath { scheme, template }): Path<SchemeTemplatePath>
) -> Response {
    let scheme = sanitize_name(&scheme);
    let template = sanitize_name(&template);

    let scheme_info = SCHEME_INDEX.find_exact(&scheme)
        .or_else(|| SCHEME_INDEX.find_fuzzy(&scheme, 0.8));

    let scheme_info = match scheme_info {
        Some(info) => info,
        None => return (StatusCode::NOT_FOUND, format!("Scheme '{}' not found", scheme)).into_response(),
    };

    let template_info = match TEMPLATE_INDEX.find(&template) {
        Some(info) => info,
        None => return (StatusCode::NOT_FOUND, format!("Template '{}' not found", template)).into_response(),
    };

    let scheme_yaml_str = match std::fs::read_to_string(&scheme_info.path) {
        Ok(s) => s,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read scheme file").into_response(),
    };

    let scheme_data: SchemeYaml = match serde_yaml::from_str(&scheme_yaml_str) {
        Ok(d) => d,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to parse scheme YAML").into_response(),
    };

    let template_str = match std::fs::read_to_string(&template_info.path) {
        Ok(s) => s,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read template file").into_response(),
    };

    let template_compiled = match mustache::compile_str(&template_str) {
        Ok(t) => t,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to compile template").into_response(),
    };

    let mut data = MapBuilder::new()
        .insert_str("scheme-name", &scheme_data.name)
        .insert_str("scheme-author", &scheme_data.author)
        .insert_str("scheme-slug", slugify(&scheme_data.name))
        .insert_str("scheme-system", &scheme_info.system);

    if !scheme_data.variant.is_empty() {
        data = data.insert_str("scheme-variant", &scheme_data.variant);
    }

    for (key, value) in &scheme_data.palette {
        let hex_key = format!("{}-hex", key);
        let hex_value = value.trim_start_matches('#');
        data = data.insert_str(&hex_key, hex_value);

        if hex_value.len() == 6 {
            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(&hex_value[0..2], 16),
                u8::from_str_radix(&hex_value[2..4], 16),
                u8::from_str_radix(&hex_value[4..6], 16),
            ) {
                data = data
                    .insert_str(&format!("{}-rgb-r", key), r.to_string())
                    .insert_str(&format!("{}-rgb-g", key), g.to_string())
                    .insert_str(&format!("{}-rgb-b", key), b.to_string());
            }
        }
    }

    let rendered = match template_compiled.render_data_to_string(&data.build()) {
        Ok(r) => r,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to render template").into_response(),
    };

    Response::builder()
        .header("content-type", "text/plain; charset=utf-8")
        .header("x-scheme-name", &scheme_info.name)
        .header("x-template-name", &template_info.name)
        .body(Body::from(rendered))
        .unwrap()
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    Lazy::force(&SCHEME_INDEX);
    Lazy::force(&TEMPLATE_INDEX);

    let app = Router::new()
        .route("/", get(|| async { "base16.sh server" }))
        .route("/--help", get(handle_help))
        .route("/{scheme}/{template}", get(handle_scheme_template))
        .route("/{scheme}", get(handle_scheme));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
