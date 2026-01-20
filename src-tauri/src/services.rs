use crate::error::{AppError, Result};
use crate::models::*;
use kondo::{expand_tilde, resolve_config_path, Config, ExecuteOptions, KondoOrganizer};
use std::path::{Path, PathBuf};

// ============================================================================
// Service Layer - Business logic abstraction (Repository Pattern)
// ============================================================================

pub struct OrganizerService;

impl OrganizerService {
    /// Create a new organizer instance with validated paths and config
    fn create_organizer(path: &str, config_path: Option<String>) -> Result<KondoOrganizer> {
        let source_dir = Self::validate_directory(path)?;
        let config = Self::load_and_validate_config(config_path)?;

        Ok(KondoOrganizer::new(source_dir, config))
    }

    /// Validate that a directory exists and is accessible
    fn validate_directory(path: &str) -> Result<PathBuf> {
        let dir = expand_tilde(path);

        if !dir.exists() {
            return Err(AppError::path_error(format!(
                "Directory does not exist: {}",
                dir.display()
            )));
        }

        if !dir.is_dir() {
            return Err(AppError::path_error(format!(
                "Path is not a directory: {}",
                dir.display()
            )));
        }

        Ok(dir)
    }

    /// Load and validate configuration
    fn load_and_validate_config(config_path: Option<String>) -> Result<Config> {
        let path = resolve_config_path(config_path)
            .map_err(|e| AppError::config_error(format!("Failed to resolve config path: {}", e)))?;

        Config::from_file(&path)
            .map_err(|e| AppError::config_error(format!("Failed to load config: {}", e)))
    }

    /// Scan a directory and categorize files
    pub fn scan(request: DirectoryRequest) -> Result<ScanResponse> {
        let organizer = Self::create_organizer(&request.path, request.config_path)?;

        organizer
            .scan(request.recursive, request.follow_symlinks)
            .map_err(|e| AppError::operation_error(format!("Scan failed: {}", e)))
    }

    /// Preview file organization operations
    pub fn preview(request: DirectoryRequest) -> Result<PreviewResponse> {
        let organizer = Self::create_organizer(&request.path, request.config_path)?;

        let options = ExecuteOptions {
            collision_strategy: kondo::CollisionStrategy::Rename,
            dry_run: true,
            recursive: request.recursive,
            backup: false,
            follow_symlinks: request.follow_symlinks,
        };

        organizer
            .preview(&options)
            .map_err(|e| AppError::operation_error(format!("Preview failed: {}", e)))
    }

    /// Execute file organization
    pub fn execute(request: ExecuteRequest) -> Result<ExecuteResponse> {
        let organizer = Self::create_organizer(&request.path, request.config_path)?;

        let options = ExecuteOptions {
            collision_strategy: request.collision_strategy.into(),
            dry_run: request.dry_run,
            recursive: request.recursive,
            backup: request.backup,
            follow_symlinks: request.follow_symlinks,
        };

        let (stats, manifest) = organizer
            .execute(options)
            .map_err(|e| AppError::operation_error(format!("Execution failed: {}", e)))?;

        // Save backup manifest if created
        if let Some(manifest) = manifest {
            if !manifest.operations.is_empty() && !request.dry_run {
                Self::save_backup_manifest(&manifest)?;
            }
        }

        Ok(stats)
    }

    /// Save backup manifest to local data directory
    fn save_backup_manifest(manifest: &kondo::BackupManifest) -> Result<()> {
        let manifest_path = dirs::data_local_dir()
            .ok_or_else(|| AppError::path_error("Could not determine local data directory"))?
            .join("kondo")
            .join(format!(
                "backup-{}.json",
                chrono::Utc::now().format("%Y%m%d-%H%M%S")
            ));

        manifest
            .save(&manifest_path)
            .map_err(|e| AppError::operation_error(format!("Failed to save backup: {}", e)))
    }
}

pub struct ConfigService;

impl ConfigService {
    /// Load configuration from file or default location
    pub fn load(request: ConfigRequest) -> Result<ConfigResponse> {
        let path = resolve_config_path(request.path)
            .map_err(|e| AppError::config_error(format!("Failed to resolve config path: {}", e)))?;

        Config::from_file(&path)
            .map_err(|e| AppError::config_error(format!("Failed to load config: {}", e)))
    }

    /// Save configuration to file
    pub fn save(request: SaveConfigRequest) -> Result<()> {
        // Validate config before saving
        request
            .config
            .validate()
            .map_err(|e| AppError::validation_error(format!("Invalid config: {}", e)))?;

        let config_path = expand_tilde(&request.path);

        // Create parent directory if needed
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::path_error(format!("Failed to create config directory: {}", e))
            })?;
        }

        // Serialize and save
        let config_str = request
            .config
            .to_string()
            .map_err(|e| AppError::serialization(format!("Failed to serialize config: {}", e)))?;

        std::fs::write(&config_path, config_str)
            .map_err(|e| AppError::operation_error(format!("Failed to write config: {}", e)))?;

        Ok(())
    }

    /// Get the default configuration path
    pub fn get_default_path() -> Result<String> {
        kondo::get_default_config_path()
            .map(|p| p.display().to_string())
            .ok_or_else(|| AppError::path_error("Could not determine default config path"))
    }
}
