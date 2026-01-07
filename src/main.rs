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

    let scheme_yaml_str = match std::fs::read_to_string(&scheme_info.path) {
        Ok(s) => s,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file").into_response(),
    };

    let wants_json = query.format.as_deref() == Some("json")
        || headers.get("accept")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.contains("application/json"))
            .unwrap_or(false);

    let wants_html = headers.get("accept")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.contains("text/html"))
        .unwrap_or(false);

    if wants_json {
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
    } else if wants_html {
        let scheme_data: SchemeYaml = match serde_yaml::from_str(&scheme_yaml_str) {
            Ok(d) => d,
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to parse scheme YAML").into_response(),
        };

        let bg = scheme_data.palette.get("base00").cloned().unwrap_or_else(|| "#000000".to_string()).trim_start_matches('#').to_string();
        let fg = scheme_data.palette.get("base05").cloned().unwrap_or_else(|| "#ffffff".to_string()).trim_start_matches('#').to_string();
        let comment = scheme_data.palette.get("base03").cloned().unwrap_or_else(|| "#888888".to_string()).trim_start_matches('#').to_string();
        let keyword = scheme_data.palette.get("base0E").cloned().unwrap_or_else(|| "#aa88ff".to_string()).trim_start_matches('#').to_string();
        let string = scheme_data.palette.get("base0B").cloned().unwrap_or_else(|| "#88ff88".to_string()).trim_start_matches('#').to_string();
        let function = scheme_data.palette.get("base0D").cloned().unwrap_or_else(|| "#8888ff".to_string()).trim_start_matches('#').to_string();
        let number = scheme_data.palette.get("base09").cloned().unwrap_or_else(|| "#ffaa88".to_string()).trim_start_matches('#').to_string();

        // Build color palette SVG
        let mut palette_svg = String::from(r#"<svg width="320" height="40" xmlns="http://www.w3.org/2000/svg">"#);
        for i in 0..16 {
            let color = scheme_data.palette.get(&format!("base{:02X}", i))
                .cloned()
                .unwrap_or_else(|| "#000000".to_string());
            palette_svg.push_str(&format!(r#"<rect x="{}" y="0" width="20" height="40" fill="{}"/>"#, i * 20, color));
        }
        palette_svg.push_str("</svg>");

        fn html_escape(s: &str) -> String {
            s.replace('&', "&amp;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
                .replace('"', "&quot;")
        }

        // Generate highlighted Clojure
        let clojure_highlighted = format!(
            r#"<span style="color: #{}">;; Calculate factorial recursively</span>
(<span style="color: #{}"defn</span> factorial [n]
  (<span style="color: #{}"if</span> (<span style="color: #{}"&lt;=</span> n <span style="color: #{}"1</span>)
    <span style="color: #{}"1</span>
    (<span style="color: #{}"*</span> n (factorial (<span style="color: #{}"dec</span> n)))))"#,
            comment, keyword, keyword, function, number, number, function, function
        );

        // Generate highlighted HTML
        let html_highlighted = format!(
            r#"<span style="color: #{}">&lt;!-- Page header --&gt;</span>
&lt;<span style="color: #{}"div</span> <span style="color: #{}"class</span>=<span style="color: #{}">"container"</span>&gt;
  &lt;<span style="color: #{}"h1</span>&gt;Hello World&lt;/<span style="color: #{}"h1</span>&gt;
  &lt;<span style="color: #{}"p</span>&gt;Welcome to our site!&lt;/<span style="color: #{}"p</span>&gt;
&lt;/<span style="color: #{}"div</span>&gt;"#,
            comment, keyword, keyword, string, keyword, keyword, keyword, keyword, keyword
        );

        // Generate highlighted Rust
        let rust_highlighted = format!(
            r#"<span style="color: #{}">//Calculate fibonacci recursively</span>
<span style="color: #{}"fn</span> <span style="color: #{}"fib</span>(n: <span style="color: #{}"u64</span>) -&gt; <span style="color: #{}"u64</span> {{
    <span style="color: #{}"match</span> n {{
        <span style="color: #{}"0</span> | <span style="color: #{}"1</span> =&gt; n,
        _ =&gt; <span style="color: #{}"fib</span>(n - <span style="color: #{}"1</span>) + <span style="color: #{}"fib</span>(n - <span style="color: #{}"2</span>),
    }}
}}"#,
            comment, keyword, function, keyword, keyword, keyword, number, number, function, number, function, number
        );

        let html = format!(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>{} - base16.sh</title>
    <style>
        body {{ font-family: monospace; background: #{}; color: #{}; padding: 20px; }}
        h1 {{ text-align: center; }}
        .container {{ max-width: 1200px; margin: 0 auto; }}
        .palette {{ text-align: center; margin: 20px 0; }}
        .code-examples {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 20px; margin: 40px 0; }}
        .code-block {{ border: 1px solid #{}; padding: 15px; }}
        .lang-label {{ font-size: 12px; color: #{}; margin-bottom: 5px; }}
        pre {{ margin: 0; font-size: 12px; line-height: 1.4; overflow-x: auto; }}
        .yaml-section {{ margin-top: 40px; }}
        .yaml-section h2 {{ color: #{}; }}
        .yaml-container {{ background: #{}; border: 1px solid #{}; padding: 15px; position: relative; }}
        .copy-btn {{ position: absolute; top: 10px; right: 10px; background: #{}; color: #{}; border: 1px solid #{}; padding: 5px 10px; cursor: pointer; }}
        .copy-btn:hover {{ background: #{}; }}
        .back-link {{ text-align: center; margin: 20px 0; }}
        .back-link a {{ color: #{}; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="back-link"><a href="/">← Back to all themes</a></div>
        <h1>{} ({})</h1>
        <p style="text-align: center; color: #{}">{}</p>
        <div class="palette">{}</div>

        <div class="code-examples">
            <div class="code-block">
                <div class="lang-label">Clojure</div>
                <pre>{}</pre>
            </div>
            <div class="code-block">
                <div class="lang-label">HTML</div>
                <pre>{}</pre>
            </div>
            <div class="code-block">
                <div class="lang-label">Rust</div>
                <pre>{}</pre>
            </div>
        </div>

        <div class="yaml-section">
            <h2>Scheme YAML</h2>
            <div class="yaml-container">
                <button class="copy-btn" onclick="navigator.clipboard.writeText(document.getElementById('yaml').textContent)">Copy</button>
                <pre id="yaml">{}</pre>
            </div>
        </div>
    </div>
</body>
</html>
"#,
            scheme_data.name, bg, fg, comment, comment, fg, bg, comment, bg, fg, comment, bg, function,
            scheme_data.name, scheme_info.system, comment, scheme_data.author, palette_svg,
            clojure_highlighted,
            html_highlighted,
            rust_highlighted,
            scheme_yaml_str
        );

        Response::builder()
            .header("content-type", "text/html; charset=utf-8")
            .header("x-scheme-name", &scheme_info.name)
            .header("x-scheme-system", &scheme_info.system)
            .body(Body::from(html))
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

async fn handle_index() -> Response {
    let mut schemes: Vec<(&String, &SchemeInfo)> = SCHEME_INDEX.schemes.iter().collect();
    schemes.sort_by_key(|(name, _)| *name);

    let mut html = String::from(r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <title>base16.sh - All Themes</title>
    <style>
        body { font-family: monospace; background: #1a1a1a; color: #ddd; padding: 20px; max-width: 1200px; margin: 0 auto; }
        h1 { text-align: center; }
        .schemes { display: grid; grid-template-columns: repeat(auto-fill, minmax(250px, 1fr)); gap: 10px; margin-top: 40px; }
        .scheme-card { border: 1px solid #333; padding: 10px; display: block; text-decoration: none; }
        .scheme-card:hover { background: #333; }
        .scheme-name { color: #6a9fb5; margin-bottom: 5px; }
        .scheme-palette { margin-top: 5px; }
    </style>
</head>
<body>
    <h1>base16.sh - All Themes (441)</h1>
    <p style="text-align: center;">Click any theme to see preview with code examples</p>
    <div class="schemes">
"#);

    for (name, info) in &schemes {
        if let Ok(yaml_str) = std::fs::read_to_string(&info.path) {
            if let Ok(scheme_data) = serde_yaml::from_str::<SchemeYaml>(&yaml_str) {
                let mut palette_svg = String::from(r#"<svg width="230" height="20" xmlns="http://www.w3.org/2000/svg">"#);
                for i in 0..16 {
                    let color = scheme_data.palette.get(&format!("base{:02X}", i))
                        .cloned()
                        .unwrap_or_else(|| "#000000".to_string());
                    palette_svg.push_str(&format!(r#"<rect x="{}" y="0" width="14" height="20" fill="{}"/>"#, i * 14, color));
                }
                palette_svg.push_str("</svg>");

                html.push_str(&format!(r#"        <a href="/{}" class="scheme-card">
            <div class="scheme-name">{}</div>
            <div class="scheme-palette">{}</div>
        </a>
"#, name, name, palette_svg));
            }
        }
    }

    html.push_str(r#"    </div>
</body>
</html>
"#);

    Response::builder()
        .header("content-type", "text/html; charset=utf-8")
        .body(Body::from(html))
        .unwrap()
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
        .route("/", get(handle_index))
        .route("/--help", get(handle_help))
        .route("/{scheme}/{template}", get(handle_scheme_template))
        .route("/{scheme}", get(handle_scheme));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
