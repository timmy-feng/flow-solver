use std::collections::HashSet;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

pub mod search;
pub mod dsu;
pub mod gen;

use crate::search::{SearchFlow, SearchConfig as InternalSearchConfig};

#[derive(Debug, Clone)]
pub struct SolverConfig {
    pub log_period: Option<usize>,
    pub rotation: usize,
    pub allow_zigzag: bool,
    pub use_table: bool,
    pub use_vcut: bool,
    pub use_diagonals: bool,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            log_period: Some(1_000_000),
            rotation: 0,
            allow_zigzag: false,
            use_table: false,
            use_vcut: false,
            use_diagonals: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionEdges {
    pub h: usize,
    pub w: usize,
    pub down: Vec<bool>,
    pub right: Vec<bool>,
}

fn ccw(board: &Vec<Vec<usize>>) -> Vec<Vec<usize>> {
    (0..board[0].len())
        .map(|row| {
            (0..board.len())
                .map(|col| board[col][board[0].len() - 1 - row])
                .collect()
        })
        .collect()
}

fn dfs_solve(
    state: &mut SearchFlow,
    num_nodes: &mut usize,
    visited: &mut HashSet<Vec<usize>>,
    cfg: &SolverConfig,
) -> bool {
    *num_nodes += 1;

    if cfg.use_table {
        if visited.contains(&state.get_state()) {
            return false;
        }
    }

    if let Some(period) = cfg.log_period {
        if *num_nodes % period == 0 {
            println!("Searched {} nodes", *num_nodes);
            println!("{}\n", state.dump());
        }
    }

    if state.solved() {
        return true;
    }

    if !state.feasible() {
        return false;
    }

    for down in [false, true] {
        for right in [false, true] {
            if state.extend(down, right) {
                if dfs_solve(state, num_nodes, visited, cfg) {
                    return true;
                }
                state.undo();
            }
        }
    }

    if cfg.use_table {
        visited.insert(state.get_state());
    }

    false
}

#[derive(Debug, Clone)]
pub struct SolveResult {
    pub solved: bool,
    pub edges: Option<SolutionEdges>,
    pub nodes: usize,
    pub elapsed: Duration,
    pub colors: Option<Vec<usize>>,
}

pub fn solve_board(mut board: Vec<Vec<usize>>, cfg: &SolverConfig) -> SolveResult {
    for _ in 0..cfg.rotation {
        board = ccw(&board);
    }

    let internal_cfg = InternalSearchConfig {
        allow_zigzag: cfg.allow_zigzag,
        use_vcut: cfg.use_vcut,
        use_diagonals: cfg.use_diagonals,
    };

    let mut solution = SearchFlow::from_with_config(&board, internal_cfg);

    let start_time = Instant::now();
    let mut visited: HashSet<Vec<usize>> = HashSet::new();
    let mut num_nodes: usize = 0;

    let solved = dfs_solve(&mut solution, &mut num_nodes, &mut visited, cfg);

    let (edges, colors) = if solved {
        let (h, w, down, right) = solution.edges();
        let colors = solution.colors();
        (Some(SolutionEdges { h, w, down, right }), Some(colors))
    } else {
        (None, None)
    };

    SolveResult {
        solved,
        edges,
        nodes: num_nodes,
        elapsed: start_time.elapsed(),
        colors,
    }
}
