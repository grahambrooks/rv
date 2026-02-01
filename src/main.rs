mod graph;
mod layout;
mod scanner;
mod svg;

use clap::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "rv-viz")]
#[command(about = "Generate SVG visualizations of directory structures")]
struct Args {
    /// Directory to visualize
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output SVG file
    #[arg(short, long, default_value = "output.svg")]
    output: PathBuf,

    /// SVG width in pixels
    #[arg(short = 'W', long, default_value = "1200")]
    width: u32,

    /// SVG height in pixels
    #[arg(short = 'H', long, default_value = "800")]
    height: u32,

    /// Maximum directory depth to traverse
    #[arg(short = 'd', long)]
    max_depth: Option<usize>,

    /// Include all directories (don't filter build/output dirs like node_modules, target, etc.)
    #[arg(short = 'a', long)]
    all: bool,
}

fn main() {
    let args = Args::parse();

    let path = args.path.canonicalize().unwrap_or_else(|e| {
        eprintln!("Error: Cannot access path '{}': {}", args.path.display(), e);
        std::process::exit(1);
    });

    if !path.is_dir() {
        eprintln!("Error: '{}' is not a directory", path.display());
        std::process::exit(1);
    }

    let filter_output_dirs = !args.all;
    eprintln!("Scanning directory: {}", path.display());
    if filter_output_dirs {
        eprintln!("Filtering build/output directories (use --all to include)");
    }
    let scan_result = scanner::scan_directory(&path, args.max_depth, filter_output_dirs);
    eprintln!("Found {} entries", scan_result.entries.len());

    if scan_result.entries.is_empty() {
        eprintln!("Error: No files found in directory");
        std::process::exit(1);
    }

    eprintln!("Building graph...");
    let graph = graph::build_graph(scan_result);
    eprintln!(
        "Graph: {} nodes, {} edges",
        graph.nodes.len(),
        graph.edges.len()
    );

    eprintln!("Computing layout...");
    let mut layout_result = layout::compute_layout(&graph, args.width as f64, args.height as f64);
    layout::normalize_positions(
        &mut layout_result,
        args.width as f64,
        args.height as f64,
        80.0,
    );

    eprintln!("Rendering SVG...");
    let svg_content = svg::render_svg(&graph, &layout_result, args.width, args.height);

    fs::write(&args.output, &svg_content).unwrap_or_else(|e| {
        eprintln!("Error: Cannot write to '{}': {}", args.output.display(), e);
        std::process::exit(1);
    });

    eprintln!("Output written to: {}", args.output.display());
}
