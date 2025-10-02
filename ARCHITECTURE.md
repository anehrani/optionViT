# 🏗️ Code Architecture Documentation

## Overview

The codebase has been refactored into a **clean, modular architecture** following Rust best practices. Instead of having all code in a single 1363-line `main.rs` file, the functionality is now split into focused, reusable modules.

## 📁 Project Structure

```
src/
├── main.rs          # Entry point (20 lines) - Server setup only
├── lib.rs           # Module exports and re-exports
├── types.rs         # Data structures and type definitions
├── api_client.rs    # Deribit API client implementation
├── filters.rs       # Server-side filtering logic
├── handlers.rs      # HTTP request handlers
└── ui.rs            # HTML/CSS/JavaScript UI content
```

## 📦 Module Breakdown

### 1. `main.rs` - Application Entry Point
**Purpose**: Bootstrap the HTTP server
**Lines**: ~20
**Responsibilities**:
- Initialize Actix-web server
- Configure CORS
- Define routes
- Start listening on port 8080

**Key Code**:
```rust
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .wrap(Cors::permissive())
            .route("/", web::get().to(index))
            .route("/api/options", web::get().to(get_options))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

### 2. `lib.rs` - Public API
**Purpose**: Module organization and public exports
**Responsibilities**:
- Declare all modules
- Re-export public APIs
- Provide clean interface for external use

**Exports**:
```rust
pub mod api_client;
pub mod filters;
pub mod handlers;
pub mod types;
pub mod ui;

pub use api_client::DeribitClient;
pub use handlers::get_options;
pub use types::{OptionsRequest, OptionsRequestOld};
pub use ui::index;
```

### 3. `types.rs` - Type Definitions
**Purpose**: Define data structures
**Lines**: ~25
**Responsibilities**:
- Request/response types
- Serialization annotations
- API contracts

**Key Types**:
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct OptionsRequest {
    pub currency: String,
    pub creation_from: Option<String>,
    pub creation_to: Option<String>,
    pub include_expired: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptionsRequestOld {
    // Legacy structure for backwards compatibility
    ...
}
```

### 4. `api_client.rs` - Deribit API Client
**Purpose**: Handle all external API communication
**Lines**: ~150
**Responsibilities**:
- HTTP client initialization
- Fetch instruments from Deribit
- Fetch ticker data for instruments
- Enrich instrument data with market info
- Concurrent API calls with `join_all`

**Key Methods**:
```rust
impl DeribitClient {
    pub fn new() -> Self
    pub async fn fetch_instruments(...) -> Result<Vec<Value>, String>
    pub async fn fetch_ticker(...) -> Option<Value>
    pub async fn enrich_instruments(...) -> Vec<Value>
}
```

**Features**:
- ✅ Async/await for all network calls
- ✅ Concurrent enrichment of up to 100 instruments
- ✅ Proper error handling
- ✅ Clean separation of concerns

### 5. `filters.rs` - Filtering Logic
**Purpose**: Server-side data filtering
**Lines**: ~100
**Responsibilities**:
- Filter by creation date range
- Debug logging for sample instruments
- Date parsing and timestamp conversions

**Key Functions**:
```rust
pub fn filter_by_creation_date(
    instruments: Vec<Value>,
    creation_from: Option<&String>,
    creation_to: Option<&String>,
) -> Vec<Value>

pub fn print_sample_instruments(instruments: &[Value], count: usize)
```

**Philosophy**:
- Server filters only by creation date (performance optimization)
- All other filtering done client-side (instant response)

### 6. `handlers.rs` - HTTP Handlers
**Purpose**: Handle HTTP requests
**Lines**: ~70
**Responsibilities**:
- Parse query parameters
- Orchestrate API calls and filtering
- Return JSON responses
- Error handling

**Key Handler**:
```rust
pub async fn get_options(query: web::Query<OptionsRequest>) -> Result<HttpResponse>
```

**Flow**:
1. Log query parameters
2. Create DeribitClient
3. Fetch instruments from API
4. Apply creation date filter
5. Limit to 100 instruments
6. Enrich with ticker data
7. Return JSON response

### 7. `ui.rs` - User Interface
**Purpose**: Serve HTML/CSS/JavaScript
**Lines**: ~1000+ (but isolated)
**Responsibilities**:
- Serve the main HTML page
- Embedded CSS styles
- JavaScript for two-part filtering
- Client-side filtering logic

**Key Function**:
```rust
pub async fn index() -> Result<HttpResponse>
fn get_html_content() -> &'static str  // Returns complete HTML
```

**UI Features**:
- Two-part filtering system UI
- Modern gradient design
- Responsive layout
- Real-time filtering
- Statistics cards
- Sortable data table

## 🔄 Data Flow

```
User Browser
    ↓
  index() [ui.rs]
    ↓
HTML/CSS/JS sent to browser
    ↓
User clicks "Fetch Data"
    ↓
JavaScript fetch to /api/options
    ↓
  get_options() [handlers.rs]
    ↓
  DeribitClient::fetch_instruments() [api_client.rs]
    ↓
Deribit API (instruments)
    ↓
  filter_by_creation_date() [filters.rs]
    ↓
  DeribitClient::enrich_instruments() [api_client.rs]
    ↓
Deribit API (ticker) x100 concurrent
    ↓
JSON response to browser
    ↓
JavaScript applyClientFilters()
    ↓
Display filtered data
```

## ✅ Benefits of This Architecture

### 1. **Readability**
- Each module has a single responsibility
- Easy to find specific functionality
- Self-documenting code organization

### 2. **Maintainability**
- Changes isolated to relevant modules
- No need to search through 1000+ lines
- Clear dependencies between components

### 3. **Testability**
- Each module can be unit tested independently
- Mock API calls easily
- Test filters without server

### 4. **Reusability**
- `DeribitClient` can be used in other projects
- Filters can be extracted to library
- Types can be shared between services

### 5. **Scalability**
- Easy to add new API endpoints
- Simple to add more filters
- Can split into microservices later

## 🔧 How to Extend

### Adding a New Filter
1. Add filter logic to `filters.rs`
2. Update `OptionsRequest` in `types.rs` if needed
3. Modify `get_options()` in `handlers.rs` to use new filter
4. Update UI in `ui.rs` if needed

### Adding a New API Endpoint
1. Define handler in `handlers.rs`
2. Add route in `main.rs`
3. Add UI elements in `ui.rs` if needed

### Adding New Data Sources
1. Create new module like `binance_client.rs`
2. Implement similar interface to `DeribitClient`
3. Add to `lib.rs` exports

## 📊 Comparison: Before vs After

| Aspect | Before | After |
|--------|--------|-------|
| **Files** | 1 monolithic file | 7 focused modules |
| **Lines in main.rs** | 1363 lines | 20 lines |
| **Readability** | Hard to navigate | Clear structure |
| **Testability** | Difficult | Easy |
| **Reusability** | None | High |
| **Maintainability** | Low | High |

## 🚀 Future Improvements

1. **Split UI into separate files**:
   - `templates/index.html`
   - `static/css/styles.css`
   - `static/js/app.js`

2. **Add proper error types**:
   ```rust
   pub enum DeribitError {
       NetworkError(String),
       ParseError(String),
       ApiError(String),
   }
   ```

3. **Add unit tests**:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       
       #[test]
       fn test_filter_by_creation_date() {
           // ...
       }
   }
   ```

4. **Add configuration module**:
   ```rust
   pub struct Config {
       pub port: u16,
       pub api_base_url: String,
       pub max_instruments: usize,
   }
   ```

5. **Add logging framework**:
   - Replace `println!` with `tracing` or `log`
   - Add structured logging
   - Log levels (debug, info, warn, error)

## 📝 Code Style Guidelines

### Module Organization
- Public items first
- Private items last
- Related functionality grouped together

### Naming Conventions
- Modules: snake_case (e.g., `api_client.rs`)
- Structs: PascalCase (e.g., `DeribitClient`)
- Functions: snake_case (e.g., `fetch_instruments`)
- Constants: SCREAMING_SNAKE_CASE (e.g., `MAX_ITEMS`)

### Documentation
- Every public function should have doc comments
- Modules should have module-level documentation
- Complex logic should have inline comments

### Error Handling
- Use `Result<T, E>` for fallible operations
- Provide meaningful error messages
- Log errors before returning

## 🎯 Summary

The refactored codebase is now:
- ✅ **Modular**: Clear separation of concerns
- ✅ **Clean**: Each file < 200 lines (except UI)
- ✅ **Testable**: Easy to write unit tests
- ✅ **Maintainable**: Easy to find and fix bugs
- ✅ **Scalable**: Easy to add new features
- ✅ **Professional**: Follows Rust best practices

The code is now ready for production use and team collaboration!
