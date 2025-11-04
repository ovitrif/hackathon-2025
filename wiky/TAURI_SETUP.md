# Tauri Migration Complete

This project has been successfully migrated from eframe/egui to Tauri.

## Project Structure

```
wiky/
├── src-tauri/          # Tauri Rust backend
│   ├── src/
│   │   ├── main.rs     # Tauri entry point with commands
│   │   ├── lib.rs      # Shared library code
│   │   └── utils.rs    # Utility functions
│   ├── Cargo.toml      # Rust dependencies
│   ├── build.rs        # Build script
│   └── tauri.conf.json # Tauri configuration
├── ui/                 # Frontend (HTML/CSS/JS)
│   ├── index.html      # Main UI
│   ├── styles.css      # Styling
│   └── app.js          # Frontend logic
└── assets/             # Resources (logo, etc.)
```

## System Dependencies (Linux)

Before building on Linux, install these system dependencies:

```bash
sudo apt-get update
sudo apt-get install -y \
    libgtk-3-dev \
    libwebkit2gtk-4.0-dev \
    libayatana-appindicator3-dev \
    librsvg2-dev
```

## Building and Running

1. Navigate to the src-tauri directory:
   ```bash
   cd wiky/src-tauri
   ```

2. Build the project:
   ```bash
   cargo build
   ```

3. Run the application:
   ```bash
   cargo run
   ```

## Key Changes from eframe to Tauri

1. **Architecture**: Moved from immediate-mode GUI (egui) to web-based UI with HTML/CSS/JS
2. **Backend**: Rust backend now uses Tauri commands instead of eframe's update loop
3. **Frontend**: Created a web-based UI that communicates with the Rust backend via Tauri's IPC
4. **State Management**: Auth and app state managed in Rust, accessed via Tauri commands
5. **QR Code**: Generated as base64-encoded PNG for web display

## Features Preserved

- ✅ Authentication with QR code scanning
- ✅ Create, edit, view, and delete wiki pages
- ✅ Fork wiki pages from other users
- ✅ Markdown rendering
- ✅ Share page links
- ✅ Discover forks from followed users

## Tauri Commands

The following commands are available from the frontend:

- `get_auth_state()` - Get current authentication state
- `start_authentication()` - Start auth flow
- `get_wiki_pages()` - List all wiki pages
- `get_wiki_content(userId, pageId)` - Get wiki content
- `create_wiki(content, filename?)` - Create new wiki
- `update_wiki(pageId, content)` - Update existing wiki
- `delete_wiki(pageId)` - Delete wiki
- `get_qr_image()` - Get QR code as base64 image
- `get_follows()` - Get list of followed users
- `discover_forks(pageId)` - Find forks of a page
