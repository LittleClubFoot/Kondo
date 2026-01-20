/**
 * UI Components - Factory Pattern
 * Reusable component builders following DRY principles
 */

// ============================================================================
// DOM Utilities
// ============================================================================

function createElement(tag, className = '', content = '') {
    const element = document.createElement(tag);
    if (className) element.className = className;
    if (content) element.textContent = content;
    return element;
}

function createIcon(path) {
    const svg = document.createElementNS('http://www.w3.org/2000/svg', 'svg');
    svg.setAttribute('width', '20');
    svg.setAttribute('height', '20');
    svg.setAttribute('viewBox', '0 0 24 24');
    svg.setAttribute('fill', 'none');
    svg.setAttribute('stroke', 'currentColor');
    svg.setAttribute('stroke-width', '2');

    const pathEl = document.createElementNS('http://www.w3.org/2000/svg', 'path');
    pathEl.setAttribute('d', path);
    svg.appendChild(pathEl);

    return svg;
}

// ============================================================================
// Component Builders
// ============================================================================

export const ComponentFactory = {
    /**
     * Create a category card for scan results
     */
    createCategoryCard(categoryName, files, options = {}) {
        const card = createElement('div', 'category-card');

        if (options.highlight) {
            card.style.borderColor = `var(--${options.highlight})`;
        }

        // Header
        const header = createElement('div', 'category-header');
        const name = createElement('div', 'category-name', categoryName);
        const count = createElement('div', 'file-count', `${files.length} files`);

        if (options.highlight) {
            count.style.backgroundColor = `var(--${options.highlight})`;
        }

        header.appendChild(name);
        header.appendChild(count);

        // File list
        const fileList = createElement('div', 'file-list');
        const displayLimit = options.displayLimit || 10;

        files.slice(0, displayLimit).forEach(file => {
            const fileItem = createElement('div', 'file-item', file.name);
            fileList.appendChild(fileItem);
        });

        if (files.length > displayLimit) {
            const more = createElement(
                'div',
                'file-item',
                `... and ${files.length - displayLimit} more`
            );
            more.style.fontWeight = 'bold';
            fileList.appendChild(more);
        }

        card.appendChild(header);
        if (options.destination) {
            const dest = createElement(
                'div',
                'category-destination',
                `→ ${options.destination}`
            );
            card.appendChild(dest);
        }
        card.appendChild(fileList);

        return card;
    },

    /**
     * Create an empty state message
     */
    createEmptyState(message, icon = null) {
        const container = createElement('div', 'empty-state');

        if (icon) {
            container.appendChild(icon);
        }

        const text = createElement('p', '', message);
        container.appendChild(text);

        return container;
    },

    /**
     * Create a loading spinner
     */
    createLoader(message = 'Loading...') {
        const container = createElement('div', 'loader-container');
        const spinner = createElement('div', 'spinner');
        const text = createElement('p', '', message);

        container.appendChild(spinner);
        container.appendChild(text);

        return container;
    },

    /**
     * Create an alert/notification
     */
    createAlert(message, type = 'info') {
        const alert = createElement('div', `alert alert-${type}`);

        const icon = type === 'error' ? '⚠️' : type === 'success' ? '✓' : 'ℹ️';
        const iconSpan = createElement('span', 'alert-icon', icon);
        const messageSpan = createElement('span', 'alert-message', message);

        alert.appendChild(iconSpan);
        alert.appendChild(messageSpan);

        return alert;
    },

    /**
     * Create a statistic card
     */
    createStatCard(label, value, icon = null) {
        const card = createElement('div', 'stat-card');

        if (icon) {
            card.appendChild(icon);
        }

        const valueEl = createElement('div', 'stat-value', String(value));
        const labelEl = createElement('div', 'stat-label', label);

        card.appendChild(valueEl);
        card.appendChild(labelEl);

        return card;
    },

    /**
     * Create a button with icon
     */
    createButton(text, className = 'btn-primary', icon = null) {
        const button = createElement('button', `btn ${className}`);

        if (icon) {
            button.appendChild(icon);
        }

        const textNode = document.createTextNode(text);
        button.appendChild(textNode);

        return button;
    }
};

// ============================================================================
// Modal Dialog Builder
// ============================================================================

export class ModalDialog {
    constructor(title, content) {
        this.overlay = createElement('div', 'modal-overlay');
        this.dialog = createElement('div', 'modal-dialog');

        // Header
        const header = createElement('div', 'modal-header');
        const titleEl = createElement('h3', '', title);
        const closeBtn = createElement('button', 'modal-close', '×');
        closeBtn.onclick = () => this.close();

        header.appendChild(titleEl);
        header.appendChild(closeBtn);

        // Content
        const contentEl = createElement('div', 'modal-content');
        if (typeof content === 'string') {
            contentEl.textContent = content;
        } else {
            contentEl.appendChild(content);
        }

        // Footer
        this.footer = createElement('div', 'modal-footer');

        this.dialog.appendChild(header);
        this.dialog.appendChild(contentEl);
        this.dialog.appendChild(this.footer);
        this.overlay.appendChild(this.dialog);
    }

    addButton(text, onClick, className = 'btn-secondary') {
        const btn = ComponentFactory.createButton(text, className);
        btn.onclick = () => {
            onClick();
            this.close();
        };
        this.footer.appendChild(btn);
        return this;
    }

    show() {
        document.body.appendChild(this.overlay);
        // Trigger reflow for animation
        this.overlay.offsetHeight;
        this.overlay.classList.add('active');
    }

    close() {
        this.overlay.classList.remove('active');
        setTimeout(() => {
            this.overlay.remove();
        }, 200);
    }
}
