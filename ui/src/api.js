/**
 * API Layer - Repository Pattern
 * Abstracts all Tauri backend communication
 */

import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

// ============================================================================
// Error Handling
// ============================================================================

class ApiError extends Error {
    constructor(message, original) {
        super(message);
        this.name = 'ApiError';
        this.original = original;
    }
}

function handleError(error, context) {
    console.error(`API Error [${context}]:`, error);
    const message = error?.message || error || 'Unknown error occurred';
    throw new ApiError(`${context}: ${message}`, error);
}

// ============================================================================
// Directory Operations
// ============================================================================

export const DirectoryAPI = {
    /**
     * Open directory selection dialog
     */
    async select() {
        try {
            return await open({
                directory: true,
                multiple: false,
                title: 'Select Folder to Organize'
            });
        } catch (error) {
            handleError(error, 'Failed to select directory');
        }
    },

    /**
     * Scan directory and categorize files
     */
    async scan(path, options = {}) {
        try {
            return await invoke('scan_directory', {
                request: {
                    path,
                    config_path: options.config_path || null,
                    recursive: options.recursive || false,
                    follow_symlinks: options.follow_symlinks || false
                }
            });
        } catch (error) {
            handleError(error, 'Failed to scan directory');
        }
    },

    /**
     * Preview organization operations
     */
    async preview(path, options = {}) {
        try {
            return await invoke('preview_organization', {
                request: {
                    path,
                    config_path: options.config_path || null,
                    recursive: options.recursive || false,
                    follow_symlinks: options.follow_symlinks || false
                }
            });
        } catch (error) {
            handleError(error, 'Failed to preview organization');
        }
    },

    /**
     * Execute file organization
     */
    async organize(path, options = {}) {
        try {
            return await invoke('execute_organization', {
                request: {
                    path,
                    config_path: options.config_path || null,
                    collision_strategy: options.collision_strategy || 'rename',
                    dry_run: options.dry_run || false,
                    recursive: options.recursive || false,
                    backup: options.backup !== undefined ? options.backup : true,
                    follow_symlinks: options.follow_symlinks || false
                }
            });
        } catch (error) {
            handleError(error, 'Failed to organize files');
        }
    }
};

// ============================================================================
// Configuration Operations
// ============================================================================

export const ConfigAPI = {
    /**
     * Load configuration
     */
    async load(path = null) {
        try {
            return await invoke('load_config', {
                request: { path }
            });
        } catch (error) {
            handleError(error, 'Failed to load configuration');
        }
    },

    /**
     * Save configuration
     */
    async save(config, path) {
        try {
            return await invoke('save_config', {
                request: { config, path }
            });
        } catch (error) {
            handleError(error, 'Failed to save configuration');
        }
    },

    /**
     * Get default config path
     */
    async getDefaultPath() {
        try {
            return await invoke('get_default_config_path');
        } catch (error) {
            handleError(error, 'Failed to get default config path');
        }
    }
};
