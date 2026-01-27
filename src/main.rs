use axum::{
    Router,
    routing::get,
    extract::{Path, Query},
    response::{IntoResponse, Response, Redirect},
    http::{StatusCode, HeaderMap, HeaderValue},
    body::Body,
};
use tower_http::set_header::SetResponseHeaderLayer;
use mustache::MapBuilder;
use once_cell::sync::Lazy;
use rand::seq::SliceRandom;
use regex::Regex;
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

static INDEX_TEMPLATE: Lazy<mustache::Template> = Lazy::new(|| {
    mustache::compile_path("templates/index.html.mustache")
        .expect("Failed to load index template")
});

static SCHEME_TEMPLATE: Lazy<mustache::Template> = Lazy::new(|| {
    mustache::compile_path("templates/scheme.html.mustache")
        .expect("Failed to load scheme template")
});

#[derive(Debug)]
struct SchemeInfo {
    name: String,
    path: String,
    system: String,
}

struct SchemeIndex {
    schemes: HashMap<String, SchemeInfo>,
    names_sorted: Vec<String>,
    color_sorted: Vec<String>,
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
                    if path.extension().and_then(|s| s.to_str()) == Some("yaml")
                        && let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            let name = sanitize_name(&stem.to_lowercase());
                            if name.is_empty() {
                                continue;
                            }
                            schemes.insert(name.clone(), SchemeInfo {
                                name: name.clone(),
                                path: path.to_string_lossy().to_string(),
                                system: system.to_string(),
                            });
                        }
                }
            }
        }

        let mut names_sorted: Vec<String> = schemes.keys().cloned().collect();
        names_sorted.sort();

        let color_sorted = Self::compute_color_order(&schemes);

        tracing::info!("Loaded {} schemes into index", schemes.len());

        Ok(SchemeIndex { schemes, names_sorted, color_sorted })
    }

    fn compute_color_order(schemes: &HashMap<String, SchemeInfo>) -> Vec<String> {
        let color_keys = [
            "base00", "base01", "base02", "base03", "base04", "base05", "base06", "base07",
            "base08", "base09", "base0A", "base0B", "base0C", "base0D", "base0E", "base0F",
        ];

        let scheme_to_vector = |palette: &HashMap<String, String>| -> Vec<f64> {
            color_keys.iter().flat_map(|key| {
                let hex = palette.get(*key).map(|s| s.as_str()).unwrap_or("#000000").trim_start_matches('#');
                if hex.len() >= 6 {
                    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f64;
                    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f64;
                    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f64;
                    [r, g, b]
                } else {
                    [0.0, 0.0, 0.0]
                }
            }).collect()
        };

        let color_distance = |a: &[f64], b: &[f64]| -> f64 {
            a.iter().zip(b.iter()).map(|(x, y)| (x - y).powi(2)).sum::<f64>().sqrt()
        };

        let is_grey_scheme = |palette: &HashMap<String, String>| -> bool {
            let accent_keys = ["base08", "base09", "base0A", "base0B", "base0C", "base0D", "base0E", "base0F"];
            let mut grey_count = 0;
            for key in accent_keys {
                if let Some(hex) = palette.get(key) {
                    let hex = hex.trim_start_matches('#');
                    if hex.len() >= 6 {
                        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f64 / 255.0;
                        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f64 / 255.0;
                        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f64 / 255.0;
                        let max = r.max(g).max(b);
                        let min = r.min(g).min(b);
                        let saturation = if max == 0.0 { 0.0 } else { (max - min) / max };
                        if saturation < 0.2 {
                            grey_count += 1;
                        }
                    }
                }
            }
            grey_count >= 5
        };

        // Load scheme data and compute color vectors
        let mut schemes_with_data: Vec<(String, HashMap<String, String>)> = schemes
            .iter()
            .filter_map(|(name, info)| {
                let yaml_str = std::fs::read_to_string(&info.path).ok()?;
                let scheme_data: SchemeYaml = serde_yaml::from_str(&yaml_str).ok()?;
                Some((name.clone(), scheme_data.palette))
            })
            .collect();

        // Sort alphabetically first for consistent starting point
        schemes_with_data.sort_by(|(a, _), (b, _)| a.cmp(b));

        let n = schemes_with_data.len();
        if n == 0 {
            return Vec::new();
        }

        let vectors: Vec<Vec<f64>> = schemes_with_data.iter().map(|(_, p)| scheme_to_vector(p)).collect();

        // Nearest-neighbor traversal starting from darkest
        let mut visited = vec![false; n];
        let mut order = Vec::with_capacity(n);
        let start = vectors.iter().enumerate().min_by(|(_, a), (_, b)| {
            (a[0] + a[1] + a[2]).partial_cmp(&(b[0] + b[1] + b[2])).unwrap()
        }).map(|(i, _)| i).unwrap_or(0);
        order.push(start);
        visited[start] = true;

        for _ in 1..n {
            let last = *order.last().unwrap();
            if let Some(next) = (0..n).filter(|&i| !visited[i]).min_by(|&a, &b| {
                color_distance(&vectors[last], &vectors[a]).partial_cmp(&color_distance(&vectors[last], &vectors[b])).unwrap()
            }) {
                order.push(next);
                visited[next] = true;
            }
        }

        // Partition grey schemes to end
        let (non_grey, grey): (Vec<usize>, Vec<usize>) = order.into_iter().partition(|&i| {
            !is_grey_scheme(&schemes_with_data[i].1)
        });
        let order: Vec<usize> = non_grey.into_iter().chain(grey).collect();

        order.iter().map(|&i| schemes_with_data[i].0.clone()).collect()
    }

    fn get_neighbors(&self, name: &str, by_color: bool) -> (Option<&str>, Option<&str>) {
        let list = if by_color { &self.color_sorted } else { &self.names_sorted };
        let Some(i) = list.iter().position(|n| n == name) else {
            return (None, None);
        };
        let prev = i.checked_sub(1).map(|j| list[j].as_str());
        let next = list.get(i + 1).map(|s| s.as_str());
        (prev, next)
    }

    fn find_exact(&self, name: &str) -> Option<&SchemeInfo> {
        self.schemes.get(&name.to_lowercase())
    }

    fn find_fuzzy(&self, query: &str, threshold: f64) -> Option<&SchemeInfo> {
        let query_lower = query.to_lowercase();
        self.names_sorted
            .iter()
            .map(|name| (name, strsim::jaro_winkler(&query_lower, name)))
            .filter(|(_, sim)| *sim >= threshold)
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .and_then(|(name, _)| self.schemes.get(name))
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

                if let Ok(config_str) = std::fs::read_to_string(&config_path)
                    && let Ok(config) = serde_yaml::from_str::<HashMap<String, serde_yaml::Value>>(&config_str) {
                        let short_repo = sanitize_name(
                            repo_name
                                .trim_start_matches("base16-")
                                .trim_start_matches("base24-")
                        );
                        if short_repo.is_empty() {
                            continue;
                        }

                        let template_count = config.len();

                        for (template_name, _) in config.iter() {
                            let mustache_file = format!("{}.mustache", template_name);
                            let template_path = repo_path.join(format!("templates/{}", mustache_file));

                            if template_path.exists() {
                                let key = if template_count == 1 || template_name == "default" {
                                    short_repo.clone()
                                } else {
                                    format!("{}-{}", short_repo, template_name)
                                };

                                templates.insert(key.clone(), TemplateInfo {
                                    name: key,
                                    path: template_path.to_string_lossy().to_string(),
                                    _repo: repo_name.to_string(),
                                });
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

    fn sorted_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.templates.keys().cloned().collect();
        names.sort();
        names
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
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
    #[serde(default)]
    order: Option<String>,
}

#[derive(Deserialize)]
struct IndexQuery {
    #[serde(default)]
    sort: Option<String>,
    #[serde(default)]
    view: Option<String>,
    #[serde(default)]
    format: Option<String>,
    #[serde(default)]
    filter: Option<String>,
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

fn get_base_description(base: &str) -> Option<&'static str> {
    match base {
        "base00" => Some("Default Background"),
        "base01" => Some("Lighter Background (status bars, line numbers)"),
        "base02" => Some("Selection Background"),
        "base03" => Some("Comments, Invisibles, Line Highlighting"),
        "base04" => Some("Dark Foreground (status bars)"),
        "base05" => Some("Default Foreground, Caret, Delimiters, Operators"),
        "base06" => Some("Light Foreground"),
        "base07" => Some("Lightest Foreground"),
        "base08" => Some("Variables, XML Tags, Markup Link Text, Diff Deleted"),
        "base09" => Some("Integers, Booleans, Constants, XML Attributes"),
        "base0A" | "base0a" => Some("Classes, Markup Bold, Search Text Background"),
        "base0B" | "base0b" => Some("Strings, Inherited Class, Markup Code, Diff Inserted"),
        "base0C" | "base0c" => Some("Support, Regular Expressions, Escape Characters"),
        "base0D" | "base0d" => Some("Functions, Methods, Attribute IDs, Headings"),
        "base0E" | "base0e" => Some("Keywords, Storage, Selector, Diff Changed"),
        "base0F" | "base0f" => Some("Deprecated, Embedded Language Tags"),
        "base10" => Some("Darker Background (Base24)"),
        "base11" => Some("Darkest Background (Base24)"),
        "base12" => Some("Bright Red (Base24)"),
        "base13" => Some("Bright Yellow (Base24)"),
        "base14" => Some("Bright Green (Base24)"),
        "base15" => Some("Bright Cyan (Base24)"),
        "base16" => Some("Bright Blue (Base24)"),
        "base17" => Some("Bright Magenta (Base24)"),
        _ => None,
    }
}

fn colorize_yaml_hex_values(yaml: &str, fg_hex: &str) -> String {
    let hex_pattern = Regex::new(r#"["']?(#[0-9A-Fa-f]{6})["']?"#).unwrap();
    let base_pattern = Regex::new(r"^(\s*)(base[0-9A-Fa-f]{2}):").unwrap();

    yaml.lines()
        .map(|line| {
            // Check if this is a baseXX line first
            let base_name = base_pattern.captures(line)
                .and_then(|caps| caps.get(2))
                .map(|m| m.as_str().to_string());

            // Colorize hex values, including base name as data attribute if available
            let colored_line = hex_pattern.replace_all(line, |caps: &regex::Captures| {
                let full_match = caps.get(0).unwrap().as_str();
                let hex_color = caps.get(1).unwrap().as_str();
                if let Some(ref base) = base_name {
                    format!(r#"<span class="hex-color" data-base="{}" style="--fg: #{}; color: {};">{}</span>"#,
                        base, fg_hex, hex_color, full_match)
                } else {
                    format!(r#"<span class="hex-color" style="--fg: #{}; color: {};">{}</span>"#,
                        fg_hex, hex_color, full_match)
                }
            });

            // Wrap with tooltip if it's a baseXX line
            if let Some(base) = &base_name {
                if let Some(desc) = get_base_description(base) {
                    return format!(r#"<span class="palette-row" title="{}">{}</span>"#, desc, colored_line);
                }
            }
            colored_line.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn build_palette_svg(scheme_data: &SchemeYaml, width: u32, height: u32, rect_width: u32, is_base24: bool) -> String {
    if is_base24 {
        // Double height for two rows
        let total_height = height * 2;
        let mut svg = format!(r#"<svg viewBox="0 0 {} {}" preserveAspectRatio="none" shape-rendering="crispEdges" xmlns="http://www.w3.org/2000/svg">"#, width, total_height);
        // First row: base00-base0F
        for i in 0..16 {
            let color = scheme_data.palette.get(&format!("base{:02X}", i))
                .cloned()
                .unwrap_or_else(|| "#000000".to_string());
            svg.push_str(&format!(r#"<rect x="{}" y="0" width="{}" height="{}" fill="{}"/>"#, i * rect_width, rect_width, height, color));
        }
        // Second row: base10-base17 (8 colors, double width)
        let wide_rect = rect_width * 2;
        for i in 0..8 {
            let color = scheme_data.palette.get(&format!("base{:02X}", 0x10 + i))
                .cloned()
                .unwrap_or_else(|| "#000000".to_string());
            svg.push_str(&format!(r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}"/>"#, i * wide_rect, height, wide_rect, height, color));
        }
        svg.push_str("</svg>");
        svg
    } else {
        let mut svg = format!(r#"<svg viewBox="0 0 {} {}" preserveAspectRatio="none" shape-rendering="crispEdges" xmlns="http://www.w3.org/2000/svg">"#, width, height);
        for i in 0..16 {
            let color = scheme_data.palette.get(&format!("base{:02X}", i))
                .cloned()
                .unwrap_or_else(|| "#000000".to_string());
            svg.push_str(&format!(r#"<rect x="{}" y="0" width="{}" height="{}" fill="{}"/>"#, i * rect_width, rect_width, height, color));
        }
        svg.push_str("</svg>");
        svg
    }
}

fn build_palette_grid_svg(scheme_data: &SchemeYaml, is_base24: bool) -> String {
    if is_base24 {
        // Top half: 4x4 grid for base00-base0F, bottom half: 4x2 grid for base10-base17
        let mut svg = String::from(r#"<svg viewBox="0 0 4 6" preserveAspectRatio="none" shape-rendering="crispEdges" xmlns="http://www.w3.org/2000/svg">"#);
        // Base16 colors (4x4)
        for i in 0..16 {
            let color = scheme_data.palette.get(&format!("base{:02X}", i))
                .cloned()
                .unwrap_or_else(|| "#000000".to_string());
            let x = i % 4;
            let y = i / 4;
            svg.push_str(&format!(r#"<rect x="{}" y="{}" width="1" height="1" fill="{}"/>"#, x, y, color));
        }
        // Base24 extra colors (4x2)
        for i in 0..8 {
            let color = scheme_data.palette.get(&format!("base{:02X}", 0x10 + i))
                .cloned()
                .unwrap_or_else(|| "#000000".to_string());
            let x = i % 4;
            let y = 4 + i / 4;
            svg.push_str(&format!(r#"<rect x="{}" y="{}" width="1" height="1" fill="{}"/>"#, x, y, color));
        }
        svg.push_str("</svg>");
        svg
    } else {
        let mut svg = String::from(r#"<svg viewBox="0 0 4 4" preserveAspectRatio="none" shape-rendering="crispEdges" xmlns="http://www.w3.org/2000/svg">"#);
        for i in 0..16 {
            let color = scheme_data.palette.get(&format!("base{:02X}", i))
                .cloned()
                .unwrap_or_else(|| "#000000".to_string());
            let x = i % 4;
            let y = i / 4;
            svg.push_str(&format!(r#"<rect x="{}" y="{}" width="1" height="1" fill="{}"/>"#, x, y, color));
        }
        svg.push_str("</svg>");
        svg
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

async fn handle_scheme(
    Path(SchemePath { scheme }): Path<SchemePath>,
    Query(query): Query<FormatQuery>,
    headers: HeaderMap,
) -> Response {
    let sanitized = sanitize_name(&scheme);

    let scheme_info = if let Some(info) = SCHEME_INDEX.find_exact(&sanitized) {
        if scheme != info.name {
            return Redirect::permanent(&format!("/{}", info.name)).into_response();
        }
        info
    } else if let Some(info) = SCHEME_INDEX.find_fuzzy(&sanitized, 0.8) {
        return Redirect::permanent(&format!("/{}", info.name)).into_response();
    } else {
        return (StatusCode::NOT_FOUND, format!("Scheme '{}' not found", sanitized)).into_response();
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

        let fg = scheme_data.palette.get("base05").cloned().unwrap_or_else(|| "#ffffff".to_string()).trim_start_matches('#').to_string();
        let is_base24 = scheme_info.system == "base24";
        let palette_svg = build_palette_svg(&scheme_data, 320, 40, 20, is_base24);

        // Determine sort order and compute prev/next
        let by_color = query.order.as_deref() == Some("color");
        let order_param = if by_color { "?order=color" } else { "" };
        let (prev, next) = SCHEME_INDEX.get_neighbors(&scheme_info.name, by_color);

        let random_href = if by_color { "/--random?order=color" } else { "/--random" };

        let mut data = MapBuilder::new()
            .insert_str("scheme-name", &scheme_data.name)
            .insert_str("scheme-author", &scheme_data.author)
            .insert_str("scheme-system", &scheme_info.system)
            .insert_str("palette-svg", &palette_svg)
            .insert_str("yaml-colorized", colorize_yaml_hex_values(&scheme_yaml_str, &fg))
            .insert_str("random-href", random_href);

        if let Some(prev_name) = prev {
            data = data.insert_str("prev-scheme", prev_name)
                       .insert_str("prev-href", format!("/{}{}", prev_name, order_param))
                       .insert_bool("has-prev", true);
        }
        if let Some(next_name) = next {
            data = data.insert_str("next-scheme", next_name)
                       .insert_str("next-href", format!("/{}{}", next_name, order_param))
                       .insert_bool("has-next", true);
        }

        for (key, value) in &scheme_data.palette {
            let hex_value = value.trim_start_matches('#');
            data = data.insert_str(format!("{}-hex", key), hex_value);
        }

        let html = match SCHEME_TEMPLATE.render_data_to_string(&data.build()) {
            Ok(h) => h,
            Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to render template").into_response(),
        };

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

async fn handle_index(Query(query): Query<IndexQuery>, headers: HeaderMap) -> Response {
    let accept = headers.get("accept")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let format = match query.format.as_deref() {
        Some("yaml") => "yaml",
        Some("json") => "json",
        Some("html") => "html",
        None if accept.contains("application/yaml") => "yaml",
        None if accept.contains("text/html") => "html",
        _ => "json",
    };

    if format != "html" {
        let response = HelpResponse {
            schemes: SCHEME_INDEX.names_sorted.clone(),
            templates: TEMPLATE_INDEX.sorted_names(),
        };

        return match format {
            "yaml" => {
                let yaml = serde_yaml::to_string(&response).unwrap();
                Response::builder()
                    .header("content-type", "application/yaml")
                    .body(Body::from(yaml))
                    .unwrap()
            }
            _ => {
                let json = serde_json::to_string_pretty(&response).unwrap();
                Response::builder()
                    .header("content-type", "application/json")
                    .body(Body::from(json))
                    .unwrap()
            }
        };
    }

    let sort_by_color = query.sort.as_deref() == Some("color");
    let view_grid = query.view.as_deref() == Some("grid");
    let filter = query.filter.as_deref().unwrap_or("all");
    let filter_base16 = filter == "base16";
    let filter_base24 = filter == "base24";

    let mut schemes_with_data: Vec<(String, SchemeYaml, String, String)> = SCHEME_INDEX
        .schemes
        .iter()
        .filter_map(|(name, info)| {
            let yaml_str = std::fs::read_to_string(&info.path).ok()?;
            let scheme_data: SchemeYaml = serde_yaml::from_str(&yaml_str).ok()?;
            Some((name.clone(), scheme_data, info.path.clone(), info.system.clone()))
        })
        .collect();

    // Always sort alphabetically - color order is handled via CSS
    schemes_with_data.sort_by(|(name_a, _, _, _), (name_b, _, _, _)| name_a.cmp(name_b));

    let template_names = TEMPLATE_INDEX.sorted_names();

    let base16_count = schemes_with_data.iter().filter(|(_, _, _, sys)| sys == "base16").count();
    let base24_count = schemes_with_data.iter().filter(|(_, _, _, sys)| sys == "base24").count();

    let data = MapBuilder::new()
        .insert_str("scheme-count", schemes_with_data.len().to_string())
        .insert_str("base16-count", base16_count.to_string())
        .insert_str("base24-count", base24_count.to_string())
        .insert_str("template-count", template_names.len().to_string())
        .insert_bool("sort-by-name", !sort_by_color)
        .insert_bool("sort-by-color", sort_by_color)
        .insert_bool("view-grid", view_grid)
        .insert_bool("filter-all", !filter_base16 && !filter_base24)
        .insert_bool("filter-base16", filter_base16)
        .insert_bool("filter-base24", filter_base24)
        .insert_vec("schemes", |mut vec| {
            let color_order_map: HashMap<&str, usize> = SCHEME_INDEX.color_sorted
                .iter()
                .enumerate()
                .map(|(i, name)| (name.as_str(), i))
                .collect();

            for (name, scheme_data, _, system) in &schemes_with_data {
                let is_base24 = system == "base24";
                let palette_svg = build_palette_svg(scheme_data, 224, 20, 14, is_base24);
                let palette_grid_svg = build_palette_grid_svg(scheme_data, is_base24);
                let color_pos = color_order_map.get(name.as_str()).copied().unwrap_or(0);
                vec = vec.push_map(|map| {
                    map.insert_str("name", name.as_str())
                       .insert_str("palette-svg", &palette_svg)
                       .insert_str("palette-grid-svg", &palette_grid_svg)
                       .insert_str("color-order", color_pos.to_string())
                       .insert_str("system", system.as_str())
                       .insert_bool("is-base16", system == "base16")
                       .insert_bool("is-base24", system == "base24")
                });
            }
            vec
        })
        .insert_vec("templates", |mut vec| {
            for name in &template_names {
                vec = vec.push_map(|map| {
                    map.insert_str("name", name.as_str())
                });
            }
            vec
        })
        .build();

    let html = match INDEX_TEMPLATE.render_data_to_string(&data) {
        Ok(h) => h,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to render template").into_response(),
    };

    Response::builder()
        .header("content-type", "text/html; charset=utf-8")
        .body(Body::from(html))
        .unwrap()
}

async fn handle_help(
    Query(query): Query<FormatQuery>,
    headers: HeaderMap,
) -> Response {
    let help = HelpResponse {
        schemes: SCHEME_INDEX.names_sorted.clone(),
        templates: TEMPLATE_INDEX.sorted_names(),
    };

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
    let sanitized_scheme = sanitize_name(&scheme);
    let sanitized_template = sanitize_name(&template);

    let scheme_info = if let Some(info) = SCHEME_INDEX.find_exact(&sanitized_scheme) {
        if scheme != info.name {
            return Redirect::permanent(&format!("/{}/{}", info.name, sanitized_template)).into_response();
        }
        info
    } else if let Some(info) = SCHEME_INDEX.find_fuzzy(&sanitized_scheme, 0.8) {
        return Redirect::permanent(&format!("/{}/{}", info.name, sanitized_template)).into_response();
    } else {
        return (StatusCode::NOT_FOUND, format!("Scheme '{}' not found", sanitized_scheme)).into_response();
    };

    let template_info = match TEMPLATE_INDEX.find(&sanitized_template) {
        Some(info) => info,
        None => return (StatusCode::NOT_FOUND, format!("Template '{}' not found", sanitized_template)).into_response(),
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

    let slug = slugify(&scheme_data.name);
    let slug_underscored = slug.replace('-', "_");

    let mut data = MapBuilder::new()
        .insert_str("scheme-name", &scheme_data.name)
        .insert_str("scheme-author", &scheme_data.author)
        .insert_str("scheme-slug", &slug)
        .insert_str("scheme-slug-underscored", &slug_underscored)
        .insert_str("scheme-system", &scheme_info.system);

    if !scheme_data.variant.is_empty() {
        data = data.insert_str("scheme-variant", &scheme_data.variant);
        if scheme_data.variant == "dark" {
            data = data.insert_bool("scheme-is-dark-variant", true);
        } else if scheme_data.variant == "light" {
            data = data.insert_bool("scheme-is-light-variant", true);
        }
    }

    for (key, value) in &scheme_data.palette {
        let hex_value = value.trim_start_matches('#');
        data = data.insert_str(format!("{}-hex", key), hex_value);

        if hex_value.len() == 6 {
            let hex_r = &hex_value[0..2];
            let hex_g = &hex_value[2..4];
            let hex_b = &hex_value[4..6];

            data = data
                .insert_str(format!("{}-hex-r", key), hex_r)
                .insert_str(format!("{}-hex-g", key), hex_g)
                .insert_str(format!("{}-hex-b", key), hex_b)
                .insert_str(format!("{}-hex-bgr", key), format!("{}{}{}", hex_b, hex_g, hex_r));

            if let (Ok(r), Ok(g), Ok(b)) = (
                u8::from_str_radix(hex_r, 16),
                u8::from_str_radix(hex_g, 16),
                u8::from_str_radix(hex_b, 16),
            ) {
                let r16 = (r as u32) * 257;
                let g16 = (g as u32) * 257;
                let b16 = (b as u32) * 257;

                data = data
                    .insert_str(format!("{}-rgb-r", key), r.to_string())
                    .insert_str(format!("{}-rgb-g", key), g.to_string())
                    .insert_str(format!("{}-rgb-b", key), b.to_string())
                    .insert_str(format!("{}-rgb16-r", key), r16.to_string())
                    .insert_str(format!("{}-rgb16-g", key), g16.to_string())
                    .insert_str(format!("{}-rgb16-b", key), b16.to_string())
                    .insert_str(format!("{}-dec-r", key), format!("{:.6}", r as f64 / 255.0))
                    .insert_str(format!("{}-dec-g", key), format!("{:.6}", g as f64 / 255.0))
                    .insert_str(format!("{}-dec-b", key), format!("{:.6}", b as f64 / 255.0));
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

async fn handle_random(Query(query): Query<FormatQuery>) -> Redirect {
    let scheme = SCHEME_INDEX.names_sorted
        .choose(&mut rand::thread_rng())
        .map(|s| s.as_str())
        .unwrap_or("monokai");
    let order_param = if query.order.as_deref() == Some("color") { "?order=color" } else { "" };
    Redirect::to(&format!("/{}{}", scheme, order_param))
}

fn create_app() -> Router {
    Router::new()
        .route("/", get(handle_index))
        .route("/--random", get(handle_random))
        .route("/--help", get(handle_help))
        .route("/{scheme}/{template}", get(handle_scheme_template))
        .route("/{scheme}", get(handle_scheme))
        .layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    Lazy::force(&SCHEME_INDEX);
    Lazy::force(&TEMPLATE_INDEX);
    Lazy::force(&INDEX_TEMPLATE);
    Lazy::force(&SCHEME_TEMPLATE);

    let app = create_app();

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::info!("listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::{Request, StatusCode};
    use tower::util::ServiceExt;

    #[test]
    fn test_scheme_index_loads() {
        let count = SCHEME_INDEX.schemes.len();
        assert!(count > 400, "Expected 400+ schemes, got {}", count);
    }

    #[test]
    fn test_template_index_loads() {
        let count = TEMPLATE_INDEX.templates.len();
        assert!(count > 20, "Expected 20+ templates, got {}", count);
    }

    #[test]
    fn test_scheme_exact_match() {
        let info = SCHEME_INDEX.find_exact("monokai");
        assert!(info.is_some());
        assert_eq!(info.unwrap().name, "monokai");
    }

    #[test]
    fn test_scheme_exact_match_case_insensitive() {
        let info = SCHEME_INDEX.find_exact("MONOKAI");
        assert!(info.is_some());
        assert_eq!(info.unwrap().name, "monokai");
    }

    #[test]
    fn test_scheme_fuzzy_match_typo() {
        let info = SCHEME_INDEX.find_fuzzy("monoki", 0.8);
        assert!(info.is_some(), "Should fuzzy match 'monoki' to 'monokai'");
        assert_eq!(info.unwrap().name, "monokai");
    }

    #[test]
    fn test_scheme_fuzzy_match_partial() {
        let info = SCHEME_INDEX.find_fuzzy("dracula", 0.8);
        assert!(info.is_some());
        assert_eq!(info.unwrap().name, "dracula");
    }

    #[test]
    fn test_scheme_fuzzy_no_match_garbage() {
        let info = SCHEME_INDEX.find_fuzzy("xyzzy123", 0.8);
        assert!(info.is_none(), "Should not match random garbage");
    }

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("hello-world"), "hello-world");
        assert_eq!(sanitize_name("hello_world"), "hello_world");
        assert_eq!(sanitize_name("hello world"), "helloworld");
        assert_eq!(sanitize_name("hello<script>"), "helloscript");
        assert_eq!(sanitize_name("../../../etc/passwd"), "etcpasswd");
    }

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Monokai"), "monokai");
        assert_eq!(slugify("Gruvbox Dark"), "gruvbox-dark");
        assert_eq!(slugify("One Light"), "one-light");
    }

    #[test]
    fn test_hex_to_rgb_conversion() {
        // Test the conversion logic used in template rendering
        let hex = "f92672";
        let hex_r = &hex[0..2];
        let hex_g = &hex[2..4];
        let hex_b = &hex[4..6];

        let r = u8::from_str_radix(hex_r, 16).unwrap();
        let g = u8::from_str_radix(hex_g, 16).unwrap();
        let b = u8::from_str_radix(hex_b, 16).unwrap();

        assert_eq!(r, 249);
        assert_eq!(g, 38);
        assert_eq!(b, 114);

        // Test rgb16 (0-65535 range)
        let r16 = (r as u32) * 257;
        let g16 = (g as u32) * 257;
        let b16 = (b as u32) * 257;

        assert_eq!(r16, 63993);
        assert_eq!(g16, 9766);
        assert_eq!(b16, 29298);

        // Test decimal (0.0-1.0 range)
        let r_dec = r as f64 / 255.0;
        let g_dec = g as f64 / 255.0;
        let b_dec = b as f64 / 255.0;

        assert!((r_dec - 0.976471).abs() < 0.0001);
        assert!((g_dec - 0.149020).abs() < 0.0001);
        assert!((b_dec - 0.447059).abs() < 0.0001);
    }

    #[test]
    fn test_hex_bgr_format() {
        let hex = "f92672";
        let hex_r = &hex[0..2];
        let hex_g = &hex[2..4];
        let hex_b = &hex[4..6];
        let bgr = format!("{}{}{}", hex_b, hex_g, hex_r);
        assert_eq!(bgr, "7226f9");
    }

    #[tokio::test]
    async fn test_scheme_endpoint_yaml() {
        let app = create_app();
        let response = app
            .oneshot(Request::builder().uri("/monokai").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/yaml"
        );
        assert_eq!(
            response.headers().get("x-scheme-name").unwrap(),
            "monokai"
        );
    }

    #[tokio::test]
    async fn test_scheme_endpoint_json() {
        let app = create_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/monokai?format=json")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/json"
        );

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["name"], "Monokai");
    }

    #[tokio::test]
    async fn test_scheme_fuzzy_redirect() {
        let app = create_app();
        let response = app
            .oneshot(Request::builder().uri("/monoki").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
        assert_eq!(
            response.headers().get("location").unwrap(),
            "/monokai"
        );
    }

    #[tokio::test]
    async fn test_scheme_redirect_with_trailing_punctuation() {
        let app = create_app();
        let response = app
            .oneshot(Request::builder().uri("/monokai%20.!").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::PERMANENT_REDIRECT);
        assert_eq!(
            response.headers().get("location").unwrap(),
            "/monokai"
        );
    }

    #[tokio::test]
    async fn test_scheme_not_found() {
        let app = create_app();
        let response = app
            .oneshot(Request::builder().uri("/xyzzy123456").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_template_endpoint() {
        let app = create_app();
        let response = app
            .oneshot(Request::builder().uri("/monokai/vim").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("x-scheme-name").unwrap(),
            "monokai"
        );

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let content = String::from_utf8(body.to_vec()).unwrap();
        assert!(content.contains("monokai"), "Template should contain scheme name");
    }

    #[tokio::test]
    async fn test_template_not_found() {
        let app = create_app();
        let response = app
            .oneshot(Request::builder().uri("/monokai/nonexistent").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_help_endpoint() {
        let app = create_app();
        let response = app
            .oneshot(Request::builder().uri("/--help").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let content = String::from_utf8(body.to_vec()).unwrap();
        assert!(content.contains("monokai"));
        assert!(content.contains("vim"));
    }

    #[tokio::test]
    async fn test_nosniff_header() {
        let app = create_app();
        let response = app
            .oneshot(Request::builder().uri("/monokai").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(
            response.headers().get("x-content-type-options").unwrap(),
            "nosniff"
        );
    }

    #[tokio::test]
    async fn test_index_json_default() {
        let app = create_app();
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/json"
        );

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert!(json["schemes"].is_array());
        assert!(json["templates"].is_array());
    }

    #[tokio::test]
    async fn test_index_html_with_accept() {
        let app = create_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header("accept", "text/html")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(response
            .headers()
            .get("content-type")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("text/html"));
    }

    #[tokio::test]
    async fn test_index_json_with_format_param() {
        let app = create_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/?format=json")
                    .header("accept", "text/html")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/json"
        );
    }

    #[tokio::test]
    async fn test_index_yaml_with_format_param() {
        let app = create_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/?format=yaml")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "application/yaml"
        );

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let content = String::from_utf8(body.to_vec()).unwrap();
        assert!(content.contains("schemes:"));
        assert!(content.contains("templates:"));
    }
}
