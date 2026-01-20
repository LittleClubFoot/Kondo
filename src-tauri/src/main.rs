// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod error;
mod handlers;
mod models;
mod services;

use error::Result;
use handlers::*;
use tauri::Manager;

// ============================================================================
// Main Application
// ============================================================================

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // Initialize application state if needed
            Ok(())
        })
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
