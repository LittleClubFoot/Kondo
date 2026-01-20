/**
 * State Management - Observer Pattern
 * Centralized state with subscription-based updates
 */

// ============================================================================
// Observable State
// ============================================================================

class Observable {
    constructor(initialValue) {
        this._value = initialValue;
        this._listeners = new Set();
    }

    get value() {
        return this._value;
    }

    set value(newValue) {
        if (this._value !== newValue) {
            this._value = newValue;
            this._notify();
        }
    }

    subscribe(callback) {
        this._listeners.add(callback);
        return () => this._listeners.delete(callback);
    }

    _notify() {
        this._listeners.forEach(callback => callback(this._value));
    }
}

// ============================================================================
// Application State
// ============================================================================

class AppState {
    constructor() {
        // Core state
        this.currentFolder = new Observable(null);
        this.scanData = new Observable(null);
        this.config = new Observable(null);
        this.status = new Observable({ message: 'Ready', type: 'info' });

        // UI state
        this.activeTab = new Observable('home');
        this.isLoading = new Observable(false);

        // Options
        this.options = new Observable({
            recursive: false,
            follow_symlinks: false,
            collision_strategy: 'rename',
            backup: true
        });
    }

    /**
     * Set current folder and reset related state
     */
    setFolder(path) {
        this.currentFolder.value = path;
        // Reset scan data when folder changes
        if (this.scanData.value) {
            this.scanData.value = null;
        }
    }

    /**
     * Set scan results
     */
    setScanData(data) {
        this.scanData.value = data;
    }

    /**
     * Set configuration
     */
    setConfig(config) {
        this.config.value = config;
    }

    /**
     * Set status message
     */
    setStatus(message, type = 'info') {
        this.status.value = { message, type };
    }

    /**
     * Set loading state
     */
    setLoading(loading) {
        this.isLoading.value = loading;
    }

    /**
     * Update options (merge with existing)
     */
    updateOptions(newOptions) {
        this.options.value = { ...this.options.value, ...newOptions };
    }

    /**
     * Switch active tab
     */
    switchTab(tab) {
        this.activeTab.value = tab;
    }

    /**
     * Reset all state
     */
    reset() {
        this.currentFolder.value = null;
        this.scanData.value = null;
        this.status.value = { message: 'Ready', type: 'info' };
        this.isLoading.value = false;
    }
}

// Singleton instance
export const appState = new AppState();
