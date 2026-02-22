use crate::graph::{Graph, Node};
use crate::layout::LayoutResult;
use std::fmt::Write;

pub const MIN_RADIUS: f64 = 4.0;
pub const MAX_RADIUS: f64 = 30.0;
pub const DIR_RADIUS: f64 = 8.0;
pub const BG_COLOR: &str = "#1a1a2e";

pub fn render_svg(graph: &Graph, layout: &LayoutResult, width: u32, height: u32) -> String {
    let mut svg = String::new();

    // SVG header
    writeln!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {}" width="{}" height="{}">"#,
        width, height, width, height
    )
    .unwrap();

    // Background
    writeln!(
        svg,
        r#"  <rect width="100%" height="100%" fill="{}"/>"#,
        BG_COLOR
    )
    .unwrap();

    // Styles
    svg.push_str("  <style>\n");
    svg.push_str("    .edge { stroke: #7878a0; stroke-width: 1.5; stroke-opacity: 0.8; }\n");
    svg.push_str("    .node { stroke: #fff; stroke-width: 1; stroke-opacity: 0.3; cursor: pointer; }\n");
    svg.push_str("    .label { font-family: monospace; font-size: 10px; fill: #e0e0e0; pointer-events: none; }\n");
    svg.push_str("    .dir-label { font-family: monospace; font-size: 11px; fill: #fff; font-weight: bold; pointer-events: none; }\n");
    svg.push_str("  </style>\n");

    // Render edges first (below nodes)
    svg.push_str("  <g class=\"edges\">\n");
    for edge in &graph.edges {
        let from_pos = &layout.positions[edge.from];
        let to_pos = &layout.positions[edge.to];
        writeln!(
            svg,
            r#"    <line class="edge" x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" data-from="{}" data-to="{}"/>"#,
            from_pos.x, from_pos.y, to_pos.x, to_pos.y, edge.from, edge.to
        )
        .unwrap();
    }
    svg.push_str("  </g>\n");

    // Calculate max size for scaling
    let max_size = compute_max_size(graph);

    // Render nodes
    svg.push_str("  <g class=\"nodes\">\n");
    for node in &graph.nodes {
        let pos = &layout.positions[node.id];
        let radius = calculate_radius(node, max_size);
        let color = node.file_type.color();

        writeln!(
            svg,
            r#"    <circle class="node" cx="{:.1}" cy="{:.1}" r="{:.1}" fill="{}" data-id="{}" data-label="{}" data-path="{}" data-type="{:?}" data-size="{}" data-is-dir="{}"/>"#,
            pos.x,
            pos.y,
            radius,
            color,
            node.id,
            escape_xml(&node.label),
            escape_xml(&node.full_path),
            node.file_type,
            node.size,
            node.is_dir
        )
        .unwrap();
    }
    svg.push_str("  </g>\n");

    // Render labels (only for directories and larger files)
    svg.push_str("  <g class=\"labels\">\n");
    for node in &graph.nodes {
        let pos = &layout.positions[node.id];
        let radius = calculate_radius(node, max_size);

        // Only show labels for directories or files above a threshold
        if node.is_dir || node.size > max_size as u64 / 10 {
            let label_class = if node.is_dir { "dir-label" } else { "label" };
            let truncated_label = truncate_label(&node.label, 20);

            writeln!(
                svg,
                r#"    <text class="{}" x="{:.1}" y="{:.1}" text-anchor="middle" data-label-for="{}">{}</text>"#,
                label_class,
                pos.x,
                pos.y + radius + 12.0,
                node.id,
                escape_xml(&truncated_label)
            )
            .unwrap();
        }
    }
    svg.push_str("  </g>\n");

    // Legend
    render_legend(&mut svg, width);

    svg.push_str("</svg>\n");
    svg
}

pub fn compute_max_size(graph: &Graph) -> f64 {
    graph
        .nodes
        .iter()
        .filter(|n| !n.is_dir)
        .map(|n| n.size)
        .max()
        .unwrap_or(1)
        .max(1) as f64
}

pub fn calculate_radius(node: &Node, max_size: f64) -> f64 {
    if node.is_dir {
        DIR_RADIUS
    } else if node.size == 0 {
        MIN_RADIUS
    } else {
        let normalized = (node.size as f64 / max_size).sqrt();
        MIN_RADIUS + normalized * (MAX_RADIUS - MIN_RADIUS)
    }
}

pub fn truncate_label(label: &str, max_len: usize) -> String {
    if label.chars().count() <= max_len {
        label.to_string()
    } else {
        let truncated: String = label.chars().take(max_len - 3).collect();
        format!("{}...", truncated)
    }
}

pub fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[allow(dead_code)]
pub fn format_size(size: u64) -> String {
    if size == 0 {
        return "0 B".to_string();
    }
    let units = ["B", "KB", "MB", "GB"];
    let mut s = size as f64;
    let mut unit_idx = 0;
    while s >= 1024.0 && unit_idx < units.len() - 1 {
        s /= 1024.0;
        unit_idx += 1;
    }
    if unit_idx == 0 {
        format!("{} {}", size, units[unit_idx])
    } else {
        format!("{:.1} {}", s, units[unit_idx])
    }
}

fn render_legend(svg: &mut String, width: u32) {
    let legend_items = [
        ("Directory", "#4CAF50"),
        ("Rust", "#FF7043"),
        ("JS", "#FFEB3B"),
        ("TS", "#2196F3"),
        ("Python", "#3F51B5"),
        ("Go", "#00BCD4"),
        ("Markdown", "#9E9E9E"),
        ("Config", "#9C27B0"),
        ("Other", "#BDBDBD"),
    ];

    let start_x = width as f64 - 100.0;
    let start_y = 20.0;

    svg.push_str("  <g class=\"legend\">\n");
    writeln!(
        svg,
        r#"    <rect x="{}" y="{}" width="90" height="{}" fill="{}" fill-opacity="0.8" rx="4"/>"#,
        start_x - 5.0,
        start_y - 5.0,
        legend_items.len() as f64 * 18.0 + 10.0,
        BG_COLOR
    )
    .unwrap();

    for (i, (label, color)) in legend_items.iter().enumerate() {
        let y = start_y + i as f64 * 18.0;
        writeln!(
            svg,
            r#"    <circle cx="{}" cy="{}" r="5" fill="{}"/>"#,
            start_x + 5.0,
            y + 5.0,
            color
        )
        .unwrap();
        writeln!(
            svg,
            r#"    <text x="{}" y="{}" class="label">{}</text>"#,
            start_x + 18.0,
            y + 9.0,
            label
        )
        .unwrap();
    }
    svg.push_str("  </g>\n");
}
