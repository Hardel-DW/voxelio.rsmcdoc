//! WASM bindings for McDocValidator
//! Provides a JavaScript-compatible interface for MCDOC validation

use wasm_bindgen::prelude::*;
use crate::McDocValidator as CoreValidator;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use rustc_hash::FxHashMap;

// Use the smaller allocator for WASM
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

// When the `console_error_panic_hook` feature is enabled, we can call the
// `set_panic_hook` function at least once during initialization, and then
// we will get better error messages if our code ever panics.
//
// For more details see
// https://github.com/rustwasm/console_error_panic_hook#readme
#[cfg(feature = "console_error_panic_hook")]
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

/// JavaScript-compatible wrapper for McDocValidator
/// Uses owned strings to avoid lifetime issues in WASM
#[wasm_bindgen]
pub struct WasmMcDocValidator {
    // Store the MCDOC files as owned strings
    mcdoc_files: HashMap<String, String>,
    initialized: bool,
}

/// JavaScript-compatible validation result
#[derive(Serialize, Deserialize)]
pub struct WasmValidationResult {
    pub is_valid: bool,
    pub errors: Vec<WasmValidationError>,
    pub dependencies: Vec<WasmRegistryDependency>,
}

#[derive(Serialize, Deserialize)]
pub struct WasmValidationError {
    pub path: String,
    pub message: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Serialize, Deserialize)]
pub struct WasmRegistryDependency {
    pub resource_location: String,
    pub registry_type: String,
    pub source_path: String,
}

#[wasm_bindgen]
impl WasmMcDocValidator {
    /// Create a new MCDOC validator
    #[wasm_bindgen(constructor)]
    pub fn new() -> WasmMcDocValidator {
        console_log!("🎯 Creating new McDocValidator for WASM");
        WasmMcDocValidator {
            mcdoc_files: HashMap::new(),
            initialized: false,
        }
    }

    /// Load MCDOC files from a JavaScript object
    /// Expected format: { "path/to/file.mcdoc": "file content", ... }
    #[wasm_bindgen]
    pub fn load_mcdoc_files(&mut self, files_obj: &JsValue) -> Result<(), JsValue> {
        #[cfg(feature = "serde-wasm-bindgen")]
        let files: HashMap<String, String> = serde_wasm_bindgen::from_value(files_obj.clone())
            .map_err(|e| JsValue::from_str(&format!("Failed to parse MCDOC files: {}", e)))?;
        
        #[cfg(not(feature = "serde-wasm-bindgen"))]
        let files: HashMap<String, String> = HashMap::new(); // Fallback for minimal WASM

        console_log!("📦 Loading {} MCDOC files", files.len());

        self.mcdoc_files = files;
        self.initialized = true;
        
        console_log!("✅ MCDOC files loaded successfully");
        Ok(())
    }

    /// Load registry data from JSON
    /// Expected format: { "item": ["diamond_sword", "stick", ...], "block": [...], ... }
    #[wasm_bindgen]
    pub fn load_registries(&mut self, _registries_json: &JsValue, _version: &str) -> Result<(), JsValue> {
        // For now, we'll skip registry loading in WASM
        // This can be implemented later with a different approach
        console_log!("📊 Registry loading in WASM is not yet implemented");
        Ok(())
    }

    /// Validate a JSON object against MCDOC schemas
    #[wasm_bindgen]
    pub fn validate_json(&self, json_data: &JsValue, resource_type: &str) -> Result<JsValue, JsValue> {
        if !self.initialized {
            return Err(JsValue::from_str("MCDOC files not loaded. Call load_mcdoc_files() first."));
        }

        #[cfg(feature = "serde-wasm-bindgen")]
        let json: serde_json::Value = serde_wasm_bindgen::from_value(json_data.clone())
            .map_err(|e| JsValue::from_str(&format!("Failed to parse JSON: {}", e)))?;
        
        #[cfg(not(feature = "serde-wasm-bindgen"))]
        let json: serde_json::Value = serde_json::Value::Null; // Fallback pour WASM minimal

        console_log!("🔍 Validating {} JSON", resource_type);

        // Create validator with owned string references
        let files_with_refs: FxHashMap<String, &str> = self.mcdoc_files
            .iter()
            .map(|(k, v)| (k.clone(), v.as_str()))
            .collect();

        // Create a temporary validator for this validation
        match CoreValidator::init(files_with_refs) {
            Ok(validator) => {
                let result = validator.validate_json(&json, resource_type);

                let wasm_result = WasmValidationResult {
                    is_valid: result.is_valid,
                    errors: result.errors.into_iter().map(|e| WasmValidationError {
                        path: e.path,
                        message: e.message,
                        line: e.line.unwrap_or(0),
                        column: e.column.unwrap_or(0),
                    }).collect(),
                    dependencies: result.dependencies.into_iter().map(|d| WasmRegistryDependency {
                        resource_location: d.resource_location,
                        registry_type: d.registry_type,
                        source_path: d.source_path,
                    }).collect(),
                };

                console_log!("📋 Validation result: valid={}, errors={}, deps={}", 
                    wasm_result.is_valid, 
                    wasm_result.errors.len(), 
                    wasm_result.dependencies.len()
                );

                #[cfg(feature = "serde-wasm-bindgen")]
                {
                    serde_wasm_bindgen::to_value(&wasm_result)
                        .map_err(|e| JsValue::from_str(&format!("Failed to serialize result: {}", e)))
                }
                #[cfg(not(feature = "serde-wasm-bindgen"))]
                {
                    Ok(JsValue::NULL) // Fallback pour WASM minimal
                }
            },
            Err(errors) => {
                let error_msg = format!("Validator initialization failed: {} errors", errors.len());
                Err(JsValue::from_str(&error_msg))
            }
        }
    }

    /// Get registry statistics
    #[wasm_bindgen]
    pub fn get_registry_stats(&self) -> Result<JsValue, JsValue> {
        // For now, return empty stats since registry loading is not implemented in WASM yet
        let empty_stats: HashMap<String, serde_json::Value> = HashMap::new();
        #[cfg(feature = "serde-wasm-bindgen")]
        {
            serde_wasm_bindgen::to_value(&empty_stats)
                .map_err(|e| JsValue::from_str(&format!("Failed to serialize stats: {}", e)))
        }
        #[cfg(not(feature = "serde-wasm-bindgen"))]
        {
            Ok(JsValue::NULL) // Fallback pour WASM minimal
        }
    }

    /// Get available MCDOC file paths
    #[wasm_bindgen]
    pub fn get_mcdoc_files(&self) -> Vec<String> {
        self.mcdoc_files.keys().cloned().collect()
    }

    /// Load test registries (for development/testing)
    #[wasm_bindgen]
    pub fn load_test_registries(&mut self) -> Result<(), JsValue> {
        console_log!("⚠️ Test registry loading is not available in WASM (filesystem access not supported)");
        Err(JsValue::from_str("Test registry loading is not supported in WASM environment"))
    }
}

/// Utility function to set up panic hook (call this from JavaScript)
#[wasm_bindgen]
pub fn setup_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Get version information
#[wasm_bindgen]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
} 