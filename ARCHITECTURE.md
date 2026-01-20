# Kondo Tauri Architecture Documentation

## Overview

Kondo's Tauri desktop application follows enterprise-grade architectural patterns for maintainability, testability, and scalability.

## Architecture Patterns

### Backend (Rust)

#### 1. Layered Architecture
```
┌─────────────────────────────────┐
│   Tauri Handlers (handlers.rs)  │  ← IPC Commands (thin wrappers)
├─────────────────────────────────┤
│   Services (services.rs)         │  ← Business Logic (Repository Pattern)
├─────────────────────────────────┤
│   Kondo Library (lib.rs)        │  ← Core Domain Logic
└─────────────────────────────────┘
```

#### 2. Module Structure

**main.rs** - Application entry point
- Minimal bootstrapping code
- Plugin registration
- Command handler registration

**error.rs** - Centralized error handling
- Unified `AppError` enum with thiserror
- Automatic conversions from common error types
- Serializable for Tauri IPC

**models.rs** - Data Transfer Objects (DTOs)
- Request/Response types for IPC
- Type conversions between Tauri and Kondo library
- Domain model definitions

**services.rs** - Business logic layer (Repository Pattern)
- `OrganizerService` - File organization operations
- `ConfigService` - Configuration management
- Path validation and sanitization
- Manifest backup handling

**handlers.rs** - Tauri command handlers (Command Pattern)
- Thin wrappers that delegate to services
- Async command implementations
- Direct mapping to frontend API calls

#### 3. Design Patterns

**Repository Pattern**
```rust
pub struct OrganizerService;

impl OrganizerService {
    fn create_organizer(...) -> Result<KondoOrganizer>
    pub fn scan(...) -> Result<ScanResponse>
    pub fn preview(...) -> Result<PreviewResponse>
    pub fn execute(...) -> Result<ExecuteResponse>
}
```

**Command Pattern**
```rust
#[tauri::command]
pub async fn scan_directory(request: DirectoryRequest) -> Result<ScanResponse> {
    OrganizerService::scan(request)
}
```

**Factory Pattern** (Error creation)
```rust
impl AppError {
    pub fn config_error(msg: impl Into<String>) -> Self
    pub fn validation_error(msg: impl Into<String>) -> Self
}
```

### Frontend (JavaScript)

#### 1. Module Architecture (ES6 Modules)

```
┌──────────────────────────────────┐
│   main.js (Entry Point)          │
└──────────────┬───────────────────┘
               │
       ┌───────┴────────┐
       │   app.js       │  ← Controller (MVC Pattern)
       │  (KondoApp)    │
       └───────┬────────┘
               │
    ┌──────────┼──────────────┐
    │          │              │
┌───▼───┐  ┌──▼───┐  ┌──────▼─────┐
│api.js │  │state.js│ │components.js│
│(Repo) │  │(Obs.)  │ │  (Factory)  │
└───────┘  └────────┘ └────────────┘
                │
            ┌───▼────┐
            │utils.js│
            └────────┘
```

#### 2. Module Responsibilities

**main.js** - Bootstrap
- Application initialization
- Global error handling
- Development utilities

**app.js** - Application Controller (MVC Controller)
- Coordinates all modules
- Manages application flow
- Handles user interactions
- Updates UI based on state changes

**api.js** - Repository Pattern
- Abstracts all Tauri backend communication
- Centralized error handling
- Provides clean API interface
- Modules: `DirectoryAPI`, `ConfigAPI`

**state.js** - Observable State (Observer Pattern)
- Reactive state management
- Subscribe/notify pattern
- Centralized application state
- Type-safe observable values

**components.js** - Component Factory (Factory Pattern)
- Reusable UI component builders
- `ComponentFactory` for common elements
- `ModalDialog` class for dialogs
- Consistent styling and behavior

**utils.js** - DRY Utilities
- `DOM` - DOM manipulation helpers
- `Format` - Formatting utilities
- `Async` - Async operation helpers
- `Validate` - Validation functions

#### 3. Design Patterns

**Observer Pattern** (State Management)
```javascript
class Observable {
    subscribe(callback) { /* ... */ }
    _notify() { /* notify all subscribers */ }
}

// Usage
appState.currentFolder.subscribe(folder => {
    updateUI(folder);
});
```

**Repository Pattern** (API Layer)
```javascript
export const DirectoryAPI = {
    async scan(path, options) { /* ... */ },
    async organize(path, options) { /* ... */ }
};
```

**Factory Pattern** (Component Creation)
```javascript
export const ComponentFactory = {
    createCategoryCard(name, files, options) { /* ... */ },
    createAlert(message, type) { /* ... */ }
};
```

**Controller Pattern** (MVC)
```javascript
export class KondoApp {
    constructor() {
        this.state = appState;
        this.setupStateSubscriptions();
    }

    init() {
        this.setupTabs();
        this.setupHomeTab();
        // ...
    }
}
```

## Data Flow

### Scan Directory Example

```
Frontend                 Backend
────────                 ───────

User clicks "Scan"
     │
     ├─ app.scanFolder()
     │  ├─ state.setLoading(true)
     │  └─ DirectoryAPI.scan(folder)
     │          │
     │          └─ invoke('scan_directory', request)
     │                      │
     │                      ├─ handlers::scan_directory
     │                      │   └─ OrganizerService::scan
     │                      │       └─ KondoOrganizer::scan
     │                      │           └─ Returns ScanResult
     │                      │
     │          ┌───────────┘
     │          │
     ├─ state.setScanData(result)
     │  └─ Notifies subscribers
     │      └─ app.displayScanResults()
     │          └─ ComponentFactory creates UI
     └─ state.setLoading(false)
```

## Principles Applied

### SOLID Principles

**Single Responsibility**
- Each module has one clear purpose
- Services handle business logic only
- Handlers only dispatch to services

**Open/Closed**
- Easy to extend with new features
- Closed to modification (stable interfaces)

**Dependency Inversion**
- High-level modules depend on abstractions
- Services abstract implementation details

### DRY (Don't Repeat Yourself)

**Backend**
- Shared error handling functions
- Reusable validation logic
- Common path utilities

**Frontend**
- Centralized API calls
- Reusable component builders
- Shared utility functions

### Separation of Concerns

**Backend Layers**
1. Handlers - IPC interface
2. Services - Business logic
3. Library - Domain logic

**Frontend Modules**
1. API - Data access
2. State - Data management
3. App - Application logic
4. Components - UI building
5. Utils - Helper functions

## Error Handling

### Backend
```rust
pub enum AppError {
    Io(String),
    Config(String),
    Validation(String),
    Path(String),
    Operation(String),
}

// Automatic conversion
impl From<std::io::Error> for AppError { /* ... */ }
```

### Frontend
```javascript
class ApiError extends Error {
    constructor(message, original) { /* ... */ }
}

function handleError(error, context) {
    throw new ApiError(`${context}: ${message}`, error);
}
```

## State Management

### Observable Pattern
```javascript
// Create observable
this.currentFolder = new Observable(null);

// Subscribe to changes
this.currentFolder.subscribe(folder => {
    console.log('Folder changed:', folder);
});

// Update value (notifies subscribers)
this.currentFolder.value = '/new/path';
```

## Testing Strategy

### Backend (Rust)
- Unit tests for services
- Integration tests for handlers
- Mock Tauri context

### Frontend (JavaScript)
- Unit tests for utilities
- Component tests
- State management tests
- E2E tests with Tauri test harness

## Performance Considerations

1. **Lazy Loading** - Modules loaded as needed
2. **Debouncing** - User input debounced
3. **Efficient Rendering** - Only update changed components
4. **Memory Management** - Proper cleanup of subscriptions

## Security

1. **Path Validation** - All paths validated before use
2. **Error Sanitization** - No sensitive data in errors
3. **CSP Headers** - Content Security Policy configured
4. **Input Validation** - All user input validated

## Future Enhancements

1. **Undo/Redo** - Command pattern for operations
2. **Batch Operations** - Process multiple folders
3. **Custom Rules** - User-defined organization rules
4. **Plugins** - Extensible plugin system
5. **Theming** - Custom color schemes
6. **Internationalization** - Multi-language support

## Maintainability Benefits

✅ **Easy to Test** - Clear separation of concerns
✅ **Easy to Extend** - Open/Closed principle
✅ **Easy to Debug** - Centralized error handling
✅ **Easy to Understand** - Consistent patterns
✅ **Easy to Refactor** - Low coupling, high cohesion
