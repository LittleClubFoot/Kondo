# Kondo Tauri Desktop App - Getting Started

## Prerequisites

### System Requirements

**Linux:**
```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

**macOS:**
```bash
xcode-select --install
```

**Windows:**
- Install Microsoft Visual Studio C++ Build Tools
- Install WebView2 (usually pre-installed on Windows 11)

### Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Install Node.js
Download from [nodejs.org](https://nodejs.org/) or use a version manager:
```bash
# Using nvm
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.0/install.sh | bash
nvm install --lts
```

### Install Tauri CLI
```bash
cargo install tauri-cli --version ^2.0
```

## Development Setup

### 1. Install Frontend Dependencies
```bash
cd ui
npm install
```

This installs:
- `vite` - Fast development server
- `@tauri-apps/api` - Tauri JavaScript API
- `@tauri-apps/plugin-dialog` - File dialogs
- `@tauri-apps/plugin-fs` - File system access

### 2. Development Mode

Run the development server with hot reload:

```bash
# From project root
cargo tauri dev
```

This will:
1. Start the Vite dev server (http://localhost:5173)
2. Compile the Rust backend
3. Open the Tauri window

**Hot Reload:**
- Frontend changes auto-reload instantly
- Rust changes require restart

### 3. Project Structure

```
kondo/
├── src/                  # Core Rust library
│   ├── lib.rs           # Public API
│   └── main.rs          # CLI binary
├── src-tauri/           # Tauri backend
│   ├── src/
│   │   ├── main.rs      # Tauri entry point
│   │   ├── error.rs     # Error handling
│   │   ├── handlers.rs  # IPC commands
│   │   ├── models.rs    # Data models
│   │   └── services.rs  # Business logic
│   ├── Cargo.toml       # Backend dependencies
│   └── tauri.conf.json  # Tauri configuration
├── ui/                  # Frontend
│   ├── src/
│   │   ├── main.js      # Entry point
│   │   ├── app.js       # Application controller
│   │   ├── api.js       # Backend API
│   │   ├── state.js     # State management
│   │   ├── components.js # UI components
│   │   ├── utils.js     # Utilities
│   │   └── style.css    # Styles
│   ├── index.html       # HTML template
│   ├── package.json     # Frontend dependencies
│   └── vite.config.js   # Build configuration
└── config.toml          # Default Kondo config
```

## Building for Production

### Build Application

```bash
cargo tauri build
```

This creates:
- **Linux**: `.deb`, `.AppImage` in `src-tauri/target/release/bundle/`
- **macOS**: `.dmg`, `.app` in `src-tauri/target/release/bundle/`
- **Windows**: `.msi`, `.exe` in `src-tauri/target/release/bundle/`

### Build Options

**Debug build** (faster, larger):
```bash
cargo tauri build --debug
```

**Specific target**:
```bash
cargo tauri build --target x86_64-unknown-linux-gnu
```

## Usage

### Running the GUI App

1. **Select Folder**
   - Click "Select Folder" or drag & drop
   - Choose the directory to organize

2. **Scan Files**
   - Click "Scan Folder"
   - Review categorized files

3. **Preview Changes**
   - Switch to "Organize" tab
   - Click "Dry Run" to preview

4. **Organize Files**
   - Click "Organize Files"
   - Confirm the operation
   - Files are moved with backup created

### Configuration

Edit categories in the Config tab or manually edit:
```bash
~/.config/kondo/config.toml
```

Example:
```toml
[[categories]]
name = "Images"
extensions = ["jpg", "png", "gif"]
destination = "~/Pictures"

[[categories]]
name = "Documents"
extensions = ["pdf", "docx", "txt"]
destination = "~/Documents"
```

## Development Workflow

### Adding a New Feature

1. **Backend (Rust)**
   - Add function to `src-tauri/src/services.rs`
   - Create handler in `src-tauri/src/handlers.rs`
   - Register in `src-tauri/src/main.rs`

2. **Frontend (JavaScript)**
   - Add API method to `ui/src/api.js`
   - Update app logic in `ui/src/app.js`
   - Add UI components as needed

### Example: Adding a New Command

**Backend (services.rs):**
```rust
pub struct HistoryService;

impl HistoryService {
    pub fn get_recent_operations() -> Result<Vec<BackupManifest>> {
        // Implementation
    }
}
```

**Backend (handlers.rs):**
```rust
#[tauri::command]
pub async fn get_history() -> Result<Vec<BackupManifest>> {
    HistoryService::get_recent_operations()
}
```

**Backend (main.rs):**
```rust
.invoke_handler(tauri::generate_handler![
    // ... existing handlers
    get_history,
])
```

**Frontend (api.js):**
```javascript
export const HistoryAPI = {
    async getRecent() {
        return await invoke('get_history');
    }
};
```

**Frontend (app.js):**
```javascript
async loadHistory() {
    const history = await HistoryAPI.getRecent();
    this.displayHistory(history);
}
```

## Debugging

### Frontend Console

Open DevTools in development:
- **macOS**: `Cmd+Option+I`
- **Windows/Linux**: `Ctrl+Shift+I`

Or programmatically:
```javascript
console.log('Debug info:', data);
```

### Backend Logging

Add to `src-tauri/src/main.rs`:
```rust
println!("Debug: {:?}", some_value);
eprintln!("Error: {:?}", error);
```

View logs:
```bash
# Run with cargo directly
cargo run --manifest-path=src-tauri/Cargo.toml
```

### Common Issues

**Build fails with "webkit2gtk not found"**
```bash
sudo apt install libwebkit2gtk-4.1-dev
```

**Frontend not loading**
- Check Vite is running on port 5173
- Check `tauri.conf.json` devUrl setting

**Commands not working**
- Verify handler is registered in `main.rs`
- Check request/response types match
- View browser console for errors

## Performance Tips

1. **Lazy load heavy operations**
   ```javascript
   // Good
   async loadWhenNeeded() {
       const data = await HeavyAPI.load();
   }

   // Bad - loads immediately
   const data = HeavyAPI.loadSync();
   ```

2. **Debounce user input**
   ```javascript
   import { Async } from './utils.js';

   const search = Async.debounce(async (query) => {
       await searchFiles(query);
   }, 300);
   ```

3. **Use dry-run before heavy operations**
   ```javascript
   // Preview first
   const preview = await organize({ dry_run: true });
   // Then execute
   if (confirm('Proceed?')) {
       await organize({ dry_run: false });
   }
   ```

## Security Best Practices

1. **Validate all paths**
   ```rust
   // Backend validates automatically
   fn validate_directory(path: &str) -> Result<PathBuf> {
       let dir = expand_tilde(path);
       if !dir.exists() {
           return Err(AppError::path_error("Directory does not exist"));
       }
       Ok(dir)
   }
   ```

2. **Use CSP headers**
   Already configured in `tauri.conf.json`

3. **Limit file system access**
   ```json
   "plugins": {
     "fs": {
       "scope": ["$HOME/**", "$CONFIG/**"]
     }
   }
   ```

## Testing

### Run Tests
```bash
# Rust tests
cargo test

# Rust tests in Tauri
cargo test --manifest-path=src-tauri/Cargo.toml

# Frontend tests (when added)
cd ui && npm test
```

### Manual Testing Checklist

- [ ] Select folder dialog works
- [ ] Scan shows correct categories
- [ ] Dry run previews changes
- [ ] Organize moves files correctly
- [ ] Collision handling works
- [ ] Config loads/saves
- [ ] Error messages are helpful
- [ ] UI responsive

## Distribution

### Create Installer

```bash
cargo tauri build
```

Installers are in `src-tauri/target/release/bundle/`

### Code Signing (Production)

**macOS:**
```bash
export APPLE_CERTIFICATE=...
export APPLE_ID=...
cargo tauri build
```

**Windows:**
```bash
# Configure in tauri.conf.json
```

## Contributing

See main [README.md](README.md) for contribution guidelines.

## Resources

- [Tauri Documentation](https://tauri.app/)
- [Tauri API Reference](https://tauri.app/reference/javascript/api/)
- [Rust Documentation](https://doc.rust-lang.org/)
- [Kondo Architecture](ARCHITECTURE.md)
