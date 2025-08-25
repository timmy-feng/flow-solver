use axum::{routing::{post, get}, Router, Json};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use flow_solver::{solve_board, SolverConfig, SolutionEdges};
use tower_http::services::ServeDir;

#[derive(Debug, Deserialize)]
struct SolveRequest {
    board: Vec<Vec<usize>>,
    #[serde(default)]
    log_period: Option<usize>,
    #[serde(default)]
    rotation: Option<usize>,
    #[serde(default)]
    allow_zigzag: Option<bool>,
    #[serde(default)]
    use_table: Option<bool>,
    #[serde(default)]
    use_vcut: Option<bool>,
    #[serde(default)]
    use_diagonals: Option<bool>,
}

#[derive(Debug, Serialize)]
struct SolveResponse {
    solved: bool,
    nodes: usize,
    elapsed_ms: u128,
    edges: Option<SolutionEdges>,
    colors: Option<Vec<usize>>,
}

async fn solve_handler(Json(req): Json<SolveRequest>) -> Result<Json<SolveResponse>, (StatusCode, String)> {
    let cfg = SolverConfig {
        log_period: req.log_period,
        rotation: req.rotation.unwrap_or(0),
        allow_zigzag: req.allow_zigzag.unwrap_or(false),
        use_table: req.use_table.unwrap_or(false),
        use_vcut: req.use_vcut.unwrap_or(false),
        use_diagonals: req.use_diagonals.unwrap_or(true),
    };
    let res = solve_board(req.board, &cfg);
    Ok(Json(SolveResponse {
        solved: res.solved,
        nodes: res.nodes,
        elapsed_ms: res.elapsed.as_millis(),
        edges: res.edges,
        colors: res.colors,
    }))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/solve", post(solve_handler))
        .route("/health", get(|| async { "ok" }))
        .nest_service("/", ServeDir::new("web"));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Serving on http://{}", addr);
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
