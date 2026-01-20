use crate::error::Result;
use crate::models::*;
use crate::services::{ConfigService, OrganizerService};

// ============================================================================
// Command Handlers - Thin wrappers for Tauri commands (Command Pattern)
// ============================================================================

/// Scan a directory and categorize files
#[tauri::command]
pub async fn scan_directory(request: DirectoryRequest) -> Result<ScanResponse> {
    OrganizerService::scan(request)
}

/// Preview what operations would be performed
#[tauri::command]
pub async fn preview_organization(request: DirectoryRequest) -> Result<PreviewResponse> {
    OrganizerService::preview(request)
}

/// Execute file organization
#[tauri::command]
pub async fn execute_organization(request: ExecuteRequest) -> Result<ExecuteResponse> {
    OrganizerService::execute(request)
}

/// Load configuration from file
#[tauri::command]
pub async fn load_config(request: ConfigRequest) -> Result<ConfigResponse> {
    ConfigService::load(request)
}

/// Save configuration to file
#[tauri::command]
pub async fn save_config(request: SaveConfigRequest) -> Result<()> {
    ConfigService::save(request)
}

/// Get the default configuration path
#[tauri::command]
pub async fn get_default_config_path() -> Result<String> {
    ConfigService::get_default_path()
}
