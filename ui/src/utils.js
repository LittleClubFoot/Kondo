/**
 * Utility Functions - Helper methods following DRY principles
 */

// ============================================================================
// DOM Utilities
// ============================================================================

export const DOM = {
    /**
     * Get element by ID with optional type checking
     */
    get(id) {
        const element = document.getElementById(id);
        if (!element) {
            console.warn(`Element with id '${id}' not found`);
        }
        return element;
    },

    /**
     * Get all elements matching selector
     */
    getAll(selector) {
        return document.querySelectorAll(selector);
    },

    /**
     * Show/hide element
     */
    toggle(element, show) {
        if (!element) return;
        element.style.display = show ? '' : 'none';
    },

    /**
     * Add/remove class
     */
    toggleClass(element, className, add) {
        if (!element) return;
        if (add) {
            element.classList.add(className);
        } else {
            element.classList.remove(className);
        }
    },

    /**
     * Clear all children of an element
     */
    clear(element) {
        if (!element) return;
        while (element.firstChild) {
            element.removeChild(element.firstChild);
        }
    },

    /**
     * Set text content safely
     */
    setText(element, text) {
        if (!element) return;
        element.textContent = text;
    }
};

// ============================================================================
// Formatting Utilities
// ============================================================================

export const Format = {
    /**
     * Format file size
     */
    fileSize(bytes) {
        if (bytes === 0) return '0 B';

        const k = 1024;
        const sizes = ['B', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));

        return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
    },

    /**
     * Format number with separators
     */
    number(num) {
        return num.toLocaleString();
    },

    /**
     * Truncate string with ellipsis
     */
    truncate(str, maxLength) {
        if (str.length <= maxLength) return str;
        return str.substring(0, maxLength - 3) + '...';
    },

    /**
     * Format path for display
     */
    path(path, maxLength = 60) {
        if (path.length <= maxLength) return path;

        const parts = path.split('/');
        if (parts.length <= 2) return this.truncate(path, maxLength);

        const fileName = parts.pop();
        const firstPart = parts.shift();

        return `${firstPart}/.../${fileName}`;
    }
};

// ============================================================================
// Async Utilities
// ============================================================================

export const Async = {
    /**
     * Debounce function calls
     */
    debounce(func, delay) {
        let timeoutId;
        return function (...args) {
            clearTimeout(timeoutId);
            timeoutId = setTimeout(() => func.apply(this, args), delay);
        };
    },

    /**
     * Throttle function calls
     */
    throttle(func, limit) {
        let inThrottle;
        return function (...args) {
            if (!inThrottle) {
                func.apply(this, args);
                inThrottle = true;
                setTimeout(() => (inThrottle = false), limit);
            }
        };
    },

    /**
     * Retry async operation
     */
    async retry(fn, maxAttempts = 3, delay = 1000) {
        for (let attempt = 1; attempt <= maxAttempts; attempt++) {
            try {
                return await fn();
            } catch (error) {
                if (attempt === maxAttempts) throw error;
                await this.sleep(delay);
            }
        }
    },

    /**
     * Sleep/delay
     */
    sleep(ms) {
        return new Promise(resolve => setTimeout(resolve, ms));
    }
};

// ============================================================================
// Validation Utilities
// ============================================================================

export const Validate = {
    /**
     * Check if path is valid
     */
    isValidPath(path) {
        return path && typeof path === 'string' && path.trim().length > 0;
    },

    /**
     * Check if object is empty
     */
    isEmpty(obj) {
        return !obj || Object.keys(obj).length === 0;
    },

    /**
     * Check if value is defined and not null
     */
    isDefined(value) {
        return value !== undefined && value !== null;
    }
};
