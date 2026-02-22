use crate::scanner::ScanResult;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Node {
    pub id: usize,
    pub label: String,
    pub full_path: String,
    pub is_dir: bool,
    pub size: u64,
    pub file_type: FileType,
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FileType {
    Directory,
    Rust,
    JavaScript,
    TypeScript,
    Python,
    Go,
    Markdown,
    Config,
    Other,
}

impl FileType {
    pub fn from_extension(ext: Option<&str>, is_dir: bool) -> Self {
        if is_dir {
            return FileType::Directory;
        }

        match ext {
            Some("rs") => FileType::Rust,
            Some("js") | Some("jsx") | Some("mjs") => FileType::JavaScript,
            Some("ts") | Some("tsx") => FileType::TypeScript,
            Some("py") | Some("pyi") => FileType::Python,
            Some("go") => FileType::Go,
            Some("md") | Some("txt") | Some("rst") => FileType::Markdown,
            Some("json") | Some("yaml") | Some("yml") | Some("toml") | Some("ini")
            | Some("cfg") => FileType::Config,
            _ => FileType::Other,
        }
    }

    pub fn color(&self) -> &'static str {
        match self {
            FileType::Directory => "#4CAF50",  // Green
            FileType::Rust => "#FF7043",       // Orange
            FileType::JavaScript => "#FFEB3B", // Yellow
            FileType::TypeScript => "#2196F3", // Blue
            FileType::Python => "#3F51B5",     // Indigo
            FileType::Go => "#00BCD4",         // Cyan
            FileType::Markdown => "#9E9E9E",   // Gray
            FileType::Config => "#9C27B0",     // Purple
            FileType::Other => "#BDBDBD",      // Light gray
        }
    }
}

pub struct Graph {
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

pub fn build_graph(scan_result: ScanResult) -> Graph {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut path_to_id: HashMap<PathBuf, usize> = HashMap::new();

    // Create nodes
    for (id, entry) in scan_result.entries.iter().enumerate() {
        path_to_id.insert(entry.path.clone(), id);

        let file_type = FileType::from_extension(entry.extension.as_deref(), entry.is_dir);

        nodes.push(Node {
            id,
            label: entry.name.clone(),
            full_path: entry.path.to_string_lossy().to_string(),
            is_dir: entry.is_dir,
            size: entry.size,
            file_type,
        });
    }

    // Create edges
    for entry in &scan_result.entries {
        if let Some(parent_path) = scan_result.parent_map.get(&entry.path) {
            if let (Some(&from_id), Some(&to_id)) =
                (path_to_id.get(parent_path), path_to_id.get(&entry.path))
            {
                edges.push(Edge {
                    from: from_id,
                    to: to_id,
                });
            }
        }
    }

    Graph { nodes, edges }
}
