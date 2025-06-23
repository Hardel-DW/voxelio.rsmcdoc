//! WASM bindings for RSMCDOC - Production Ready API
//! Provides the exact TypeScript interface specified in developpement-plan.md

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
use crate::validator::DatapackValidator as InnerValidator;

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

/// Main DatapackValidator for WASM - EXACTLY matches TypeScript interface
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct DatapackValidator {
    // Use Box to avoid lifetime issues in WASM
    inner: Box<InnerValidator<'static>>,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl DatapackValidator {
    /// Initialisation with registries, MCDOC, and version.
    #[wasm_bindgen]
    pub fn init(registries: JsValue, mcdoc_files: JsValue, version: String) -> Result<DatapackValidator, JsValue> {
        let mut inner_validator = InnerValidator::new();

        // 1. Charger les registries
        let registries_map: HashMap<String, serde_json::Value> = serde_wasm_bindgen::from_value(registries)
            .map_err(|e| to_js_error("Invalid registries format", e))?;
        for (name, registry_data) in registries_map {
            inner_validator.load_registry(name, version.clone(), &registry_data)
                .map_err(|e| to_js_error("Registry loading failed", e))?;
        }

        // 2. Charger les fichiers MCDOC
        let files_map: HashMap<String, String> = serde_wasm_bindgen::from_value(mcdoc_files)
            .map_err(|e| to_js_error("Invalid MCDOC files format", e))?;
        for (filename, content) in files_map {
            // Convert content to static lifetime by leaking memory (acceptable for WASM usage)
            let static_content: &'static str = Box::leak(content.into_boxed_str());
            match crate::parse_mcdoc(static_content) {
                Ok(ast) => {
                    inner_validator.load_parsed_mcdoc(filename, ast)
                        .map_err(|e| to_js_error("Failed to load MCDOC schema", e))?;
                }
                Err(parse_errors) => {
                    let error_msg = format!("MCDOC parse errors in {}: {:?}", filename, parse_errors);
                    return Err(to_js_error("MCDOC parsing failed", &error_msg[..]));
                }
            }
        }
        
        Ok(DatapackValidator { inner: Box::new(inner_validator) })
    }

    /// Validation d'un JSON unique
    #[wasm_bindgen]
    pub fn validate(&self, json: JsValue, resource_type: &str, version: Option<String>) -> Result<JsValue, JsValue> {
        let json_value: serde_json::Value = serde_wasm_bindgen::from_value(json)
            .map_err(|e| to_js_error("Invalid JSON format", e))?;
        
        let result = self.inner.validate_json(&json_value, resource_type, version.as_deref());
        
        serde_wasm_bindgen::to_value(&result)
            .map_err(|e| to_js_error("Serialization error", e))
    }

    /// Analyse complète d'un datapack
    #[wasm_bindgen]
    pub fn analyze_datapack(&self, files: JsValue) -> Result<JsValue, JsValue> {
        let files_map: HashMap<String, serde_json::Value> = serde_wasm_bindgen::from_value(files)
            .map_err(|e| to_js_error("Invalid files format", e))?;
        
        let mut results = HashMap::new();
        
        for (file_path, json_content) in files_map {
            // Generic resource type inference from file path
            let resource_type = if file_path.contains("/recipes/") {
                "recipe"
            } else if file_path.contains("/loot_tables/") {
                "loot_table"
            } else if file_path.contains("/advancements/") {
                "advancement"
            } else if file_path.contains("/structures/") {
                "structure"
            } else if file_path.contains("/tags/") {
                "tag"
            } else {
                // Extract from path: data/namespace/type/file.json -> type
                let parts: Vec<&str> = file_path.split('/').collect();
                if parts.len() >= 4 && parts[0] == "data" {
                    parts[2] // Get the type part
                } else {
                    "unknown"
                }
            };
            
            let result = self.inner.validate_json(&json_content, resource_type, None);
            results.insert(file_path, result);
        }
        
        serde_wasm_bindgen::to_value(&results)
            .map_err(|e| to_js_error("Serialization error", e))
    }
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
} 