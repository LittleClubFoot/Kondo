/**
 * Application Controller - Coordinates all modules
 * Implements Controller pattern from MVC
 */

import { DirectoryAPI, ConfigAPI } from './api.js';
import { appState } from './state.js';
import { ComponentFactory, ModalDialog } from './components.js';
import { DOM, Format, Validate } from './utils.js';

// ============================================================================
// Application Controller
// ============================================================================

export class KondoApp {
    constructor() {
        this.state = appState;
        this.setupStateSubscriptions();
    }

    /**
     * Initialize the application
     */
    init() {
        this.setupTabs();
        this.setupHomeTab();
        this.setupOrganizeTab();
        this.setupConfigTab();
        this.loadInitialConfig();
    }

    // ========================================================================
    // State Subscriptions
    // ========================================================================

    setupStateSubscriptions() {
        // Subscribe to status changes
        this.state.status.subscribe(({ message, type }) => {
            this.updateStatusBar(message, type);
        });

        // Subscribe to loading state
        this.state.isLoading.subscribe(loading => {
            this.toggleLoading(loading);
        });

        // Subscribe to active tab changes
        this.state.activeTab.subscribe(tab => {
            this.switchToTab(tab);
        });
    }

    // ========================================================================
    // Tab Management
    // ========================================================================

    setupTabs() {
        const tabs = DOM.getAll('.tab');

        tabs.forEach(tab => {
            tab.addEventListener('click', () => {
                const tabName = tab.dataset.tab;
                this.state.switchTab(tabName);
            });
        });
    }

    switchToTab(tabName) {
        // Update tab buttons
        DOM.getAll('.tab').forEach(tab => {
            DOM.toggleClass(tab, 'active', tab.dataset.tab === tabName);
        });

        // Update tab content
        DOM.getAll('.tab-content').forEach(content => {
            DOM.toggleClass(content, 'active', content.id === `${tabName}-tab`);
        });
    }

    // ========================================================================
    // Home Tab
    // ========================================================================

    setupHomeTab() {
        DOM.get('select-folder-btn')?.addEventListener('click', () => {
            this.selectFolder();
        });

        DOM.get('drop-zone')?.addEventListener('click', () => {
            this.selectFolder();
        });

        DOM.get('scan-btn')?.addEventListener('click', () => {
            this.scanFolder();
        });
    }

    async selectFolder() {
        try {
            const selected = await DirectoryAPI.select();

            if (selected) {
                this.state.setFolder(selected);
                DOM.setText(DOM.get('folder-path'), Format.path(selected));
                DOM.toggle(DOM.get('selected-folder'), true);
                DOM.toggle(DOM.get('drop-zone'), false);
                this.state.setStatus(`Selected: ${selected}`);
            }
        } catch (error) {
            this.handleError(error, 'selecting folder');
        }
    }

    async scanFolder() {
        const folder = this.state.currentFolder.value;

        if (!Validate.isValidPath(folder)) {
            this.state.setStatus('Please select a folder first', 'error');
            return;
        }

        try {
            this.state.setLoading(true);
            this.state.setStatus('Scanning folder...');

            const result = await DirectoryAPI.scan(folder, {
                recursive: this.state.options.value.recursive
            });

            this.state.setScanData(result);
            this.state.setStatus(`Found ${Format.number(result.total_files)} files`);

            // Switch to organize tab
            this.state.switchTab('organize');
            this.displayScanResults(result);
        } catch (error) {
            this.handleError(error, 'scanning folder');
        } finally {
            this.state.setLoading(false);
        }
    }

    // ========================================================================
    // Organize Tab
    // ========================================================================

    setupOrganizeTab() {
        DOM.get('change-folder-btn')?.addEventListener('click', () => {
            this.state.switchTab('home');
        });

        DOM.get('dry-run-btn')?.addEventListener('click', () => {
            this.runDryRun();
        });

        DOM.get('organize-btn')?.addEventListener('click', () => {
            this.organizeFiles();
        });
    }

    displayScanResults(result) {
        const folder = this.state.currentFolder.value;

        // Update header
        DOM.setText(DOM.get('org-folder-path'), Format.path(folder));
        DOM.setText(DOM.get('total-files'), Format.number(result.total_files));

        // Clear previous results
        const categoriesContainer = DOM.get('categories');
        DOM.clear(categoriesContainer);

        // Display categories
        for (const [categoryName, files] of Object.entries(result.by_category)) {
            if (files.length === 0) continue;

            const card = ComponentFactory.createCategoryCard(categoryName, files, {
                displayLimit: 10
            });
            categoriesContainer.appendChild(card);
        }

        // Display no category files
        if (result.no_category.length > 0) {
            const card = ComponentFactory.createCategoryCard(
                'No Category',
                result.no_category,
                {
                    highlight: 'warning',
                    displayLimit: 10
                }
            );
            categoriesContainer.appendChild(card);
        }

        DOM.toggle(DOM.get('scan-results'), true);
        DOM.toggle(DOM.get('empty-state'), false);
    }

    async runDryRun() {
        const folder = this.state.currentFolder.value;

        if (!Validate.isValidPath(folder)) {
            this.state.setStatus('Please scan a folder first', 'error');
            return;
        }

        try {
            this.state.setLoading(true);
            this.state.setStatus('Running dry run...');

            const result = await DirectoryAPI.organize(folder, {
                dry_run: true,
                ...this.state.options.value
            });

            this.state.setStatus(
                `Dry run complete. Would move ${result.moved} files, skip ${result.skipped}`
            );

            this.showResultsModal('Dry Run Results', result, true);
        } catch (error) {
            this.handleError(error, 'running dry run');
        } finally {
            this.state.setLoading(false);
        }
    }

    async organizeFiles() {
        const folder = this.state.currentFolder.value;

        if (!Validate.isValidPath(folder)) {
            this.state.setStatus('Please scan a folder first', 'error');
            return;
        }

        // Confirmation dialog
        const confirmed = await this.showConfirmDialog(
            'Organize Files',
            'Are you sure you want to organize these files? This will move them to their destination folders.'
        );

        if (!confirmed) return;

        try {
            this.state.setLoading(true);
            this.state.setStatus('Organizing files...');

            const result = await DirectoryAPI.organize(folder, {
                dry_run: false,
                ...this.state.options.value
            });

            this.state.setStatus(`Organization complete! Moved ${result.moved} files`, 'success');
            this.showResultsModal('Organization Complete', result, false);

            // Rescan folder
            await this.scanFolder();
        } catch (error) {
            this.handleError(error, 'organizing files');
        } finally {
            this.state.setLoading(false);
        }
    }

    // ========================================================================
    // Config Tab
    // ========================================================================

    setupConfigTab() {
        DOM.get('load-config-btn')?.addEventListener('click', () => {
            this.loadConfig();
        });

        DOM.get('save-config-btn')?.addEventListener('click', () => {
            this.showAlert('Config editing coming soon!', 'info');
        });
    }

    async loadInitialConfig() {
        await this.loadConfig();
    }

    async loadConfig() {
        try {
            this.state.setLoading(true);
            this.state.setStatus('Loading configuration...');

            const config = await ConfigAPI.load();
            this.state.setConfig(config);

            this.displayConfig(config);
            this.state.setStatus('Configuration loaded', 'success');
        } catch (error) {
            this.handleError(error, 'loading configuration');
            const content = DOM.get('config-content');
            if (content) {
                content.innerHTML = `<p style="color: var(--error)">Error loading config: ${error.message}</p>`;
            }
        } finally {
            this.state.setLoading(false);
        }
    }

    displayConfig(config) {
        const content = DOM.get('config-content');
        if (!content) return;

        DOM.clear(content);

        const pre = document.createElement('pre');
        pre.style.fontFamily = "'Courier New', monospace";
        pre.style.fontSize = '0.9rem';
        pre.style.lineHeight = '1.6';
        pre.style.whiteSpace = 'pre-wrap';
        pre.textContent = JSON.stringify(config, null, 2);

        content.appendChild(pre);
    }

    // ========================================================================
    // UI Helpers
    // ========================================================================

    updateStatusBar(message, type) {
        const statusBar = DOM.get('status-bar');
        if (!statusBar) return;

        statusBar.textContent = message;
        statusBar.style.color =
            type === 'error'
                ? 'var(--error)'
                : type === 'success'
                ? 'var(--success)'
                : 'var(--text-secondary)';
    }

    toggleLoading(loading) {
        // Could add a loading overlay here
        console.log('Loading:', loading);
    }

    showResultsModal(title, results, isDryRun) {
        const content = document.createElement('div');
        content.style.padding = '1rem';

        const stats = [
            { label: 'Total Files', value: results.total_files },
            { label: isDryRun ? 'Would Move' : 'Moved', value: results.moved },
            { label: isDryRun ? 'Would Skip' : 'Skipped', value: results.skipped },
            { label: 'No Category', value: results.no_category }
        ];

        stats.forEach(({ label, value }) => {
            const stat = ComponentFactory.createStatCard(label, value);
            content.appendChild(stat);
        });

        const modal = new ModalDialog(title, content);
        modal.addButton('OK', () => {}, 'btn-primary');
        modal.show();
    }

    showConfirmDialog(title, message) {
        return new Promise(resolve => {
            const modal = new ModalDialog(title, message);
            modal.addButton('Cancel', () => resolve(false), 'btn-secondary');
            modal.addButton('Confirm', () => resolve(true), 'btn-primary');
            modal.show();
        });
    }

    showAlert(message, type = 'info') {
        const alert = ComponentFactory.createAlert(message, type);
        document.body.appendChild(alert);

        setTimeout(() => {
            alert.style.opacity = '0';
            setTimeout(() => alert.remove(), 300);
        }, 3000);
    }

    handleError(error, context) {
        console.error(`Error ${context}:`, error);
        const message = error.message || String(error);
        this.state.setStatus(`Error ${context}: ${message}`, 'error');
        this.showAlert(`Failed ${context}: ${message}`, 'error');
    }
}
