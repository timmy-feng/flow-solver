use std::{time::Instant, collections::HashSet};
use std::fs;

use rand::{ thread_rng, rngs::ThreadRng, Rng };

use clap::{Parser, Subcommand, Args};

use flow_solver::gen::GenFlow;
use flow_solver::*;

// modules are provided via the library crate

#[derive(Debug, Clone)]
pub struct ExtendedSolverConfig(SolverConfig, pub Option<String>);

// dfs moved to library

fn gen(state: &mut GenFlow, rng: &mut ThreadRng) -> bool {
    if state.solved() {
        return true;
    }

    for it in 0..16 {
        if state.extend(rng.gen_bool(0.1), rng.gen_bool(0.5), rng.gen_bool(0.5)) {
            if gen(state, rng) {
                return true;
            }
            state.undo();
        }
    }

    return false;
}

// rotation handled in library

fn gen_entry(h: usize, w: usize, allow_zigzag: bool, output_path: &str) {
    let mut flow = GenFlow::new(h, w, allow_zigzag);
    let mut rng = thread_rng();
    while !gen(&mut flow, &mut rng) {}
    let board = flow.get_board();
    let mut output = String::new();
    for row in board.iter() {
        for cell in row.iter() {
            output.push_str(&cell.to_string());
            output.push(' ');
        }
        output.push('\n');
    }
    fs::write(output_path, output)
        .expect("Could not write file");
    println!("{}", flow.dump());
}

fn solve_entry(input_path: &str, cfg: &ExtendedSolverConfig) {
    let input = fs::read_to_string(input_path)
        .expect("Could not read file");

    let mut board: Vec<Vec<usize>> = input.lines().map(|line| {
        line.split_whitespace().map(|cell| {
            cell.parse::<usize>().expect("Misformatted file")
        }).collect()
    }).collect();

    let result = solve_board(board, &cfg.0);
    if result.solved {
        let (h, w, down, right) = {
            let e = result.edges.as_ref().unwrap();
            (e.h, e.w, e.down.clone(), e.right.clone())
        };
        let mut out = String::new();
        out.push_str(&format!("Solved: true\nNodes: {}\nElapsed: {:?}\n", result.nodes, result.elapsed));
        out.push_str("Edges (down/right flattened) not printed here.\n");
        if let Some(path) = &cfg.1 {
            fs::write(path, out).expect("Could not write output file");
        }
        // Print ASCII dump using original mechanics via edges is non-trivial here; skip.
        println!("Solved. Nodes: {}. Time: {:?}.", result.nodes, result.elapsed);
    } else {
        println!("No solution found :(");
        println!("Searched {} nodes", result.nodes);
        println!("Time elapsed: {:?}", result.elapsed);
    }
}

#[derive(Debug, Parser)]
#[command(name = "flow_solver", version, about = "Flow puzzle generator and solver")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Solve a puzzle from a file
    Solve(SolveArgs),
    /// Generate a new puzzle to an output file
    Gen(GenArgs),
}

#[derive(Debug, Args)]
struct SolveArgs {
    /// Input puzzle file path
    input_file: String,
    /// Write solution/stats to this file
    #[arg(long, value_name = "PATH")]
    output: Option<String>,
    /// Log search progress every N nodes (omit for default; use --no-log to disable)
    #[arg(long, value_name = "N")]
    log_period: Option<usize>,
    /// Disable logging entirely
    #[arg(long, default_value_t = false)]
    no_log: bool,
    /// Rotate input counter-clockwise 0..=3 times
    #[arg(long, default_value_t = 0)]
    rotation: usize,
    /// Allow local zigzags (default: off)
    #[arg(long, default_value_t = false)]
    allow_zigzag: bool,
    /// Enable state dedup table (default: off; currently broken)
    #[arg(long, default_value_t = false)]
    use_table: bool,
    /// Enable vertical cut heuristic (default: off)
    #[arg(long, default_value_t = false)]
    use_vcut: bool,
    /// Disable diagonal head-count pruning (default: on)
    #[arg(long, default_value_t = false)]
    no_diagonals: bool,
}

#[derive(Debug, Args)]
struct GenArgs {
    /// Height of the generated board
    height: usize,
    /// Width of the generated board
    width: usize,
    /// Output file path for the generated puzzle
    output_file: String,
    /// Allow local zigzags while generating (default: off)
    #[arg(long, default_value_t = false)]
    allow_zigzag: bool,
}

fn build_solver_config(args: &SolveArgs) -> ExtendedSolverConfig {
    let mut cfg = SolverConfig::default();
    cfg.rotation = args.rotation;
    cfg.allow_zigzag = args.allow_zigzag;
    cfg.use_table = args.use_table;
    cfg.use_vcut = args.use_vcut;
    cfg.use_diagonals = !args.no_diagonals;
    cfg.log_period = if args.no_log {
        None
    } else {
        match args.log_period {
            Some(n) => Some(n),
            None => cfg.log_period, // keep default Some(1_000_000)
        }
    };
    ExtendedSolverConfig(cfg, args.output.clone())
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Solve(args) => {
            let cfg = build_solver_config(&args);
            solve_entry(&args.input_file, &cfg);
        }
        Commands::Gen(args) => {
            gen_entry(args.height, args.width, args.allow_zigzag, &args.output_file);
        }
    }
}
