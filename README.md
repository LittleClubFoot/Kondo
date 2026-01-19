# Kondo

A simple, safe, and configurable file organizer inspired by Marie Kondo's tidying method. Automatically organize cluttered directories by sorting files into categories based on their extensions.

## Features

- 🎯 **Config-driven** - Define your own file categories and destinations
- 🛡️ **Safe** - Multiple collision handling strategies to prevent data loss
- 👀 **Dry-run mode** - Preview changes before executing
- 🚀 **Cross-filesystem support** - Move files across different partitions
- 🏠 **Tilde expansion** - Use `~/` paths in config and arguments
- ⚡ **Fast** - Written in Rust for performance and safety
- 📝 **Helpful errors** - Clear, actionable error messages

## Installation

### Prerequisites

- [Rust](https://rustup.rs/) 1.70 or later

### Build from source

```bash
git clone https://github.com/yourusername/kondo.git
cd kondo
cargo build --release
cargo install --path .
```

The binary will be available at `target/release/kondo` or installed to `~/.cargo/bin/kondo`.

## Quick Start

1. Create a config file (see [Configuration](#configuration))
2. Run Kondo with dry-run first:

```bash
kondo --source ~/Downloads --config config.toml --dry-run
```

3. If everything looks good, run it for real:

```bash
kondo --source ~/Downloads --config config.toml
```

## Usage

```bash
kondo --source <SOURCE_DIR> --config <CONFIG_FILE> [OPTIONS]
```

### Arguments

- `--source, -s <DIR>` - Directory to organize (supports `~/` paths)
- `--config, -c <FILE>` - Path to configuration file (supports `~/` paths)

### Options

- `--dry-run` - Preview what would happen without moving files
- `--collision <STRATEGY>` - How to handle file name collisions (default: `rename`)
  - `rename` - Automatically append counter (file.txt → file (1).txt)
  - `skip` - Don't move files that already exist
  - `prompt` - Ask for each collision (rename/skip/overwrite)

### Examples

**Preview changes before organizing:**
```bash
kondo -s ~/Downloads -c ~/config.toml --dry-run
```

**Organize with automatic renaming on collisions:**
```bash
kondo -s ~/Downloads -c ~/config.toml
# or explicitly
kondo -s ~/Downloads -c ~/config.toml --collision rename
```

**Skip files that already exist:**
```bash
kondo -s ~/Downloads -c ~/config.toml --collision skip
```

**Interactively decide for each collision:**
```bash
kondo -s ~/Downloads -c ~/config.toml --collision prompt
```

## Configuration

Configuration is done via TOML files. Each category defines:
- `name` - Category name (for logging/debugging)
- `extensions` - List of file extensions (without dots)
- `destination` - Where to move matching files (supports `~/` paths)

### Example config.toml

```toml
[[categories]]
name = "Images"
extensions = ["jpg", "jpeg", "png", "gif", "bmp", "tiff", "webp", "svg", "ico"]
destination = "~/Pictures"

[[categories]]
name = "Documents"
extensions = ["pdf", "doc", "docx", "txt", "rtf", "odt", "xls", "xlsx", "ppt", "pptx", "csv", "md"]
destination = "~/Documents"

[[categories]]
name = "Audio"
extensions = ["mp3", "wav", "ogg", "flac", "aac", "wma", "m4a", "opus"]
destination = "~/Music"

[[categories]]
name = "Videos"
extensions = ["mp4", "avi", "mkv", "mov", "wmv", "flv", "webm", "m4v", "mpeg"]
destination = "~/Videos"

[[categories]]
name = "Archives"
extensions = ["zip", "tar", "gz", "bz2", "7z", "rar", "xz", "tgz"]
destination = "~/Downloads/Archives"

[[categories]]
name = "Code"
extensions = ["rs", "py", "js", "ts", "java", "c", "cpp", "h", "hpp", "go", "rb", "php", "swift", "kt"]
destination = "~/Code"
```

### Customizing Categories

You can add, remove, or modify categories to suit your needs:

```toml
# Custom category for design files
[[categories]]
name = "Design"
extensions = ["psd", "ai", "sketch", "fig", "xd"]
destination = "~/Design"

# Category for ebooks
[[categories]]
name = "Books"
extensions = ["epub", "mobi", "azw", "azw3"]
destination = "~/Books"
```

### Extension Matching

- Extensions are case-insensitive (`.JPG` and `.jpg` both match)
- The first matching category wins (order matters!)
- Files with no matching category are left untouched

## Architecture & Design

Kondo follows these design principles:

### Type Safety
- Uses `serde` for type-safe config deserialization
- No raw string matching or magic values
- Config validation at parse time

### Separation of Concerns
- Config loading separated from file operations
- Clear, single-purpose functions
- Easy to test and maintain

### Extensibility
- Config-driven instead of hardcoded mappings
- Easy to add new file categories without code changes
- Open for extension, closed for modification

### Error Handling
- No panics - all errors are handled gracefully
- Descriptive error messages with context
- Clear indication of what went wrong and how to fix it

## Safety Features

### File Collision Handling
When a file with the same name already exists in the destination:
- **Rename mode** (default): Creates `filename (1).ext`, `filename (2).ext`, etc.
- **Skip mode**: Leaves the original file untouched
- **Prompt mode**: Asks you for each collision

### Dry-Run Mode
Always test with `--dry-run` first to see what would happen:
```
[DRY RUN] Would move: /home/user/Downloads/photo.jpg -> /home/user/Pictures/photo.jpg
[DRY RUN] Would rename due to collision: photo (1).jpg
```

### Cross-Filesystem Support
Automatically handles moving files across different partitions or drives using a safe copy+delete approach when needed.

## Limitations

- Only processes files in the top-level directory (not recursive)
- Directories are skipped
- Symlinks are not followed
- Files without extensions are ignored

## Contributing

Contributions welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

## Acknowledgments

Named after Marie Kondo, whose tidying methods inspire joy and organization.
