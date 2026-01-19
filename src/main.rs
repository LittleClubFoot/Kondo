use clap::{Parser, ValueEnum};
use serde::Deserialize;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, ValueEnum)]
enum CollisionStrategy {
    /// Rename file with counter (e.g., file (1).txt)
    Rename,
    /// Skip file if it already exists
    Skip,
    /// Prompt user for each collision
    Prompt,
}

#[derive(Debug, Deserialize)]
struct Config {
    categories: Vec<FileCategory>,
}

#[derive(Debug, Deserialize)]
struct FileCategory {
    name: String,
    extensions: Vec<String>,
    destination: String,
}

#[derive(Debug, Default)]
struct Statistics {
    total_files: usize,
    moved: usize,
    skipped: usize,
    no_category: usize,
}

impl Statistics {
    fn print_summary(&self, dry_run: bool) {
        let mode = if dry_run { "[DRY RUN] " } else { "" };
        println!("\n{}Summary:", mode);
        println!("  Total files processed: {}", self.total_files);
        println!("  Files moved: {}", self.moved);
        println!("  Files skipped (collision): {}", self.skipped);
        println!("  Files with no category: {}", self.no_category);
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about = "File organizer by type")]
struct Args {
    /// Source directory to scan
    #[arg(short, long)]
    source: String,

    /// Path to the config.toml file (defaults to ~/.config/kondo/config.toml)
    #[arg(short, long)]
    config: Option<String>,

    /// How to handle file name collisions
    #[arg(long, value_enum, default_value = "rename")]
    collision: CollisionStrategy,

    /// Preview changes without moving files
    #[arg(long)]
    dry_run: bool,

    /// Process subdirectories recursively
    #[arg(short, long)]
    recursive: bool,
}

impl Config {
    fn from_file(path: &Path) -> std::io::Result<Self> {
        let content = fs::read_to_string(path).map_err(|e| {
            std::io::Error::new(
                e.kind(),
                format!("Failed to read config file '{}': {}", path.display(), e),
            )
        })?;

        let config: Config = toml::from_str(&content).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse config file '{}': {}", path.display(), e),
            )
        })?;

        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> std::io::Result<()> {
        // Check if config has at least one category
        if self.categories.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Config must contain at least one category",
            ));
        }

        // Validate each category
        for (idx, category) in self.categories.iter().enumerate() {
            // Check category name is not empty
            if category.name.trim().is_empty() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Category {} has an empty name", idx + 1),
                ));
            }

            // Check extensions list is not empty
            if category.extensions.is_empty() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Category '{}' has no extensions defined", category.name),
                ));
            }

            // Check for empty extensions
            for ext in &category.extensions {
                if ext.trim().is_empty() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Category '{}' contains an empty extension", category.name),
                    ));
                }
            }

            // Check destination is not empty
            if category.destination.trim().is_empty() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Category '{}' has an empty destination path", category.name),
                ));
            }
        }

        // Check for duplicate extensions across categories
        let mut seen_extensions = std::collections::HashMap::new();
        for category in &self.categories {
            for ext in &category.extensions {
                let ext_lower = ext.to_lowercase();
                if let Some(existing_category) = seen_extensions.get(&ext_lower) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "Extension '{}' is defined in both '{}' and '{}' categories. Each extension can only belong to one category.",
                            ext, existing_category, category.name
                        ),
                    ));
                }
                seen_extensions.insert(ext_lower, &category.name);
            }
        }

        Ok(())
    }

    fn find_category_for_extension(&self, extension: &str) -> Option<&FileCategory> {
        self.categories
            .iter()
            .find(|cat| cat.extensions.iter().any(|ext| ext == extension))
    }
}

fn find_available_filename(base_path: &Path) -> PathBuf {
    if !base_path.exists() {
        return base_path.to_path_buf();
    }

    let parent = base_path.parent().unwrap();
    let file_stem = base_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let extension = base_path.extension().and_then(|s| s.to_str());

    let mut counter = 1;
    loop {
        let new_name = match extension {
            Some(ext) => format!("{} ({}).{}", file_stem, counter, ext),
            None => format!("{} ({})", file_stem, counter),
        };
        let new_path = parent.join(new_name);
        if !new_path.exists() {
            return new_path;
        }
        counter += 1;
    }
}

fn prompt_user_action(file_name: &str) -> std::io::Result<String> {
    print!("File '{}' already exists. [r]ename, [s]kip, or [o]verwrite? ", file_name);
    io::stdout().flush()?;

    let mut response = String::new();
    io::stdin().read_line(&mut response)?;

    Ok(response.trim().to_lowercase())
}

enum MoveResult {
    Moved,
    Skipped,
}

fn move_file(source: &Path, destination: &Path, strategy: &CollisionStrategy, dry_run: bool) -> std::io::Result<MoveResult> {
    // Create destination directory if it doesn't exist (skip in dry-run mode)
    if !dry_run {
        fs::create_dir_all(destination)?;
    }

    if let Some(file_name) = source.file_name() {
        let dest_path = destination.join(file_name);

        // Handle collision if file exists
        let final_dest = if dest_path.exists() {
            match strategy {
                CollisionStrategy::Rename => {
                    let new_path = find_available_filename(&dest_path);
                    if dry_run {
                        println!("[DRY RUN] Would rename due to collision: {}", new_path.file_name().unwrap().to_str().unwrap());
                    } else {
                        println!("Collision detected: renaming to {}", new_path.file_name().unwrap().to_str().unwrap());
                    }
                    new_path
                }
                CollisionStrategy::Skip => {
                    if dry_run {
                        println!("[DRY RUN] Would skip: {} (already exists)", dest_path.display());
                    } else {
                        println!("Skipped: {} (already exists)", dest_path.display());
                    }
                    return Ok(MoveResult::Skipped);
                }
                CollisionStrategy::Prompt => {
                    if dry_run {
                        println!("[DRY RUN] File exists, would prompt: {}", file_name.to_str().unwrap());
                        return Ok(MoveResult::Moved);
                    }
                    let action = prompt_user_action(file_name.to_str().unwrap())?;
                    match action.as_str() {
                        "r" | "rename" => {
                            let new_path = find_available_filename(&dest_path);
                            println!("Renaming to {}", new_path.file_name().unwrap().to_str().unwrap());
                            new_path
                        }
                        "s" | "skip" => {
                            println!("Skipped: {}", file_name.to_str().unwrap());
                            return Ok(MoveResult::Skipped);
                        }
                        "o" | "overwrite" => {
                            println!("Overwriting: {}", file_name.to_str().unwrap());
                            dest_path
                        }
                        _ => {
                            println!("Invalid input. Skipping file.");
                            return Ok(MoveResult::Skipped);
                        }
                    }
                }
            }
        } else {
            dest_path
        };

        if dry_run {
            println!("[DRY RUN] Would move: {} -> {}", source.display(), final_dest.display());
        } else {
            // Try rename first (fast for same filesystem)
            match fs::rename(source, &final_dest) {
                Ok(_) => {
                    println!("Moved: {} -> {}", source.display(), final_dest.display());
                }
                Err(e) => {
                    // If rename fails (e.g., cross-filesystem), fall back to copy + remove
                    if e.raw_os_error() == Some(18) || e.kind() == std::io::ErrorKind::CrossesDevices {
                        fs::copy(source, &final_dest)?;
                        fs::remove_file(source)?;
                        println!("Moved (cross-filesystem): {} -> {}", source.display(), final_dest.display());
                    } else {
                        // If it's a different error, propagate it
                        return Err(e);
                    }
                }
            }
        }
    }

    Ok(MoveResult::Moved)
}

fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

fn get_default_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|config_dir| config_dir.join("kondo").join("config.toml"))
}

fn resolve_config_path(config_arg: Option<String>) -> std::io::Result<PathBuf> {
    match config_arg {
        Some(path) => Ok(expand_tilde(&path)),
        None => {
            get_default_config_path().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Could not determine config directory. Please specify config path with --config",
                )
            })
        }
    }
}

fn get_extension(file_path: &Path) -> Option<String> {
    file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
}

fn organize_files(args: Args) -> std::io::Result<()> {
    let source_dir = expand_tilde(&args.source);
    let config_path = resolve_config_path(args.config)?;

    if args.dry_run {
        println!("=== DRY RUN MODE: No files will be moved ===\n");
    }

    // Load and parse config with type safety (validation happens automatically)
    let config = Config::from_file(&config_path)?;

    // Check if source directory exists
    if !source_dir.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Source directory does not exist: {}", source_dir.display()),
        ));
    }

    let mut stats = Statistics::default();

    // Process files based on recursive flag
    if args.recursive {
        // Use walkdir for recursive traversal
        for entry in WalkDir::new(&source_dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            // Skip directories
            if path.is_dir() {
                continue;
            }

            stats.total_files += 1;

            // Get file extension and find matching category
            if let Some(extension) = get_extension(path) {
                if let Some(category) = config.find_category_for_extension(&extension) {
                    let dest_dir = expand_tilde(&category.destination);
                    match move_file(path, &dest_dir, &args.collision, args.dry_run)? {
                        MoveResult::Moved => stats.moved += 1,
                        MoveResult::Skipped => stats.skipped += 1,
                    }
                } else {
                    stats.no_category += 1;
                }
            } else {
                stats.no_category += 1;
            }
        }
    } else {
        // Process only top-level directory
        for entry in fs::read_dir(&source_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Skip directories
            if path.is_dir() {
                continue;
            }

            stats.total_files += 1;

            // Get file extension and find matching category
            if let Some(extension) = get_extension(&path) {
                if let Some(category) = config.find_category_for_extension(&extension) {
                    let dest_dir = expand_tilde(&category.destination);
                    match move_file(&path, &dest_dir, &args.collision, args.dry_run)? {
                        MoveResult::Moved => stats.moved += 1,
                        MoveResult::Skipped => stats.skipped += 1,
                    }
                } else {
                    stats.no_category += 1;
                }
            } else {
                stats.no_category += 1;
            }
        }
    }

    stats.print_summary(args.dry_run);
    Ok(())
}

fn main() {
    let args = Args::parse();

    if let Err(e) = organize_files(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_get_extension() {
        assert_eq!(get_extension(Path::new("file.txt")), Some("txt".to_string()));
        assert_eq!(get_extension(Path::new("file.PDF")), Some("pdf".to_string()));
        assert_eq!(get_extension(Path::new("file.tar.gz")), Some("gz".to_string()));
        assert_eq!(get_extension(Path::new("file")), None);
        assert_eq!(get_extension(Path::new(".gitignore")), None);
    }

    #[test]
    fn test_expand_tilde_with_home() {
        let result = expand_tilde("~/test/path");
        if let Some(home) = dirs::home_dir() {
            assert_eq!(result, home.join("test/path"));
        }
    }

    #[test]
    fn test_expand_tilde_without_tilde() {
        let path = "/absolute/path";
        assert_eq!(expand_tilde(path), PathBuf::from(path));

        let relative = "relative/path";
        assert_eq!(expand_tilde(relative), PathBuf::from(relative));
    }

    #[test]
    fn test_config_validation_empty_categories() {
        let config = Config { categories: vec![] };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_empty_name() {
        let config = Config {
            categories: vec![FileCategory {
                name: "   ".to_string(),
                extensions: vec!["txt".to_string()],
                destination: "~/test".to_string(),
            }],
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_empty_extensions() {
        let config = Config {
            categories: vec![FileCategory {
                name: "Test".to_string(),
                extensions: vec![],
                destination: "~/test".to_string(),
            }],
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_empty_destination() {
        let config = Config {
            categories: vec![FileCategory {
                name: "Test".to_string(),
                extensions: vec!["txt".to_string()],
                destination: "  ".to_string(),
            }],
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_duplicate_extensions() {
        let config = Config {
            categories: vec![
                FileCategory {
                    name: "Category1".to_string(),
                    extensions: vec!["txt".to_string()],
                    destination: "~/test1".to_string(),
                },
                FileCategory {
                    name: "Category2".to_string(),
                    extensions: vec!["TXT".to_string()], // Case insensitive duplicate
                    destination: "~/test2".to_string(),
                },
            ],
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_validation_valid() {
        let config = Config {
            categories: vec![
                FileCategory {
                    name: "Images".to_string(),
                    extensions: vec!["jpg".to_string(), "png".to_string()],
                    destination: "~/Pictures".to_string(),
                },
                FileCategory {
                    name: "Documents".to_string(),
                    extensions: vec!["pdf".to_string(), "doc".to_string()],
                    destination: "~/Documents".to_string(),
                },
            ],
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_find_category_for_extension() {
        let config = Config {
            categories: vec![
                FileCategory {
                    name: "Images".to_string(),
                    extensions: vec!["jpg".to_string(), "png".to_string()],
                    destination: "~/Pictures".to_string(),
                },
                FileCategory {
                    name: "Documents".to_string(),
                    extensions: vec!["pdf".to_string(), "doc".to_string()],
                    destination: "~/Documents".to_string(),
                },
            ],
        };

        assert!(config.find_category_for_extension("jpg").is_some());
        assert_eq!(
            config.find_category_for_extension("jpg").unwrap().name,
            "Images"
        );
        assert!(config.find_category_for_extension("pdf").is_some());
        assert!(config.find_category_for_extension("mp3").is_none());
    }

    #[test]
    fn test_find_available_filename() {
        use std::fs;

        let temp_dir = std::env::temp_dir().join("kondo_test");
        fs::create_dir_all(&temp_dir).unwrap();

        let test_file = temp_dir.join("test.txt");
        fs::write(&test_file, "test").unwrap();

        let available = find_available_filename(&test_file);
        assert_eq!(available, temp_dir.join("test (1).txt"));

        // Create the first collision file
        fs::write(&available, "test").unwrap();
        let available2 = find_available_filename(&test_file);
        assert_eq!(available2, temp_dir.join("test (2).txt"));

        // Cleanup
        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_config_from_toml() {
        let toml_content = r#"
[[categories]]
name = "Images"
extensions = ["jpg", "png"]
destination = "~/Pictures"

[[categories]]
name = "Documents"
extensions = ["pdf", "txt"]
destination = "~/Documents"
"#;

        let temp_dir = std::env::temp_dir().join("kondo_test_config");
        fs::create_dir_all(&temp_dir).unwrap();

        let config_file = temp_dir.join("test_config.toml");
        let mut file = fs::File::create(&config_file).unwrap();
        file.write_all(toml_content.as_bytes()).unwrap();

        let config = Config::from_file(&config_file).unwrap();
        assert_eq!(config.categories.len(), 2);
        assert_eq!(config.categories[0].name, "Images");
        assert_eq!(config.categories[0].extensions.len(), 2);

        fs::remove_dir_all(&temp_dir).unwrap();
    }

    #[test]
    fn test_statistics_default() {
        let stats = Statistics::default();
        assert_eq!(stats.total_files, 0);
        assert_eq!(stats.moved, 0);
        assert_eq!(stats.skipped, 0);
        assert_eq!(stats.no_category, 0);
    }
}
