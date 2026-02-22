use crate::graph::Graph;
use crate::layout::LayoutResult;
use crate::svg::{
    calculate_radius, compute_max_size, escape_xml, truncate_label, BG_COLOR,
};
use std::fmt::Write;

/// Render an interactive HTML file with embedded SVG and JavaScript.
pub fn render_interactive_html(
    graph: &Graph,
    layout: &LayoutResult,
    width: u32,
    height: u32,
) -> String {
    let mut html = String::new();

    let max_size = compute_max_size(graph);

    // Build JSON data for nodes
    let mut nodes_json = String::from("[");
    for (i, node) in graph.nodes.iter().enumerate() {
        if i > 0 {
            nodes_json.push(',');
        }
        let pos = &layout.positions[node.id];
        let radius = calculate_radius(node, max_size);
        let color = node.file_type.color();
        write!(
            nodes_json,
            r#"{{"id":{},"label":"{}","fullPath":"{}","isDir":{},"size":{},"fileType":"{}","color":"{}","x":{:.2},"y":{:.2},"r":{:.2}}}"#,
            node.id,
            escape_json(&node.label),
            escape_json(&node.full_path),
            node.is_dir,
            node.size,
            format!("{:?}", node.file_type),
            color,
            pos.x,
            pos.y,
            radius,
        )
        .unwrap();
    }
    nodes_json.push(']');

    // Build JSON data for edges
    let mut edges_json = String::from("[");
    for (i, edge) in graph.edges.iter().enumerate() {
        if i > 0 {
            edges_json.push(',');
        }
        write!(edges_json, r#"{{"from":{},"to":{}}}"#, edge.from, edge.to).unwrap();
    }
    edges_json.push(']');

    // Build parent->children map JSON for collapsing
    let mut children_json = String::from("{");
    {
        use std::collections::HashMap;
        let mut parent_children: HashMap<usize, Vec<usize>> = HashMap::new();
        for edge in &graph.edges {
            parent_children
                .entry(edge.from)
                .or_default()
                .push(edge.to);
        }
        let mut first = true;
        for (parent, children) in &parent_children {
            if !first {
                children_json.push(',');
            }
            first = false;
            write!(children_json, "\"{}\":[", parent).unwrap();
            for (j, child) in children.iter().enumerate() {
                if j > 0 {
                    children_json.push(',');
                }
                write!(children_json, "{}", child).unwrap();
            }
            children_json.push(']');
        }
    }
    children_json.push('}');

    // Build the legend data
    let legend_items = [
        ("Directory", "#4CAF50"),
        ("Rust", "#FF7043"),
        ("JavaScript", "#FFEB3B"),
        ("TypeScript", "#2196F3"),
        ("Python", "#3F51B5"),
        ("Go", "#00BCD4"),
        ("Markdown", "#9E9E9E"),
        ("Config", "#9C27B0"),
        ("Other", "#BDBDBD"),
    ];

    // Generate the static SVG content for the inner graph
    let mut svg_edges = String::new();
    for (i, edge) in graph.edges.iter().enumerate() {
        let from_pos = &layout.positions[edge.from];
        let to_pos = &layout.positions[edge.to];
        writeln!(
            svg_edges,
            r#"<line class="edge" x1="{:.1}" y1="{:.1}" x2="{:.1}" y2="{:.1}" data-from="{}" data-to="{}" data-idx="{}"/>"#,
            from_pos.x, from_pos.y, to_pos.x, to_pos.y, edge.from, edge.to, i
        )
        .unwrap();
    }

    let mut svg_nodes = String::new();
    for node in &graph.nodes {
        let pos = &layout.positions[node.id];
        let radius = calculate_radius(node, max_size);
        let color = node.file_type.color();
        writeln!(
            svg_nodes,
            r#"<circle class="node" cx="{:.1}" cy="{:.1}" r="{:.1}" fill="{}" data-id="{}" data-is-dir="{}"/>"#,
            pos.x, pos.y, radius, color, node.id, node.is_dir
        )
        .unwrap();
    }

    let mut svg_labels = String::new();
    for node in &graph.nodes {
        let pos = &layout.positions[node.id];
        let radius = calculate_radius(node, max_size);
        if node.is_dir || node.size as f64 > max_size / 10.0 {
            let label_class = if node.is_dir { "dir-label" } else { "label" };
            let truncated = truncate_label(&node.label, 20);
            writeln!(
                svg_labels,
                r#"<text class="{}" x="{:.1}" y="{:.1}" text-anchor="middle" data-label-for="{}">{}</text>"#,
                label_class,
                pos.x,
                pos.y + radius + 12.0,
                node.id,
                escape_xml(&truncated)
            )
            .unwrap();
        }
    }

    // Legend SVG
    let mut svg_legend = String::new();
    let legend_x = 10.0_f64;
    let legend_y = 10.0_f64;
    let legend_stroke = "#404060";
    writeln!(
        svg_legend,
        r#"<rect x="{}" y="{}" width="110" height="{}" fill="{}" fill-opacity="0.9" rx="6" stroke="{}" stroke-width="1"/>"#,
        legend_x,
        legend_y,
        legend_items.len() as f64 * 20.0 + 14.0,
        BG_COLOR,
        legend_stroke,
    )
    .unwrap();
    for (i, (label, color)) in legend_items.iter().enumerate() {
        let y = legend_y + 10.0 + i as f64 * 20.0;
        writeln!(
            svg_legend,
            r#"<circle cx="{}" cy="{}" r="5" fill="{}"/>"#,
            legend_x + 14.0,
            y + 4.0,
            color,
        )
        .unwrap();
        writeln!(
            svg_legend,
            r#"<text x="{}" y="{}" class="legend-text">{}</text>"#,
            legend_x + 26.0,
            y + 8.0,
            label,
        )
        .unwrap();
    }

    // --- Assemble the HTML ---
    write!(
        html,
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1.0"/>
<title>rv — Interactive Directory Visualization</title>
<style>
  * {{ margin: 0; padding: 0; box-sizing: border-box; }}
  body {{
    background: {bg};
    color: #e0e0e0;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, monospace;
    overflow: hidden;
    width: 100vw;
    height: 100vh;
  }}
  #container {{
    width: 100%;
    height: 100%;
    position: relative;
  }}
  svg {{
    display: block;
    width: 100%;
    height: 100%;
  }}
  /* Search bar */
  #search-bar {{
    position: fixed;
    top: 12px;
    left: 50%;
    transform: translateX(-50%);
    z-index: 100;
    display: flex;
    align-items: center;
    gap: 6px;
  }}
  #search-input {{
    background: rgba(26, 26, 46, 0.92);
    border: 1px solid #404060;
    border-radius: 6px;
    color: #e0e0e0;
    font-family: monospace;
    font-size: 14px;
    padding: 8px 14px;
    width: 320px;
    outline: none;
    backdrop-filter: blur(8px);
  }}
  #search-input:focus {{
    border-color: #6c63ff;
    box-shadow: 0 0 0 2px rgba(108,99,255,0.25);
  }}
  #search-input::placeholder {{
    color: #666;
  }}
  #search-count {{
    font-size: 12px;
    color: #888;
    min-width: 60px;
  }}
  /* Tooltip */
  #tooltip {{
    position: fixed;
    pointer-events: none;
    background: rgba(20, 20, 40, 0.95);
    border: 1px solid #505070;
    border-radius: 8px;
    padding: 10px 14px;
    font-size: 12px;
    line-height: 1.6;
    color: #e0e0e0;
    max-width: 400px;
    z-index: 200;
    display: none;
    backdrop-filter: blur(8px);
    box-shadow: 0 4px 20px rgba(0,0,0,0.4);
  }}
  #tooltip .tt-name {{
    font-weight: bold;
    font-size: 13px;
    color: #fff;
    margin-bottom: 2px;
  }}
  #tooltip .tt-path {{
    color: #888;
    font-size: 11px;
    word-break: break-all;
  }}
  #tooltip .tt-detail {{
    margin-top: 4px;
    color: #aaa;
  }}
  #tooltip .tt-detail span {{
    color: #ccc;
  }}
  #tooltip .tt-hint {{
    margin-top: 6px;
    font-size: 10px;
    color: #666;
    font-style: italic;
  }}
  /* Controls help */
  #controls-help {{
    position: fixed;
    bottom: 12px;
    left: 12px;
    z-index: 100;
    font-size: 11px;
    color: #555;
    background: rgba(26,26,46,0.85);
    border: 1px solid #303050;
    border-radius: 6px;
    padding: 8px 12px;
    line-height: 1.7;
    backdrop-filter: blur(8px);
  }}
  #controls-help kbd {{
    background: #2a2a4a;
    border: 1px solid #404060;
    border-radius: 3px;
    padding: 1px 5px;
    font-size: 10px;
    color: #999;
  }}
  /* SVG styles */
  .edge {{ stroke: #7878a0; stroke-width: 1.5; stroke-opacity: 0.8; }}
  .edge.highlighted {{ stroke: #6c63ff; stroke-width: 2.5; stroke-opacity: 1; }}
  .edge.dimmed {{ stroke-opacity: 0.08; }}
  .node {{ stroke: #fff; stroke-width: 1; stroke-opacity: 0.3; cursor: pointer; transition: stroke-opacity 0.15s; }}
  .node:hover {{ stroke-opacity: 0.8; stroke-width: 2; }}
  .node.highlighted {{ stroke: #6c63ff; stroke-width: 2.5; stroke-opacity: 1; filter: drop-shadow(0 0 6px rgba(108,99,255,0.5)); }}
  .node.search-match {{ stroke: #ffeb3b; stroke-width: 3; stroke-opacity: 1; filter: drop-shadow(0 0 8px rgba(255,235,59,0.6)); }}
  .node.dimmed {{ opacity: 0.15; }}
  .node.collapsed-dir {{ stroke: #ff5722; stroke-width: 2; stroke-dasharray: 3,2; stroke-opacity: 0.8; }}
  .label, .dir-label {{ font-family: monospace; pointer-events: none; }}
  .label {{ font-size: 10px; fill: #e0e0e0; }}
  .dir-label {{ font-size: 11px; fill: #fff; font-weight: bold; }}
  .label.dimmed, .dir-label.dimmed {{ opacity: 0.1; }}
  .legend-text {{ font-family: monospace; font-size: 11px; fill: #ccc; }}
</style>
</head>
<body>

<div id="search-bar">
  <input id="search-input" type="text" placeholder="Search files… (Ctrl+F)" autocomplete="off"/>
  <span id="search-count"></span>
</div>

<div id="tooltip"></div>

<div id="controls-help">
  <kbd>Scroll</kbd> Zoom &nbsp; <kbd>Drag</kbd> Pan &nbsp; <kbd>Drag node</kbd> Move &nbsp; <kbd>Click dir</kbd> Collapse/Expand &nbsp; <kbd>Ctrl+F</kbd> Search &nbsp; <kbd>Esc</kbd> Reset
</div>

<div id="container">
<svg id="graph-svg" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {w} {h}">
  <rect width="100%" height="100%" fill="{bg}"/>
  <g id="pan-zoom-group">
    <g id="edges-group">
{edges}
    </g>
    <g id="nodes-group">
{nodes}
    </g>
    <g id="labels-group">
{labels}
    </g>
  </g>
  <g id="legend-group">
{legend}
  </g>
</svg>
</div>

<script>
// === Data ===
const NODES = {nodes_json};
const EDGES = {edges_json};
const CHILDREN_MAP = {children_json};

// === State ===
let scale = 1;
let panX = 0, panY = 0;
let isPanning = false;
let panStartX = 0, panStartY = 0;
let panStartPanX = 0, panStartPanY = 0;
let isDraggingNode = false;
let dragNodeId = null;
let dragOffsetX = 0, dragOffsetY = 0;
let hoveredNodeId = null;
let collapsedDirs = new Set();
let hiddenNodes = new Set();
let searchMatches = new Set();

const svg = document.getElementById('graph-svg');
const panZoomGroup = document.getElementById('pan-zoom-group');
const edgesGroup = document.getElementById('edges-group');
const nodesGroup = document.getElementById('nodes-group');
const labelsGroup = document.getElementById('labels-group');
const tooltip = document.getElementById('tooltip');
const searchInput = document.getElementById('search-input');
const searchCount = document.getElementById('search-count');

const nodeEls = nodesGroup.querySelectorAll('.node');
const edgeEls = edgesGroup.querySelectorAll('.edge');
const labelEls = labelsGroup.querySelectorAll('text');

// Index elements by data-id
const nodeElById = {{}};
nodeEls.forEach(el => {{ nodeElById[el.getAttribute('data-id')] = el; }});
const labelElById = {{}};
labelEls.forEach(el => {{ labelElById[el.getAttribute('data-label-for')] = el; }});

// Build edge index: nodeId -> [edge indices]
const edgesByNode = {{}};
edgeEls.forEach((el, idx) => {{
  const from = el.getAttribute('data-from');
  const to = el.getAttribute('data-to');
  if (!edgesByNode[from]) edgesByNode[from] = [];
  if (!edgesByNode[to]) edgesByNode[to] = [];
  edgesByNode[from].push(idx);
  edgesByNode[to].push(idx);
}});

// Build neighbor index
const neighborsByNode = {{}};
EDGES.forEach(e => {{
  if (!neighborsByNode[e.from]) neighborsByNode[e.from] = [];
  if (!neighborsByNode[e.to]) neighborsByNode[e.to] = [];
  neighborsByNode[e.from].push(e.to);
  neighborsByNode[e.to].push(e.from);
}});

// === Transform ===
function applyTransform() {{
  panZoomGroup.setAttribute('transform', `translate(${{panX}},${{panY}}) scale(${{scale}})`);
}}

// === Zoom ===
svg.addEventListener('wheel', (e) => {{
  e.preventDefault();
  const rect = svg.getBoundingClientRect();
  const svgW = parseFloat(svg.getAttribute('viewBox').split(' ')[2]);
  const svgH = parseFloat(svg.getAttribute('viewBox').split(' ')[3]);
  // Mouse position in SVG coordinate space
  const mx = (e.clientX - rect.left) / rect.width * svgW;
  const my = (e.clientY - rect.top) / rect.height * svgH;

  const oldScale = scale;
  const zoomFactor = e.deltaY < 0 ? 1.12 : 1 / 1.12;
  scale = Math.min(Math.max(scale * zoomFactor, 0.1), 30);

  // Adjust pan so zoom is centered on mouse
  panX = mx - (mx - panX) * (scale / oldScale);
  panY = my - (my - panY) * (scale / oldScale);

  applyTransform();
}}, {{ passive: false }});

// === Pan ===
svg.addEventListener('mousedown', (e) => {{
  // Check if we hit a node
  const target = e.target;
  if (target.classList.contains('node')) {{
    const nodeId = parseInt(target.getAttribute('data-id'));
    const nodeData = NODES[nodeId];

    // If it's a directory and not shift-held, toggle collapse
    if (nodeData && nodeData.isDir && !e.shiftKey) {{
      toggleCollapse(nodeId);
      return;
    }}

    // Start dragging
    isDraggingNode = true;
    dragNodeId = nodeId;
    const rect = svg.getBoundingClientRect();
    const svgW = parseFloat(svg.getAttribute('viewBox').split(' ')[2]);
    const svgH = parseFloat(svg.getAttribute('viewBox').split(' ')[3]);
    const mx = (e.clientX - rect.left) / rect.width * svgW;
    const my = (e.clientY - rect.top) / rect.height * svgH;
    const worldX = (mx - panX) / scale;
    const worldY = (my - panY) / scale;
    dragOffsetX = worldX - nodeData.x;
    dragOffsetY = worldY - nodeData.y;
    e.preventDefault();
    return;
  }}

  isPanning = true;
  panStartX = e.clientX;
  panStartY = e.clientY;
  panStartPanX = panX;
  panStartPanY = panY;
  svg.style.cursor = 'grabbing';
}});

window.addEventListener('mousemove', (e) => {{
  if (isPanning) {{
    const rect = svg.getBoundingClientRect();
    const svgW = parseFloat(svg.getAttribute('viewBox').split(' ')[2]);
    const svgH = parseFloat(svg.getAttribute('viewBox').split(' ')[3]);
    const dx = (e.clientX - panStartX) / rect.width * svgW;
    const dy = (e.clientY - panStartY) / rect.height * svgH;
    panX = panStartPanX + dx;
    panY = panStartPanY + dy;
    applyTransform();
    return;
  }}
  if (isDraggingNode && dragNodeId !== null) {{
    const rect = svg.getBoundingClientRect();
    const svgW = parseFloat(svg.getAttribute('viewBox').split(' ')[2]);
    const svgH = parseFloat(svg.getAttribute('viewBox').split(' ')[3]);
    const mx = (e.clientX - rect.left) / rect.width * svgW;
    const my = (e.clientY - rect.top) / rect.height * svgH;
    const worldX = (mx - panX) / scale;
    const worldY = (my - panY) / scale;
    const newX = worldX - dragOffsetX;
    const newY = worldY - dragOffsetY;

    // Update data
    NODES[dragNodeId].x = newX;
    NODES[dragNodeId].y = newY;

    // Update node circle
    const el = nodeElById[dragNodeId];
    if (el) {{
      el.setAttribute('cx', newX.toFixed(1));
      el.setAttribute('cy', newY.toFixed(1));
    }}

    // Update label
    const lbl = labelElById[dragNodeId];
    if (lbl) {{
      lbl.setAttribute('x', newX.toFixed(1));
      lbl.setAttribute('y', (newY + NODES[dragNodeId].r + 12).toFixed(1));
    }}

    // Update connected edges
    (edgesByNode[dragNodeId] || []).forEach(idx => {{
      const edgeEl = edgeEls[idx];
      const fromId = edgeEl.getAttribute('data-from');
      const toId = edgeEl.getAttribute('data-to');
      const fromNode = NODES[fromId];
      const toNode = NODES[toId];
      edgeEl.setAttribute('x1', fromNode.x.toFixed(1));
      edgeEl.setAttribute('y1', fromNode.y.toFixed(1));
      edgeEl.setAttribute('x2', toNode.x.toFixed(1));
      edgeEl.setAttribute('y2', toNode.y.toFixed(1));
    }});

    e.preventDefault();
    return;
  }}

  // Hover detection
  handleHover(e);
}});

window.addEventListener('mouseup', () => {{
  isPanning = false;
  isDraggingNode = false;
  dragNodeId = null;
  svg.style.cursor = '';
}});

// === Hover / Tooltip ===
function handleHover(e) {{
  const target = document.elementFromPoint(e.clientX, e.clientY);
  if (target && target.classList.contains('node')) {{
    const nodeId = parseInt(target.getAttribute('data-id'));
    if (nodeId !== hoveredNodeId) {{
      hoveredNodeId = nodeId;
      highlightNode(nodeId);
      showTooltip(e, nodeId);
    }} else {{
      moveTooltip(e);
    }}
  }} else {{
    if (hoveredNodeId !== null) {{
      hoveredNodeId = null;
      clearHighlight();
      hideTooltip();
    }}
  }}
}}

function highlightNode(nodeId) {{
  const connectedNodes = new Set([nodeId]);
  const connectedEdges = new Set();

  (edgesByNode[nodeId] || []).forEach(idx => {{
    connectedEdges.add(idx);
    const el = edgeEls[idx];
    connectedNodes.add(parseInt(el.getAttribute('data-from')));
    connectedNodes.add(parseInt(el.getAttribute('data-to')));
  }});

  // Dim everything, then highlight connected
  nodeEls.forEach(el => {{
    const id = parseInt(el.getAttribute('data-id'));
    if (connectedNodes.has(id)) {{
      el.classList.add('highlighted');
      el.classList.remove('dimmed');
    }} else {{
      el.classList.add('dimmed');
      el.classList.remove('highlighted');
    }}
  }});

  edgeEls.forEach((el, idx) => {{
    if (connectedEdges.has(idx)) {{
      el.classList.add('highlighted');
      el.classList.remove('dimmed');
    }} else {{
      el.classList.add('dimmed');
      el.classList.remove('highlighted');
    }}
  }});

  labelEls.forEach(el => {{
    const forId = parseInt(el.getAttribute('data-label-for'));
    if (connectedNodes.has(forId)) {{
      el.classList.remove('dimmed');
    }} else {{
      el.classList.add('dimmed');
    }}
  }});
}}

function clearHighlight() {{
  nodeEls.forEach(el => {{
    el.classList.remove('highlighted', 'dimmed');
    // Restore search match state
    const id = parseInt(el.getAttribute('data-id'));
    if (searchMatches.size > 0 && searchMatches.has(id)) {{
      el.classList.add('search-match');
    }}
  }});
  edgeEls.forEach(el => el.classList.remove('highlighted', 'dimmed'));
  labelEls.forEach(el => el.classList.remove('dimmed'));
  // Reapply collapse hiding
  applyVisibility();
}}

function formatSize(bytes) {{
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB'];
  let s = bytes;
  let i = 0;
  while (s >= 1024 && i < units.length - 1) {{ s /= 1024; i++; }}
  return i === 0 ? `${{bytes}} B` : `${{s.toFixed(1)}} ${{units[i]}}`;
}}

function showTooltip(e, nodeId) {{
  const node = NODES[nodeId];
  if (!node) return;
  let html = `<div class="tt-name">${{node.label}}</div>`;
  html += `<div class="tt-path">${{node.fullPath}}</div>`;
  html += `<div class="tt-detail">Type: <span>${{node.fileType}}</span></div>`;
  if (!node.isDir) {{
    html += `<div class="tt-detail">Size: <span>${{formatSize(node.size)}}</span></div>`;
  }}
  if (node.isDir) {{
    const childCount = (CHILDREN_MAP[nodeId] || []).length;
    const isCollapsed = collapsedDirs.has(nodeId);
    html += `<div class="tt-detail">Children: <span>${{childCount}}</span></div>`;
    html += `<div class="tt-hint">Click to ${{isCollapsed ? 'expand' : 'collapse'}}</div>`;
  }} else {{
    html += `<div class="tt-hint">Shift+drag to move</div>`;
  }}
  tooltip.innerHTML = html;
  tooltip.style.display = 'block';
  moveTooltip(e);
}}

function moveTooltip(e) {{
  const pad = 16;
  let x = e.clientX + pad;
  let y = e.clientY + pad;
  // Keep tooltip in viewport
  const rect = tooltip.getBoundingClientRect();
  if (x + rect.width > window.innerWidth) x = e.clientX - rect.width - pad;
  if (y + rect.height > window.innerHeight) y = e.clientY - rect.height - pad;
  tooltip.style.left = x + 'px';
  tooltip.style.top = y + 'px';
}}

function hideTooltip() {{
  tooltip.style.display = 'none';
}}

// === Collapse / Expand Directories ===
function getAllDescendants(nodeId) {{
  const descendants = new Set();
  const queue = [...(CHILDREN_MAP[nodeId] || [])];
  while (queue.length > 0) {{
    const id = queue.pop();
    if (descendants.has(id)) continue;
    descendants.add(id);
    (CHILDREN_MAP[id] || []).forEach(child => queue.push(child));
  }}
  return descendants;
}}

function toggleCollapse(nodeId) {{
  if (collapsedDirs.has(nodeId)) {{
    collapsedDirs.delete(nodeId);
  }} else {{
    collapsedDirs.add(nodeId);
  }}
  recomputeHidden();
  applyVisibility();

  // Update directory visual indicator
  const el = nodeElById[nodeId];
  if (el) {{
    if (collapsedDirs.has(nodeId)) {{
      el.classList.add('collapsed-dir');
    }} else {{
      el.classList.remove('collapsed-dir');
    }}
  }}
}}

function recomputeHidden() {{
  hiddenNodes.clear();
  collapsedDirs.forEach(dirId => {{
    getAllDescendants(dirId).forEach(id => hiddenNodes.add(id));
  }});
}}

function applyVisibility() {{
  nodeEls.forEach(el => {{
    const id = parseInt(el.getAttribute('data-id'));
    el.style.display = hiddenNodes.has(id) ? 'none' : '';
  }});
  edgeEls.forEach(el => {{
    const from = parseInt(el.getAttribute('data-from'));
    const to = parseInt(el.getAttribute('data-to'));
    el.style.display = (hiddenNodes.has(from) || hiddenNodes.has(to)) ? 'none' : '';
  }});
  labelEls.forEach(el => {{
    const forId = parseInt(el.getAttribute('data-label-for'));
    el.style.display = hiddenNodes.has(forId) ? 'none' : '';
  }});
}}

// === Search ===
function handleSearch() {{
  const query = searchInput.value.trim().toLowerCase();
  searchMatches.clear();

  if (query === '') {{
    searchCount.textContent = '';
    nodeEls.forEach(el => {{
      el.classList.remove('search-match', 'dimmed');
    }});
    edgeEls.forEach(el => el.classList.remove('dimmed'));
    labelEls.forEach(el => el.classList.remove('dimmed'));
    return;
  }}

  NODES.forEach(node => {{
    if (node.label.toLowerCase().includes(query) || node.fullPath.toLowerCase().includes(query)) {{
      searchMatches.add(node.id);
    }}
  }});

  searchCount.textContent = `${{searchMatches.size}} match${{searchMatches.size !== 1 ? 'es' : ''}}`;

  // Highlight matches, dim others
  nodeEls.forEach(el => {{
    const id = parseInt(el.getAttribute('data-id'));
    if (searchMatches.has(id)) {{
      el.classList.add('search-match');
      el.classList.remove('dimmed');
    }} else {{
      el.classList.add('dimmed');
      el.classList.remove('search-match');
    }}
  }});

  edgeEls.forEach(el => {{
    const from = parseInt(el.getAttribute('data-from'));
    const to = parseInt(el.getAttribute('data-to'));
    if (searchMatches.has(from) && searchMatches.has(to)) {{
      el.classList.remove('dimmed');
    }} else {{
      el.classList.add('dimmed');
    }}
  }});

  labelEls.forEach(el => {{
    const forId = parseInt(el.getAttribute('data-label-for'));
    if (searchMatches.has(forId)) {{
      el.classList.remove('dimmed');
    }} else {{
      el.classList.add('dimmed');
    }}
  }});

  // Auto-pan to first match
  if (searchMatches.size > 0) {{
    const firstId = searchMatches.values().next().value;
    panToNode(firstId);
  }}
}}

function panToNode(nodeId) {{
  const node = NODES[nodeId];
  if (!node) return;
  const svgW = parseFloat(svg.getAttribute('viewBox').split(' ')[2]);
  const svgH = parseFloat(svg.getAttribute('viewBox').split(' ')[3]);
  // Center the node in the viewport
  const targetScale = Math.max(scale, 1.5);
  panX = svgW / 2 - node.x * targetScale;
  panY = svgH / 2 - node.y * targetScale;
  scale = targetScale;
  applyTransform();
}}

searchInput.addEventListener('input', handleSearch);

// === Keyboard shortcuts ===
document.addEventListener('keydown', (e) => {{
  // Ctrl+F or Cmd+F => focus search
  if ((e.ctrlKey || e.metaKey) && e.key === 'f') {{
    e.preventDefault();
    searchInput.focus();
    searchInput.select();
  }}
  // Escape => clear search, reset view
  if (e.key === 'Escape') {{
    searchInput.value = '';
    searchInput.blur();
    handleSearch();
    // Reset pan/zoom
    scale = 1;
    panX = 0;
    panY = 0;
    applyTransform();
    // Clear collapsed
    collapsedDirs.clear();
    recomputeHidden();
    applyVisibility();
    nodeEls.forEach(el => el.classList.remove('collapsed-dir'));
  }}
}});

// === Fit to view on load ===
(function() {{
  if (NODES.length === 0) return;
  let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
  NODES.forEach(n => {{
    if (n.x - n.r < minX) minX = n.x - n.r;
    if (n.x + n.r > maxX) maxX = n.x + n.r;
    if (n.y - n.r < minY) minY = n.y - n.r;
    if (n.y + n.r > maxY) maxY = n.y + n.r;
  }});
  const svgW = parseFloat(svg.getAttribute('viewBox').split(' ')[2]);
  const svgH = parseFloat(svg.getAttribute('viewBox').split(' ')[3]);
  const graphW = maxX - minX || 1;
  const graphH = maxY - minY || 1;
  const padding = 60;
  const sx = (svgW - padding * 2) / graphW;
  const sy = (svgH - padding * 2) / graphH;
  scale = Math.min(sx, sy, 3);
  panX = (svgW - graphW * scale) / 2 - minX * scale;
  panY = (svgH - graphH * scale) / 2 - minY * scale;
  applyTransform();
}})();
</script>
</body>
</html>
"##,
        bg = BG_COLOR,
        w = width,
        h = height,
        edges = svg_edges,
        nodes = svg_nodes,
        labels = svg_labels,
        legend = svg_legend,
        nodes_json = nodes_json,
        edges_json = edges_json,
        children_json = children_json,
    )
    .unwrap();

    html
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
