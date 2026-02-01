use ignore::WalkBuilder;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

/// Common output/build directories to filter across languages
static OUTPUT_DIRS: LazyLock<HashSet<&'static str>> = LazyLock::new(|| {
    HashSet::from([
        // Rust
        "target",
        // JavaScript/Node
        "node_modules",
        ".npm",
        ".yarn",
        ".pnpm-store",
        "bower_components",
        // Python
        "__pycache__",
        ".venv",
        "venv",
        ".tox",
        ".nox",
        ".mypy_cache",
        ".pytest_cache",
        ".ruff_cache",
        "*.egg-info",
        ".eggs",
        // Go
        "vendor",
        // Java/Kotlin
        "build",
        ".gradle",
        // C#/.NET
        "bin",
        "obj",
        "packages",
        // C/C++
        "cmake-build-debug",
        "cmake-build-release",
        // Ruby
        ".bundle",
        // PHP
        // "vendor" already listed under Go
        // General
        "dist",
        "out",
        ".cache",
        ".parcel-cache",
        // Frontend frameworks
        ".next",
        ".nuxt",
        ".svelte-kit",
        ".angular",
        ".turbo",
        // Testing/Coverage
        "coverage",
        ".nyc_output",
        "htmlcov",
        // IDE/Editor
        ".idea",
        ".vscode",
        // Version control
        ".git",
        ".hg",
        ".svn",
        // OS files
        ".DS_Store",
        "Thumbs.db",
    ])
});

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub extension: Option<String>,
}

pub struct ScanResult {
    pub entries: Vec<FileEntry>,
    pub parent_map: HashMap<PathBuf, PathBuf>,
}

pub fn scan_directory(
    root: &Path,
    max_depth: Option<usize>,
    filter_output_dirs: bool,
) -> ScanResult {
    let mut entries = Vec::new();
    let mut parent_map = HashMap::new();

    let mut builder = WalkBuilder::new(root);
    builder.hidden(false).git_ignore(true).git_global(true);

    if let Some(depth) = max_depth {
        builder.max_depth(Some(depth + 1));
    }

    // Filter output directories
    if filter_output_dirs {
        builder.filter_entry(|entry| {
            if let Some(name) = entry.file_name().to_str() {
                !OUTPUT_DIRS.contains(name)
            } else {
                true
            }
        });
    }

    for result in builder.build() {
        let entry = match result {
            Ok(e) => e,
            Err(_) => continue,
        };

        let path = entry.path().to_path_buf();
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.to_string_lossy().to_string());

        let is_dir = metadata.is_dir();
        let size = if is_dir { 0 } else { metadata.len() };

        let extension = if is_dir {
            None
        } else {
            path.extension().map(|e| e.to_string_lossy().to_lowercase())
        };

        if let Some(parent) = path.parent() {
            if parent != path {
                parent_map.insert(path.clone(), parent.to_path_buf());
            }
        }

        entries.push(FileEntry {
            path,
            name,
            is_dir,
            size,
            extension,
        });
    }

    ScanResult {
        entries,
        parent_map,
    }
}
