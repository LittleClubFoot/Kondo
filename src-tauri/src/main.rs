// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use kondo::{
    expand_tilde, resolve_config_path, CollisionStrategy, Config, ExecuteOptions, FileInfo,
    KondoOrganizer, PlannedOperation, ScanResult, Statistics,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// ============================================================================
// Tauri Command Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScanRequest {
    path: String,
    config_path: Option<String>,
    recursive: bool,
    follow_symlinks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PreviewRequest {
    path: String,
    config_path: Option<String>,
    recursive: bool,
    follow_symlinks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExecuteRequest {
    path: String,
    config_path: Option<String>,
    collision_strategy: String,
    dry_run: bool,
    recursive: bool,
    backup: bool,
    follow_symlinks: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigSaveRequest {
    config: Config,
    path: String,
}

// ============================================================================
// Tauri Commands
// ============================================================================

#[tauri::command]
async fn scan_directory(request: ScanRequest) -> Result<ScanResult, String> {
    let source_dir = expand_tilde(&request.path);

    if !source_dir.exists() {
        return Err(format!(
            "Source directory does not exist: {}",
            source_dir.display()
        ));
    }

    let config_path = resolve_config_path(request.config_path)
        .map_err(|e| format!("Failed to resolve config path: {}", e))?;

    let config = Config::from_file(&config_path)
        .map_err(|e| format!("Failed to load config: {}", e))?;

    let organizer = KondoOrganizer::new(source_dir, config);

    organizer
        .scan(request.recursive, request.follow_symlinks)
        .map_err(|e| format!("Failed to scan directory: {}", e))
}

#[tauri::command]
async fn preview_organization(request: PreviewRequest) -> Result<Vec<PlannedOperation>, String> {
    let source_dir = expand_tilde(&request.path);

    if !source_dir.exists() {
        return Err(format!(
            "Source directory does not exist: {}",
            source_dir.display()
        ));
    }

    let config_path = resolve_config_path(request.config_path)
        .map_err(|e| format!("Failed to resolve config path: {}", e))?;

    let config = Config::from_file(&config_path)
        .map_err(|e| format!("Failed to load config: {}", e))?;

    let organizer = KondoOrganizer::new(source_dir, config);

    let options = ExecuteOptions {
        collision_strategy: CollisionStrategy::Rename,
        dry_run: true,
        recursive: request.recursive,
        backup: false,
        follow_symlinks: request.follow_symlinks,
    };

    organizer
        .preview(&options)
        .map_err(|e| format!("Failed to preview: {}", e))
}

#[tauri::command]
async fn execute_organization(request: ExecuteRequest) -> Result<Statistics, String> {
    let source_dir = expand_tilde(&request.path);

    if !source_dir.exists() {
        return Err(format!(
            "Source directory does not exist: {}",
            source_dir.display()
        ));
    }

    let config_path = resolve_config_path(request.config_path)
        .map_err(|e| format!("Failed to resolve config path: {}", e))?;

    let config = Config::from_file(&config_path)
        .map_err(|e| format!("Failed to load config: {}", e))?;

    let organizer = KondoOrganizer::new(source_dir, config);

    let collision_strategy = match request.collision_strategy.as_str() {
        "skip" => CollisionStrategy::Skip,
        "prompt" => CollisionStrategy::Prompt,
        _ => CollisionStrategy::Rename,
    };

    let options = ExecuteOptions {
        collision_strategy,
        dry_run: request.dry_run,
        recursive: request.recursive,
        backup: request.backup,
        follow_symlinks: request.follow_symlinks,
    };

    let (stats, manifest) = organizer
        .execute(options)
        .map_err(|e| format!("Failed to execute: {}", e))?;

    // Save backup manifest if created
    if let Some(manifest) = manifest {
        if !manifest.operations.is_empty() && !request.dry_run {
            let manifest_path = dirs::data_local_dir()
                .ok_or_else(|| "Could not determine local data directory".to_string())?
                .join("kondo")
                .join(format!(
                    "backup-{}.json",
                    chrono::Utc::now().format("%Y%m%d-%H%M%S")
                ));

            manifest
                .save(&manifest_path)
                .map_err(|e| format!("Failed to save backup manifest: {}", e))?;
        }
    }

    Ok(stats)
}

#[tauri::command]
async fn load_config(path: Option<String>) -> Result<Config, String> {
    let config_path =
        resolve_config_path(path).map_err(|e| format!("Failed to resolve config path: {}", e))?;

    Config::from_file(&config_path).map_err(|e| format!("Failed to load config: {}", e))
}

#[tauri::command]
async fn save_config(request: ConfigSaveRequest) -> Result<(), String> {
    let config_path = expand_tilde(&request.path);

    // Validate config before saving
    request
        .config
        .validate()
        .map_err(|e| format!("Config validation failed: {}", e))?;

    // Create parent directory if it doesn't exist
    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let config_str = request
        .config
        .to_string()
        .map_err(|e| format!("Failed to serialize config: {}", e))?;

    std::fs::write(&config_path, config_str)
        .map_err(|e| format!("Failed to write config: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn get_default_config_path() -> Result<String, String> {
    kondo::get_default_config_path()
        .map(|p| p.display().to_string())
        .ok_or_else(|| "Could not determine default config path".to_string())
}

// ============================================================================
// Main Application
// ============================================================================

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            scan_directory,
            preview_organization,
            execute_organization,
            load_config,
            save_config,
            get_default_config_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
