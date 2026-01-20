use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// ============================================================================
// Public Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CollisionStrategy {
    Rename,
    Skip,
    Prompt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub categories: Vec<FileCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileCategory {
    pub name: String,
    pub extensions: Vec<String>,
    pub destination: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Statistics {
    pub total_files: usize,
    pub moved: usize,
    pub skipped: usize,
    pub no_category: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileOperation {
    pub timestamp: String,
    pub source: String,
    pub destination: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub created_at: String,
    pub operations: Vec<FileOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedOperation {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub category: String,
    pub collision: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub total_files: usize,
    pub by_category: HashMap<String, Vec<FileInfo>>,
    pub no_category: Vec<FileInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: PathBuf,
    pub name: String,
    pub extension: Option<String>,
    pub size: u64,
}

#[derive(Debug, Clone)]
pub struct ExecuteOptions {
    pub collision_strategy: CollisionStrategy,
    pub dry_run: bool,
    pub recursive: bool,
    pub backup: bool,
    pub follow_symlinks: bool,
}

// ============================================================================
// Public API - Main Organizer
// ============================================================================

pub struct KondoOrganizer {
    config: Config,
    source_dir: PathBuf,
}

impl KondoOrganizer {
    pub fn new(source: PathBuf, config: Config) -> Self {
        Self {
            config,
            source_dir: source,
        }
    }

    /// Scan the source directory and categorize files
    pub fn scan(&self, recursive: bool, follow_symlinks: bool) -> std::io::Result<ScanResult> {
        let mut by_category: HashMap<String, Vec<FileInfo>> = HashMap::new();
        let mut no_category = Vec::new();
        let mut total_files = 0;

        // Initialize category buckets
        for category in &self.config.categories {
            by_category.insert(category.name.clone(), Vec::new());
        }

        let file_iterator: Box<dyn Iterator<Item = PathBuf>> = if recursive {
            Box::new(
                WalkDir::new(&self.source_dir)
                    .follow_links(follow_symlinks)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .map(|e| e.path().to_path_buf())
                    .filter(|p| p.is_file() && (!p.is_symlink() || follow_symlinks)),
            )
        } else {
            Box::new(
                fs::read_dir(&self.source_dir)
                    .map_err(|e| {
                        std::io::Error::new(
                            e.kind(),
                            format!("Failed to read directory: {}", e),
                        )
                    })?
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| p.is_file() && (!p.is_symlink() || follow_symlinks)),
            )
        };

        for path in file_iterator {
            total_files += 1;
            let file_info = create_file_info(&path)?;

            if let Some(extension) = &file_info.extension {
                if let Some(category) = self.config.find_category_for_extension(extension) {
                    by_category
                        .get_mut(&category.name)
                        .unwrap()
                        .push(file_info);
                } else {
                    no_category.push(file_info);
                }
            } else {
                no_category.push(file_info);
            }
        }

        Ok(ScanResult {
            total_files,
            by_category,
            no_category,
        })
    }

    /// Preview what operations would be performed
    pub fn preview(&self, options: &ExecuteOptions) -> std::io::Result<Vec<PlannedOperation>> {
        let mut operations = Vec::new();

        let file_iterator: Box<dyn Iterator<Item = PathBuf>> = if options.recursive {
            Box::new(
                WalkDir::new(&self.source_dir)
                    .follow_links(options.follow_symlinks)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .map(|e| e.path().to_path_buf())
                    .filter(|p| p.is_file() && (!p.is_symlink() || options.follow_symlinks)),
            )
        } else {
            Box::new(
                fs::read_dir(&self.source_dir)?
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| p.is_file() && (!p.is_symlink() || options.follow_symlinks)),
            )
        };

        for path in file_iterator {
            if let Some(extension) = get_extension(&path) {
                if let Some(category) = self.config.find_category_for_extension(&extension) {
                    let dest_dir = expand_tilde(&category.destination);
                    if let Some(file_name) = path.file_name() {
                        let dest_path = dest_dir.join(file_name);
                        let collision = dest_path.exists();

                        operations.push(PlannedOperation {
                            source: path.clone(),
                            destination: dest_path,
                            category: category.name.clone(),
                            collision,
                        });
                    }
                }
            }
        }

        Ok(operations)
    }

    /// Execute the file organization
    pub fn execute(&self, options: ExecuteOptions) -> std::io::Result<(Statistics, Option<BackupManifest>)> {
        let mut stats = Statistics::default();
        let mut manifest = if options.backup {
            Some(BackupManifest::new())
        } else {
            None
        };

        // Validate all category destinations
        for category in &self.config.categories {
            let dest = expand_tilde(&category.destination);
            if let Err(e) = validate_destination(&dest) {
                eprintln!(
                    "Warning: Invalid destination for category '{}': {}",
                    category.name, e
                );
            }
        }

        let file_iterator: Box<dyn Iterator<Item = PathBuf>> = if options.recursive {
            Box::new(
                WalkDir::new(&self.source_dir)
                    .follow_links(options.follow_symlinks)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .map(|e| e.path().to_path_buf())
                    .filter(|p| {
                        p.is_file() && (!p.is_symlink() || options.follow_symlinks)
                    }),
            )
        } else {
            Box::new(
                fs::read_dir(&self.source_dir)?
                    .filter_map(|e| e.ok())
                    .map(|e| e.path())
                    .filter(|p| {
                        p.is_file() && (!p.is_symlink() || options.follow_symlinks)
                    }),
            )
        };

        for path in file_iterator {
            stats.total_files += 1;

            if let Some(extension) = get_extension(&path) {
                if let Some(category) = self.config.find_category_for_extension(&extension) {
                    let dest_dir = expand_tilde(&category.destination);
                    match move_file(&path, &dest_dir, &options.collision_strategy, options.dry_run)? {
                        MoveResult::Moved(final_dest) => {
                            stats.moved += 1;
                            if let Some(ref mut m) = manifest {
                                m.add_operation(&path, &final_dest);
                            }
                        }
                        MoveResult::Skipped => stats.skipped += 1,
                    }
                } else {
                    stats.no_category += 1;
                }
            } else {
                stats.no_category += 1;
            }
        }

        Ok((stats, manifest))
    }
}

// ============================================================================
// Config Implementation
// ============================================================================

impl Config {
    pub fn from_file(path: &Path) -> std::io::Result<Self> {
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

    pub fn from_str(content: &str) -> std::io::Result<Self> {
        let config: Config = toml::from_str(content).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse config: {}", e),
            )
        })?;

        config.validate()?;
        Ok(config)
    }

    pub fn to_string(&self) -> std::io::Result<String> {
        toml::to_string_pretty(self).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to serialize config: {}", e),
            )
        })
    }

    pub fn validate(&self) -> std::io::Result<()> {
        if self.categories.is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Config must contain at least one category",
            ));
        }

        for (idx, category) in self.categories.iter().enumerate() {
            if category.name.trim().is_empty() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Category {} has an empty name", idx + 1),
                ));
            }

            if category.extensions.is_empty() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Category '{}' has no extensions defined", category.name),
                ));
            }

            for ext in &category.extensions {
                if ext.trim().is_empty() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!("Category '{}' contains an empty extension", category.name),
                    ));
                }
            }

            if category.destination.trim().is_empty() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("Category '{}' has an empty destination path", category.name),
                ));
            }
        }

        // Check for duplicate extensions
        let mut seen_extensions = HashMap::new();
        for category in &self.categories {
            for ext in &category.extensions {
                let ext_lower = ext.to_lowercase();
                if let Some(existing_category) = seen_extensions.get(&ext_lower) {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        format!(
                            "Extension '{}' is defined in both '{}' and '{}' categories",
                            ext, existing_category, category.name
                        ),
                    ));
                }
                seen_extensions.insert(ext_lower, &category.name);
            }
        }

        Ok(())
    }

    pub fn find_category_for_extension(&self, extension: &str) -> Option<&FileCategory> {
        self.categories
            .iter()
            .find(|cat| cat.extensions.iter().any(|ext| ext == extension))
    }
}

// ============================================================================
// BackupManifest Implementation
// ============================================================================

impl BackupManifest {
    pub fn new() -> Self {
        Self {
            created_at: Utc::now().to_rfc3339(),
            operations: Vec::new(),
        }
    }

    pub fn add_operation(&mut self, source: &Path, destination: &Path) {
        self.operations.push(FileOperation {
            timestamp: Utc::now().to_rfc3339(),
            source: source.display().to_string(),
            destination: destination.display().to_string(),
        });
    }

    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }

    pub fn load(path: &Path) -> std::io::Result<Self> {
        let content = fs::read_to_string(path)?;
        serde_json::from_str(&content).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse backup manifest: {}", e),
            )
        })
    }
}

impl Statistics {
    pub fn print_summary(&self, dry_run: bool) {
        let mode = if dry_run { "[DRY RUN] " } else { "" };
        println!("\n{}Summary:", mode);
        println!("  Total files processed: {}", self.total_files);
        println!("  Files moved: {}", self.moved);
        println!("  Files skipped (collision): {}", self.skipped);
        println!("  Files with no category: {}", self.no_category);
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

enum MoveResult {
    Moved(PathBuf),
    Skipped,
}

fn move_file(
    source: &Path,
    destination: &Path,
    strategy: &CollisionStrategy,
    dry_run: bool,
) -> std::io::Result<MoveResult> {
    if !dry_run {
        fs::create_dir_all(destination)?;
    }

    if let Some(file_name) = source.file_name() {
        let dest_path = destination.join(file_name);

        let final_dest = if dest_path.exists() {
            match strategy {
                CollisionStrategy::Rename => {
                    let new_path = find_available_filename(&dest_path);
                    if dry_run {
                        println!(
                            "[DRY RUN] Would rename due to collision: {}",
                            new_path.file_name().unwrap().to_str().unwrap()
                        );
                    } else {
                        println!(
                            "Collision detected: renaming to {}",
                            new_path.file_name().unwrap().to_str().unwrap()
                        );
                    }
                    new_path
                }
                CollisionStrategy::Skip => {
                    if dry_run {
                        println!(
                            "[DRY RUN] Would skip: {} (already exists)",
                            dest_path.display()
                        );
                    } else {
                        println!("Skipped: {} (already exists)", dest_path.display());
                    }
                    return Ok(MoveResult::Skipped);
                }
                CollisionStrategy::Prompt => {
                    if dry_run {
                        println!(
                            "[DRY RUN] File exists, would prompt: {}",
                            file_name.to_str().unwrap()
                        );
                        return Ok(MoveResult::Moved(dest_path.clone()));
                    }
                    let action = prompt_user_action(file_name.to_str().unwrap())?;
                    match action.as_str() {
                        "r" | "rename" => {
                            let new_path = find_available_filename(&dest_path);
                            println!(
                                "Renaming to {}",
                                new_path.file_name().unwrap().to_str().unwrap()
                            );
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
            println!(
                "[DRY RUN] Would move: {} -> {}",
                source.display(),
                final_dest.display()
            );
        } else {
            match fs::rename(source, &final_dest) {
                Ok(_) => {
                    println!("Moved: {} -> {}", source.display(), final_dest.display());
                }
                Err(e) => {
                    if e.raw_os_error() == Some(18)
                        || e.kind() == std::io::ErrorKind::CrossesDevices
                    {
                        fs::copy(source, &final_dest)?;
                        fs::remove_file(source)?;
                        println!(
                            "Moved (cross-filesystem): {} -> {}",
                            source.display(),
                            final_dest.display()
                        );
                    } else {
                        return Err(e);
                    }
                }
            }
        }

        return Ok(MoveResult::Moved(final_dest));
    }

    Ok(MoveResult::Moved(source.to_path_buf()))
}

fn find_available_filename(base_path: &Path) -> PathBuf {
    if !base_path.exists() {
        return base_path.to_path_buf();
    }

    let parent = base_path.parent().unwrap();
    let file_stem = base_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("");
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
    print!(
        "File '{}' already exists. [r]ename, [s]kip, or [o]verwrite? ",
        file_name
    );
    io::stdout().flush()?;

    let mut response = String::new();
    io::stdin().read_line(&mut response)?;

    Ok(response.trim().to_lowercase())
}

pub fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}

pub fn get_default_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|config_dir| config_dir.join("kondo").join("config.toml"))
}

pub fn resolve_config_path(config_arg: Option<String>) -> std::io::Result<PathBuf> {
    match config_arg {
        Some(path) => Ok(expand_tilde(&path)),
        None => get_default_config_path().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine config directory. Please specify config path with --config",
            )
        }),
    }
}

pub fn get_extension(file_path: &Path) -> Option<String> {
    file_path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|s| s.to_lowercase())
}

fn is_safe_path(path: &Path) -> bool {
    for component in path.components() {
        if component == std::path::Component::ParentDir {
            return false;
        }
    }
    true
}

pub fn validate_destination(dest: &Path) -> std::io::Result<()> {
    if !is_safe_path(dest) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Unsafe destination path detected: {}", dest.display()),
        ));
    }

    let expanded = dest.canonicalize().or_else(|_| {
        if let Some(parent) = dest.parent() {
            parent
                .canonicalize()
                .map(|p| p.join(dest.file_name().unwrap()))
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Cannot validate destination path",
            ))
        }
    })?;

    if !expanded.is_absolute() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Destination must be an absolute path: {}", dest.display()),
        ));
    }

    Ok(())
}

fn create_file_info(path: &Path) -> std::io::Result<FileInfo> {
    let metadata = fs::metadata(path)?;
    let extension = get_extension(path);
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    Ok(FileInfo {
        path: path.to_path_buf(),
        name,
        extension,
        size: metadata.len(),
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_extension() {
        assert_eq!(
            get_extension(Path::new("file.txt")),
            Some("txt".to_string())
        );
        assert_eq!(
            get_extension(Path::new("file.PDF")),
            Some("pdf".to_string())
        );
        assert_eq!(
            get_extension(Path::new("file.tar.gz")),
            Some("gz".to_string())
        );
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
        let config = Config {
            categories: vec![],
        };
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
                    extensions: vec!["TXT".to_string()],
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
        let temp_dir = std::env::temp_dir().join("kondo_test");
        fs::create_dir_all(&temp_dir).unwrap();

        let test_file = temp_dir.join("test.txt");
        fs::write(&test_file, "test").unwrap();

        let available = find_available_filename(&test_file);
        assert_eq!(available, temp_dir.join("test (1).txt"));

        fs::write(&available, "test").unwrap();
        let available2 = find_available_filename(&test_file);
        assert_eq!(available2, temp_dir.join("test (2).txt"));

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
        fs::write(&config_file, toml_content).unwrap();

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
