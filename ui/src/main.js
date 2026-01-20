/**
 * Application Entry Point
 * Bootstrap and initialize the Kondo application
 */

import { KondoApp } from './app.js';

// ============================================================================
// Application Initialization
// ============================================================================

// Initialize app when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    const app = new KondoApp();
    app.init();

    // Expose app instance for debugging (remove in production)
    if (import.meta.env.DEV) {
        window.kondo = app;
    }
});

// Handle unhandled errors
window.addEventListener('error', (event) => {
    console.error('Unhandled error:', event.error);
});

// Handle unhandled promise rejections
window.addEventListener('unhandledrejection', (event) => {
    console.error('Unhandled promise rejection:', event.reason);
});
