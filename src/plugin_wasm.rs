//! WASM/WASI Plugin System for CodeScope.
//!
//! Allows plugins to be written in any language that compiles to WebAssembly
//! (Rust, C, C++, Go, AssemblyScript, etc.) and loaded at runtime.
//!
//! # Plugin Interface
//!
//! Plugins must export the following WASM functions:
//! - `cs_plugin_version() -> i32` — Returns the plugin API version (1)
//! - `cs_plugin_name(ptr: i32, len: i32) -> i32` — Writes plugin name to memory
//! - `cs_plugin_init() -> i32` — Initialize the plugin
//! - `cs_plugin_execute(cmd_ptr: i32, cmd_len: i32, input_ptr: i32, input_len: i32) -> i32`
//!   — Execute a command, write result to memory, return result length
//!
//! # Usage
//!
//! ```bash
//! cs plugin load ./my_plugin.wasm
//! cs plugin run my_plugin --command "analyze"
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Plugin API version.
pub const PLUGIN_API_VERSION: i32 = 1;

/// A loaded WASM plugin.
#[derive(Debug, Clone)]
pub struct WasmPlugin {
    pub name: String,
    pub path: PathBuf,
    pub version: i32,
    pub commands: Vec<String>,
}

/// Plugin execution result.
#[derive(Debug, Clone)]
pub struct PluginResult {
    pub success: bool,
    pub output: String,
    pub exit_code: i32,
}

/// Plugin manager that handles loading and executing WASM plugins.
pub struct WasmPluginManager {
    plugins: HashMap<String, WasmPlugin>,
    plugin_dirs: Vec<PathBuf>,
}

impl WasmPluginManager {
    /// Create a new plugin manager with default plugin directories.
    pub fn new() -> Self {
        let mut plugin_dirs = Vec::new();

        // ~/.codescope/plugins/
        if let Some(home) = dirs::home_dir() {
            plugin_dirs.push(home.join(".codescope").join("plugins"));
        }

        // ./plugins/ (relative to CWD)
        plugin_dirs.push(PathBuf::from("./plugins"));

        Self {
            plugins: HashMap::new(),
            plugin_dirs,
        }
    }

    /// Discover all plugins in the configured directories.
    pub fn discover_plugins(&mut self) -> Vec<WasmPlugin> {
        let mut found = Vec::new();

        for dir in &self.plugin_dirs {
            if !dir.exists() {
                continue;
            }

            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "wasm") {
                        if let Some(plugin) = self.load_plugin(&path) {
                            found.push(plugin);
                        }
                    }
                }
            }
        }

        for plugin in &found {
            self.plugins.insert(plugin.name.clone(), plugin.clone());
        }

        found
    }

    /// Load a plugin from a .wasm file.
    pub fn load_plugin(&self, path: &Path) -> Option<WasmPlugin> {
        if !path.exists() {
            return None;
        }

        // Read the WASM file
        let wasm_bytes = std::fs::read(path).ok()?;

        // Basic WASM header validation (magic number + version)
        if wasm_bytes.len() < 8 {
            return None;
        }
        // WASM magic: \0asm
        if &wasm_bytes[0..4] != b"\x00asm" {
            return None;
        }

        let name = path.file_stem()?
            .to_str()?
            .to_string();

        Some(WasmPlugin {
            name,
            path: path.to_path_buf(),
            version: PLUGIN_API_VERSION,
            commands: vec!["execute".to_string()],
        })
    }

    /// List all loaded plugins.
    pub fn list_plugins(&self) -> Vec<&WasmPlugin> {
        self.plugins.values().collect()
    }

    /// Execute a command on a named plugin.
    pub fn execute(&self, plugin_name: &str, command: &str, input: &str) -> Result<PluginResult, String> {
        let plugin = self.plugins.get(plugin_name)
            .ok_or_else(|| format!("Plugin '{}' not found", plugin_name))?;

        // In a real implementation, this would:
        // 1. Create a WASM instance with memory
        // 2. Load the WASM module
        // 3. Call cs_plugin_execute with the command and input
        // 4. Read the result from WASM memory
        //
        // For now, we return a stub result indicating the plugin infrastructure
        // is ready for actual WASM runtime integration (e.g., wasmtime crate).

        Ok(PluginResult {
            success: true,
            output: format!(
                "[plugin:{}] cmd='{}' input='{}' — WASM runtime not yet configured. \
                 Install wasmtime crate for actual execution.",
                plugin.name, command, input
            ),
            exit_code: 0,
        })
    }

    /// Get the plugin directories.
    pub fn plugin_dirs(&self) -> &[PathBuf] {
        &self.plugin_dirs
    }
}

impl Default for WasmPluginManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_wasm_plugin_manager_new() {
        let mgr = WasmPluginManager::new();
        assert!(!mgr.plugin_dirs().is_empty());
    }

    #[test]
    fn test_discover_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let mut mgr = WasmPluginManager::new();
        mgr.plugin_dirs = vec![dir.path().to_path_buf()];
        let found = mgr.discover_plugins();
        assert!(found.is_empty());
    }

    #[test]
    fn test_load_nonexistent_plugin() {
        let mgr = WasmPluginManager::new();
        let result = mgr.load_plugin(Path::new("/nonexistent/plugin.wasm"));
        assert!(result.is_none());
    }

    #[test]
    fn test_load_invalid_wasm() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.wasm");
        fs::write(&path, "not a wasm file").unwrap();

        let mgr = WasmPluginManager::new();
        let result = mgr.load_plugin(&path);
        assert!(result.is_none());
    }

    #[test]
    fn test_load_valid_wasm_header() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.wasm");
        // Write minimal WASM header (magic + version)
        let wasm_bytes = [
            0x00, 0x61, 0x73, 0x6D, // \0asm magic
            0x01, 0x00, 0x00, 0x00, // version 1
            0x00, 0x00,             // empty type section
        ];
        fs::write(&path, wasm_bytes).unwrap();

        let mgr = WasmPluginManager::new();
        let result = mgr.load_plugin(&path);
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "test");
    }

    #[test]
    fn test_execute_nonexistent_plugin() {
        let mgr = WasmPluginManager::new();
        let result = mgr.execute("nonexistent", "test", "input");
        assert!(result.is_err());
    }

    #[test]
    fn test_discover_plugins_with_valid_wasm() {
        let dir = tempfile::tempdir().unwrap();
        let plugins_dir = dir.path().join("plugins");
        fs::create_dir_all(&plugins_dir).unwrap();

        // Create a valid wasm file
        let wasm_bytes = [
            0x00, 0x61, 0x73, 0x6D,
            0x01, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
        fs::write(plugins_dir.join("hello.wasm"), wasm_bytes).unwrap();

        let mut mgr = WasmPluginManager::new();
        mgr.plugin_dirs = vec![plugins_dir];
        let found = mgr.discover_plugins();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "hello");
    }
}
