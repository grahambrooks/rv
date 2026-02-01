use crate::graph::Graph;

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

pub struct LayoutResult {
    pub positions: Vec<Position>,
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
}

const REPULSION: f64 = 5000.0;
const ATTRACTION: f64 = 0.01;
const DAMPING: f64 = 0.85;
const MIN_DISTANCE: f64 = 1.0;
const ITERATIONS: usize = 500;

pub fn compute_layout(graph: &Graph, width: f64, height: f64) -> LayoutResult {
    let n = graph.nodes.len();
    if n == 0 {
        return LayoutResult {
            positions: vec![],
            min_x: 0.0,
            max_x: width,
            min_y: 0.0,
            max_y: height,
        };
    }

    // Initialize positions in a circle
    let mut positions: Vec<Position> = (0..n)
        .map(|i| {
            let angle = 2.0 * std::f64::consts::PI * (i as f64) / (n as f64);
            let radius = (width.min(height) / 3.0).min(200.0);
            Position {
                x: width / 2.0 + radius * angle.cos(),
                y: height / 2.0 + radius * angle.sin(),
            }
        })
        .collect();

    let mut velocities: Vec<Position> = vec![Position { x: 0.0, y: 0.0 }; n];

    // Force-directed iterations
    for _ in 0..ITERATIONS {
        let mut forces: Vec<Position> = vec![Position { x: 0.0, y: 0.0 }; n];

        // Repulsion between all pairs
        for i in 0..n {
            for j in (i + 1)..n {
                let dx = positions[j].x - positions[i].x;
                let dy = positions[j].y - positions[i].y;
                let dist_sq = (dx * dx + dy * dy).max(MIN_DISTANCE * MIN_DISTANCE);
                let dist = dist_sq.sqrt();

                let force = REPULSION / dist_sq;
                let fx = force * dx / dist;
                let fy = force * dy / dist;

                forces[i].x -= fx;
                forces[i].y -= fy;
                forces[j].x += fx;
                forces[j].y += fy;
            }
        }

        // Attraction along edges
        for edge in &graph.edges {
            let i = edge.from;
            let j = edge.to;

            let dx = positions[j].x - positions[i].x;
            let dy = positions[j].y - positions[i].y;
            let dist = (dx * dx + dy * dy).sqrt().max(MIN_DISTANCE);

            let force = ATTRACTION * dist;
            let fx = force * dx / dist;
            let fy = force * dy / dist;

            forces[i].x += fx;
            forces[i].y += fy;
            forces[j].x -= fx;
            forces[j].y -= fy;
        }

        // Center gravity (pull towards center)
        let center_x = width / 2.0;
        let center_y = height / 2.0;
        for i in 0..n {
            let dx = center_x - positions[i].x;
            let dy = center_y - positions[i].y;
            forces[i].x += dx * 0.001;
            forces[i].y += dy * 0.001;
        }

        // Update velocities and positions
        for i in 0..n {
            velocities[i].x = (velocities[i].x + forces[i].x) * DAMPING;
            velocities[i].y = (velocities[i].y + forces[i].y) * DAMPING;

            positions[i].x += velocities[i].x;
            positions[i].y += velocities[i].y;
        }
    }

    // Calculate bounds
    let (min_x, max_x, min_y, max_y) = positions.iter().fold(
        (f64::MAX, f64::MIN, f64::MAX, f64::MIN),
        |(min_x, max_x, min_y, max_y), p| {
            (
                min_x.min(p.x),
                max_x.max(p.x),
                min_y.min(p.y),
                max_y.max(p.y),
            )
        },
    );

    LayoutResult {
        positions,
        min_x,
        max_x,
        min_y,
        max_y,
    }
}

pub fn normalize_positions(layout: &mut LayoutResult, width: f64, height: f64, padding: f64) {
    let range_x = (layout.max_x - layout.min_x).max(1.0);
    let range_y = (layout.max_y - layout.min_y).max(1.0);

    let available_width = width - 2.0 * padding;
    let available_height = height - 2.0 * padding;

    let scale = (available_width / range_x).min(available_height / range_y);

    for pos in &mut layout.positions {
        pos.x = padding + (pos.x - layout.min_x) * scale;
        pos.y = padding + (pos.y - layout.min_y) * scale;
    }

    layout.min_x = padding;
    layout.max_x = padding + range_x * scale;
    layout.min_y = padding;
    layout.max_y = padding + range_y * scale;
}
