import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

// State
let currentFolder = null;
let scanData = null;
let currentConfig = null;

// ============================================================================
// Tab Management
// ============================================================================

function initTabs() {
    const tabs = document.querySelectorAll('.tab');
    const tabContents = document.querySelectorAll('.tab-content');

    tabs.forEach(tab => {
        tab.addEventListener('click', () => {
            const tabName = tab.dataset.tab;

            // Update active tab
            tabs.forEach(t => t.classList.remove('active'));
            tab.classList.add('active');

            // Update active content
            tabContents.forEach(content => {
                content.classList.remove('active');
            });
            document.getElementById(`${tabName}-tab`).classList.add('active');
        });
    });
}

// ============================================================================
// Status Bar
// ============================================================================

function setStatus(message, type = 'info') {
    const statusBar = document.getElementById('status-bar');
    statusBar.textContent = message;
    statusBar.style.color = type === 'error' ? 'var(--error)' : 'var(--text-secondary)';
}

// ============================================================================
// Home Tab
// ============================================================================

async function selectFolder() {
    try {
        const selected = await open({
            directory: true,
            multiple: false,
            title: 'Select Folder to Organize'
        });

        if (selected) {
            currentFolder = selected;
            document.getElementById('folder-path').textContent = selected;
            document.getElementById('selected-folder').style.display = 'block';
            document.getElementById('drop-zone').style.display = 'none';
            setStatus(`Selected: ${selected}`);
        }
    } catch (error) {
        console.error('Error selecting folder:', error);
        setStatus(`Error: ${error}`, 'error');
    }
}

async function scanFolder() {
    if (!currentFolder) {
        setStatus('Please select a folder first', 'error');
        return;
    }

    try {
        setStatus('Scanning folder...');

        const result = await invoke('scan_directory', {
            request: {
                path: currentFolder,
                config_path: null,
                recursive: false,
                follow_symlinks: false
            }
        });

        scanData = result;
        setStatus(`Found ${result.total_files} files`);

        // Switch to organize tab and display results
        document.querySelector('[data-tab="organize"]').click();
        displayScanResults(result);
    } catch (error) {
        console.error('Error scanning folder:', error);
        setStatus(`Error: ${error}`, 'error');
    }
}

function initHomeTab() {
    document.getElementById('select-folder-btn').addEventListener('click', selectFolder);
    document.getElementById('drop-zone').addEventListener('click', selectFolder);
    document.getElementById('scan-btn').addEventListener('click', scanFolder);
}

// ============================================================================
// Organize Tab
// ============================================================================

function displayScanResults(result) {
    const orgFolderPath = document.getElementById('org-folder-path');
    const totalFiles = document.getElementById('total-files');
    const categories = document.getElementById('categories');
    const scanResults = document.getElementById('scan-results');
    const emptyState = document.getElementById('empty-state');

    orgFolderPath.textContent = currentFolder;
    totalFiles.textContent = result.total_files;

    // Clear previous results
    categories.innerHTML = '';

    // Display categories
    for (const [categoryName, files] of Object.entries(result.by_category)) {
        if (files.length === 0) continue;

        const card = document.createElement('div');
        card.className = 'category-card';

        const header = document.createElement('div');
        header.className = 'category-header';

        const name = document.createElement('div');
        name.className = 'category-name';
        name.textContent = categoryName;

        const count = document.createElement('div');
        count.className = 'file-count';
        count.textContent = `${files.length} files`;

        header.appendChild(name);
        header.appendChild(count);

        const fileList = document.createElement('div');
        fileList.className = 'file-list';

        // Show first 10 files
        files.slice(0, 10).forEach(file => {
            const fileItem = document.createElement('div');
            fileItem.className = 'file-item';
            fileItem.textContent = file.name;
            fileList.appendChild(fileItem);
        });

        if (files.length > 10) {
            const more = document.createElement('div');
            more.className = 'file-item';
            more.textContent = `... and ${files.length - 10} more`;
            fileList.appendChild(more);
        }

        card.appendChild(header);
        card.appendChild(fileList);
        categories.appendChild(card);
    }

    // Display no category files
    if (result.no_category.length > 0) {
        const card = document.createElement('div');
        card.className = 'category-card';
        card.style.borderColor = 'var(--warning)';

        const header = document.createElement('div');
        header.className = 'category-header';

        const name = document.createElement('div');
        name.className = 'category-name';
        name.textContent = 'No Category';

        const count = document.createElement('div');
        count.className = 'file-count';
        count.style.backgroundColor = 'var(--warning)';
        count.textContent = `${result.no_category.length} files`;

        header.appendChild(name);
        header.appendChild(count);

        card.appendChild(header);
        categories.appendChild(card);
    }

    scanResults.style.display = 'block';
    emptyState.style.display = 'none';
}

async function runDryRun() {
    if (!currentFolder) {
        setStatus('Please scan a folder first', 'error');
        return;
    }

    try {
        setStatus('Running dry run...');

        const result = await invoke('execute_organization', {
            request: {
                path: currentFolder,
                config_path: null,
                collision_strategy: 'rename',
                dry_run: true,
                recursive: false,
                backup: false,
                follow_symlinks: false
            }
        });

        setStatus(`Dry run complete. Would move ${result.moved} files, skip ${result.skipped}`);
        alert(`Dry Run Results:\n\nTotal: ${result.total_files}\nWould move: ${result.moved}\nWould skip: ${result.skipped}\nNo category: ${result.no_category}`);
    } catch (error) {
        console.error('Error running dry run:', error);
        setStatus(`Error: ${error}`, 'error');
    }
}

async function organizeFiles() {
    if (!currentFolder) {
        setStatus('Please scan a folder first', 'error');
        return;
    }

    if (!confirm('Are you sure you want to organize these files? This will move them to their destination folders.')) {
        return;
    }

    try {
        setStatus('Organizing files...');

        const result = await invoke('execute_organization', {
            request: {
                path: currentFolder,
                config_path: null,
                collision_strategy: 'rename',
                dry_run: false,
                recursive: false,
                backup: true,
                follow_symlinks: false
            }
        });

        setStatus(`Organization complete! Moved ${result.moved} files`);
        alert(`Organization Complete!\n\nTotal: ${result.total_files}\nMoved: ${result.moved}\nSkipped: ${result.skipped}\nNo category: ${result.no_category}`);

        // Rescan the folder
        scanFolder();
    } catch (error) {
        console.error('Error organizing files:', error);
        setStatus(`Error: ${error}`, 'error');
    }
}

function initOrganizeTab() {
    document.getElementById('change-folder-btn').addEventListener('click', () => {
        document.querySelector('[data-tab="home"]').click();
    });

    document.getElementById('dry-run-btn').addEventListener('click', runDryRun);
    document.getElementById('organize-btn').addEventListener('click', organizeFiles);
}

// ============================================================================
// Config Tab
// ============================================================================

async function loadConfig() {
    try {
        setStatus('Loading configuration...');

        const config = await invoke('load_config', { path: null });
        currentConfig = config;

        displayConfig(config);
        setStatus('Configuration loaded');
    } catch (error) {
        console.error('Error loading config:', error);
        setStatus(`Error: ${error}`, 'error');
        document.getElementById('config-content').innerHTML = `<p style="color: var(--error)">Error loading config: ${error}</p>`;
    }
}

function displayConfig(config) {
    const content = document.getElementById('config-content');
    content.innerHTML = '';

    const pre = document.createElement('pre');
    pre.style.fontFamily = "'Courier New', monospace";
    pre.style.fontSize = '0.9rem';
    pre.style.lineHeight = '1.6';
    pre.style.whiteSpace = 'pre-wrap';
    pre.textContent = JSON.stringify(config, null, 2);

    content.appendChild(pre);
}

function initConfigTab() {
    document.getElementById('load-config-btn').addEventListener('click', loadConfig);
    document.getElementById('save-config-btn').addEventListener('click', async () => {
        alert('Config editing coming soon!');
    });

    // Load config initially
    loadConfig();
}

// ============================================================================
// Initialization
// ============================================================================

document.addEventListener('DOMContentLoaded', () => {
    initTabs();
    initHomeTab();
    initOrganizeTab();
    initConfigTab();
    setStatus('Ready');
});
