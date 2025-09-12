use axum::{
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use crate::{Analyzer, parsers::{GenericParser, rust::RustParser, js::JsParser, python::PythonParser, java::JavaParser}, rules::common::CommonRule};

#[derive(Deserialize)]
pub struct AnalyzeRequest {
    pub error_text: String,
    pub language: Option<String>,
    pub use_ai: Option<bool>,
}

#[derive(Serialize)]
pub struct AnalyzeResponse {
    pub summary: String,
    pub root_causes: Vec<String>,
    pub fixes: Vec<String>,
    pub related: Vec<String>,
    pub success: bool,
    pub error: Option<String>,
}

pub struct AppState {
    // Remove the analyzer from shared state since we need to create fresh instances
}

pub fn create_router() -> Router<()> {
    Router::new()
        .route("/api/analyze", post(analyze_error))
        .route("/api/health", get(health_check))
        .route("/", get(serve_index))
        .nest_service("/static", ServeDir::new("src/web"))
        .layer(CorsLayer::permissive())
}

async fn serve_index() -> axum::response::Html<String> {
    match std::fs::read_to_string("src/web/index.html") {
        Ok(content) => axum::response::Html(content),
        Err(_) => axum::response::Html(r#"
<!DOCTYPE html>
<html>
<head>
    <title>SMELS - Error Analysis</title>
</head>
<body>
    <h1>SMELS Web Interface</h1>
    <p>Error: Could not load index.html</p>
</body>
</html>
        "#.to_string()),
    }
}

async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

async fn analyze_error(
    Json(request): Json<AnalyzeRequest>,
) -> Result<Json<AnalyzeResponse>, StatusCode> {
    if request.error_text.trim().is_empty() {
        return Ok(Json(AnalyzeResponse {
            summary: "No error text provided".to_string(),
            root_causes: vec![],
            fixes: vec![],
            related: vec![],
            success: false,
            error: Some("Empty error text".to_string()),
        }));
    }

    // Create a new analyzer instance for this request
    let mut analyzer = Analyzer::new();
    
    // Configure AI if requested
    if let Some(use_ai) = request.use_ai {
        analyzer = analyzer.with_ai(use_ai);
    }

    // Add parsers based on language or use all
    analyzer.add_parser(Box::new(GenericParser));
    if request.language.as_deref() != Some("none") {
        analyzer.add_parser(Box::new(RustParser));
        analyzer.add_parser(Box::new(JsParser));
        analyzer.add_parser(Box::new(PythonParser));
        analyzer.add_parser(Box::new(JavaParser));
    }
    analyzer.add_rule(Box::new(CommonRule));    // Perform analysis
    let result = analyzer.analyze(&request.error_text).await;
    
    Ok(Json(AnalyzeResponse {
        summary: result.summary,
        root_causes: result.root_causes,
        fixes: result.fixes,
        related: result.related,
        success: true,
        error: None,
    }))
}

pub async fn start_web_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_router();

    let addr = format!("0.0.0.0:{}", port);
    println!("🚀 SMELS Web Server starting on http://{}", addr);
    println!("📖 Open your browser to view the interface");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
