# rv

A Rust CLI tool that generates force-directed graph SVG visualizations of directory structures.

## Project Structure

![Project Structure](assets/structure.svg)

## Features

- **Force-directed layout** - Organic node positioning using physics simulation
- **File size visualization** - Node sizes reflect file sizes
- **Color-coded file types** - Different colors for Rust, JavaScript, Python, Go, etc.
- **Smart filtering** - Automatically excludes build directories (node_modules, target, etc.)
- **Respects .gitignore** - Skips files ignored by git

## Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/rv.git
cd rv

# Build
cargo build --release

# Install globally (optional)
cargo install --path .
```

## Usage

```bash
rv [OPTIONS] [PATH]

Arguments:
  [PATH]  Directory to visualize [default: .]

Options:
  -o, --output <FILE>   Output SVG file [default: output.svg]
  -W, --width <N>       SVG width in pixels [default: 1200]
  -H, --height <N>      SVG height in pixels [default: 800]
  -d, --max-depth <N>   Maximum directory depth to traverse
  -a, --all             Include all directories (don't filter build dirs)
  -h, --help            Print help
```

### Examples

```bash
# Visualize current directory
rv

# Visualize a specific project
rv ~/projects/my-app -o my-app.svg

# Include node_modules and other build directories
rv --all

# Limit depth for large projects
rv -d 3 -o shallow.svg
```

## Color Legend

| Color | File Type |
|-------|-----------|
| Green | Directories |
| Orange | Rust (.rs) |
| Yellow | JavaScript (.js, .jsx) |
| Blue | TypeScript (.ts, .tsx) |
| Indigo | Python (.py) |
| Cyan | Go (.go) |
| Gray | Markdown, Text |
| Purple | Config (json, yaml, toml) |

## Filtered Directories

By default, rv excludes common build and output directories:

- **Rust:** `target`
- **JavaScript:** `node_modules`, `.npm`, `.yarn`
- **Python:** `__pycache__`, `.venv`, `venv`, `.pytest_cache`
- **Go:** `vendor`
- **Java:** `build`, `.gradle`
- **.NET:** `bin`, `obj`
- **General:** `dist`, `out`, `.cache`, `coverage`
- **IDE:** `.idea`, `.vscode`
- **VCS:** `.git`, `.hg`, `.svn`

Use `--all` to include these directories.

## License

MIT License - see [LICENSE](LICENSE) for details.
