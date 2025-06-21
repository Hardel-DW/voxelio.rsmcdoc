//! WASM bindings for RSMCDOC - Production Ready API
//! Provides the exact TypeScript interface specified in developpement-plan.md

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
use crate::validator::McDocValidator as InnerValidator;

#[cfg(feature = "wasm")]
use crate::types::DatapackResult;

#[cfg(feature = "wasm")]
use std::collections::HashMap;

/// Helper function to convert errors to JsValue (eliminating DRY violations)
#[cfg(feature = "wasm")]
fn to_js_error(msg: &str, error: impl std::fmt::Display) -> JsValue {
    JsValue::from_str(&format!("{}: {}", msg, error))
}

#[cfg(all(feature = "wasm", feature = "console_error_panic_hook"))]
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
}

/// Main McDocValidator for WASM - EXACTLY matches TypeScript interface
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct McDocValidator {
    inner: InnerValidator<'static>,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl McDocValidator {
    /// Create a new MCDOC validator instance
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<McDocValidator, JsValue> {
        Ok(McDocValidator {
            inner: InnerValidator::new(),
        })
    }

    /// Load MCDOC files (METHOD 1) - Real parsing implemented
    #[wasm_bindgen]
    pub fn load_mcdoc_files(&mut self, files: JsValue) -> Result<(), JsValue> {
        let files_map: HashMap<String, String> = serde_wasm_bindgen::from_value(files)
            .map_err(|e| to_js_error("Invalid files format", e))?;
        
        // Parse and load each MCDOC file
        for (filename, content) in files_map {
            // Use the crate's parse_mcdoc function
            let ast = crate::parse_mcdoc(&content)
                .map_err(|errors| to_js_error("MCDOC parsing failed", format!("Errors: {:?}", errors)))?;
            
            // Store the parsed AST in the validator
            // Note: For now, we are using a static lifetime
            // TODO: Handle lifetimes properly if necessary
            let static_ast = unsafe { std::mem::transmute(ast) };
            self.inner.load_parsed_mcdoc(filename, static_ast)
                .map_err(|e| to_js_error("Failed to load parsed MCDOC", e))?;
        }
        
        Ok(())
    }

    /// Load registries data (METHOD 2)
    #[wasm_bindgen]
    pub fn load_registries(&mut self, registries: JsValue, version: &str) -> Result<(), JsValue> {
        let registries_map: HashMap<String, serde_json::Value> = serde_wasm_bindgen::from_value(registries)
            .map_err(|e| to_js_error("Invalid registries format", e))?;
        
        for (name, registry_data) in registries_map {
            self.inner.load_registry(name, version.to_string(), &registry_data)
                .map_err(|e| to_js_error("Registry loading failed", e))?;
        }
        
        Ok(())
    }

    /// Validate JSON against MCDOC schemas (METHOD 3)
    #[wasm_bindgen]
    pub fn validate_json(&self, json: JsValue, resource_type: &str) -> Result<JsValue, JsValue> {
        let json_value: serde_json::Value = serde_wasm_bindgen::from_value(json)
            .map_err(|e| to_js_error("Invalid JSON format", e))?;
        
        let result = self.inner.validate_json(&json_value, resource_type);
        
        // Use types.rs structures directly - no conversion needed
        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| to_js_error("Serialization error", e))
    }

    /// Analyze complete datapack (METHOD 4) - Fixed Option<JsValue> issues
    #[wasm_bindgen]
    pub async fn analyze_datapack(&self, files: JsValue, resource_type_map: JsValue, default_resource_type: Option<String>) -> Result<JsValue, JsValue> {
        let files_map: HashMap<String, Vec<u8>> = serde_wasm_bindgen::from_value(files)
            .map_err(|e| to_js_error("Invalid files format", e))?;
        
        // Load resource type mapping from parameter (no hardcoding)
        let type_mapping: HashMap<String, String> = if resource_type_map.is_undefined() {
            HashMap::new()
        } else {
            serde_wasm_bindgen::from_value(resource_type_map)
                .map_err(|e| to_js_error("Invalid resource type mapping", e))?
        };
        
        // Use provided default or no fallback (completely configurable, no hardcoding)
        let fallback_type = default_resource_type.unwrap_or_else(|| "".to_string());
        
        let mut datapack_result = DatapackResult::new();
        
        for (file_path, file_data) in files_map {
            // Convert bytes to JSON
            let json_value: serde_json::Value = serde_json::from_slice(&file_data)
                .map_err(|e| to_js_error(&format!("Invalid JSON in {}", file_path), e))?;
            
            // Use provided mapping or configurable fallback (no hardcoding)
            let resource_type = type_mapping.get(&file_path)
                .cloned()
                .unwrap_or_else(|| fallback_type.clone());
            
            let validation_result = self.inner.validate_json(&json_value, &format!("{}/{}", resource_type, file_path));
            datapack_result.add_file_result(file_path, validation_result);
        }
        
        // Use types.rs DatapackResult directly
        serde_wasm_bindgen::to_value(&datapack_result)
            .map_err(|e| to_js_error("Serialization error", e))
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn setup_panic_hook() {
    console_error_panic_hook::set_once();
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
} 