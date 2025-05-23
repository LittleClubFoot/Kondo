use clap::Parser;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use toml::Value;

#[derive(Parser, Debug)]
#[command(author, version, about = "File organizer by type")]
struct Args {
    /// Source directory to scan
    #[arg(short, long)]
    source: String,

    /// Path to the config.toml file
    #[arg(short, long)]
    config: String,
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

fn move_file(source: &Path, destination: &Path) -> std::io::Result<()> {
    // Create destination directory if it doesn't exist
    fs::create_dir_all(destination)?;

    if let Some(file_name) = source.file_name() {
        let dest_path = destination.join(file_name);
        fs::rename(source, dest_path)?;
        println!("Moved: {} -> {}", source.display(), destination.display());
    }

    Ok(())
}

fn get_extension(file_path: &Path) -> Option<String> {
    file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
}

fn organize_files(args: Args) -> std::io::Result<()> {
    let source_dir = PathBuf::from(args.source);
    let config_path = PathBuf::from(args.config);

    // Read and parse the config.toml file
    let config_content = fs::read_to_string(config_path)?;
    let config: Value = config_content.parse::<Value>()
        .expect("Failed to parse the config file");

    // Extract output directories from the config
    let directories = config.get("directories").expect("Missing 'directories' section in config");
    let images_dir = PathBuf::from(directories.get("images").expect("Missing 'images' key in config").as_str().unwrap());
    let documents_dir = PathBuf::from(directories.get("documents").expect("Missing 'documents' key in config").as_str().unwrap());
    let audio_dir = PathBuf::from(directories.get("audio").expect("Missing 'audio' key in config").as_str().unwrap());

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
                    move_file(&path, dest_dir)?;
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
