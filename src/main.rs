use clap::{Parser, ValueEnum};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use toml::Value;

#[derive(Debug, Clone, ValueEnum)]
enum CollisionStrategy {
    /// Rename file with counter (e.g., file (1).txt)
    Rename,
    /// Skip file if it already exists
    Skip,
    /// Prompt user for each collision
    Prompt,
}

#[derive(Parser, Debug)]
#[command(author, version, about = "File organizer by type")]
struct Args {
    /// Source directory to scan
    #[arg(short, long)]
    source: String,

    /// Path to the config.toml file
    #[arg(short, long)]
    config: String,

    /// How to handle file name collisions
    #[arg(long, value_enum, default_value = "rename")]
    collision: CollisionStrategy,

    /// Preview changes without moving files
    #[arg(long)]
    dry_run: bool,
}

// Define file type mappings
fn get_file_type_mappings() -> HashMap<Vec<&'static str>, &'static str> {
    let mut mappings = HashMap::new();

    // Images
    mappings.insert(vec!["jpg", "jpeg", "png", "gif", "bmp", "tiff", "webp"], "Images");

    // Documents
    mappings.insert(vec!["pdf", "doc", "docx", "txt", "rtf", "odt", "xls", "xlsx", "ppt", "pptx"], "Documents");

    // Audio
    mappings.insert(vec!["mp3", "wav", "ogg", "flac", "aac", "wma"], "Audio");

    mappings
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

fn move_file(source: &Path, destination: &Path, strategy: &CollisionStrategy, dry_run: bool) -> std::io::Result<()> {
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
                    return Ok(());
                }
                CollisionStrategy::Prompt => {
                    if dry_run {
                        println!("[DRY RUN] File exists, would prompt: {}", file_name.to_str().unwrap());
                        return Ok(());
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
                            return Ok(());
                        }
                        "o" | "overwrite" => {
                            println!("Overwriting: {}", file_name.to_str().unwrap());
                            dest_path
                        }
                        _ => {
                            println!("Invalid input. Skipping file.");
                            return Ok(());
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

    Ok(())
}

fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

fn get_extension(file_path: &Path) -> Option<String> {
    file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
}

fn organize_files(args: Args) -> std::io::Result<()> {
    let source_dir = expand_tilde(&args.source);
    let config_path = expand_tilde(&args.config);

    if args.dry_run {
        println!("=== DRY RUN MODE: No files will be moved ===\n");
    }

    // Read and parse the config.toml file
    let config_content = fs::read_to_string(&config_path).map_err(|e| {
        std::io::Error::new(
            e.kind(),
            format!("Failed to read config file '{}': {}", config_path.display(), e),
        )
    })?;

    let config: Value = config_content.parse::<Value>().map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Failed to parse config file '{}': {}", config_path.display(), e),
        )
    })?;

    // Extract output directories from the config
    let directories = config.get("directories").ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Config file is missing required 'directories' section",
        )
    })?;

    let images_dir = expand_tilde(
        directories
            .get("images")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Config is missing 'directories.images' path or it's not a string",
                )
            })?,
    );

    let documents_dir = expand_tilde(
        directories
            .get("documents")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Config is missing 'directories.documents' path or it's not a string",
                )
            })?,
    );

    let audio_dir = expand_tilde(
        directories
            .get("audio")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Config is missing 'directories.audio' path or it's not a string",
                )
            })?,
    );

    let mappings = get_file_type_mappings();

    // Check if source directory exists
    if !source_dir.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Source directory does not exist: {}", source_dir.display()),
        ));
    }

    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        let path = entry.path();

        // Skip directories
        if path.is_dir() {
            continue;
        }

        if let Some(extension) = get_extension(&path) {
            // Find matching category for the file extension
            for (extensions, category) in &mappings {
                if extensions.iter().any(|&ext| ext == extension) {
                    let dest_dir = match *category {
                        "Images" => &images_dir,
                        "Documents" => &documents_dir,
                        "Audio" => &audio_dir,
                        _ => continue,
                    };
                    move_file(&path, dest_dir, &args.collision, args.dry_run)?;
                    break;
                }
            }
        }
    }

    Ok(())
}

fn main() {
    let args = Args::parse();

    if let Err(e) = organize_files(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
