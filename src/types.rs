//! Public types for the MCDOC API

use serde::{Deserialize, Serialize, Serializer, Deserializer};
use crate::error::{ErrorType, ParseError};
use serde::ser::SerializeMap;
use serde::de::{Visitor, MapAccess};

/// Registry dependency extracted from a JSON
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McDocDependency {
    /// Resource location (e.g., "minecraft:diamond_sword")
    pub resource_location: String,
    /// Registry type (e.g., "item", "block", "recipe")
    pub registry_type: String,
    /// Path in the source JSON (e.g., "result", "ingredients[0]")
    pub source_path: String,
    /// Optional source file for datapack analysis
    pub source_file: Option<String>,
    /// Indicates if it's a tag reference (#minecraft:swords)
    pub is_tag: bool,
}

/// MCDOC validation error
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McDocError {
    /// File name where the error occurred
    pub file: String,
    /// Path in the JSON structure
    pub path: String,
    /// Detailed error message
    pub message: String,
    /// Error type for categorization
    pub error_type: ErrorType,
    /// Line in the file (if available)
    pub line: Option<u32>,
    /// Column in the file (if available)
    pub column: Option<u32>,
}

impl From<ParseError> for McDocError {
    fn from(error: ParseError) -> Self {
        let (line, column) = error.position()
            .map(|pos| (Some(pos.line), Some(pos.column)))
            .unwrap_or((None, None));
        
        McDocError {
            file: String::new(), // Will be set by caller
            path: String::new(), // Will be set by caller
            message: error.to_string(),
            error_type: error.error_type(),
            line,
            column,
        }
    }
}

/// Validation result of a single JSON file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    /// Is the JSON valid according to the MCDOC schema?
    pub is_valid: bool,
    /// Detailed validation errors
    pub errors: Vec<McDocError>,
    /// Extracted registry dependencies
    pub dependencies: Vec<McDocDependency>,
}

impl ValidationResult {
    /// Create a successful validation result
    pub fn success(dependencies: Vec<McDocDependency>) -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            dependencies,
        }
    }
    
    /// Create a failed validation result
    pub fn failure(errors: Vec<McDocError>) -> Self {
        Self {
            is_valid: false,
            errors,
            dependencies: Vec::new(),
        }
    }
    
    /// Add an error to the result
    pub fn add_error(&mut self, error: McDocError) {
        self.errors.push(error);
        self.is_valid = false;
    }
    
    /// Add a dependency to the result
    pub fn add_dependency(&mut self, dependency: McDocDependency) {
        self.dependencies.push(dependency);
    }
}

/// Full datapack analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatapackResult {
    /// Total number of files analyzed
    pub total_files: usize,
    /// Number of valid files
    pub valid_files: usize,
    /// Validation errors per file
    pub errors: Vec<FileError>,
    /// All dependencies grouped by registry  
    #[serde(serialize_with = "serialize_fx_hashmap", deserialize_with = "deserialize_fx_hashmap")]
    pub dependencies: rustc_hash::FxHashMap<String, Vec<String>>,
    /// Total processing time in milliseconds
    pub analysis_time_ms: u32,
}

/// Error in a specific datapack file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileError {
    /// File path
    pub file_path: String,
    /// Validation error
    pub error: McDocError,
}

impl DatapackResult {
    /// Create a new empty result
    pub fn new() -> Self {
        Self {
            total_files: 0,
            valid_files: 0,
            errors: Vec::new(),
            dependencies: rustc_hash::FxHashMap::default(),
            analysis_time_ms: 0,
        }
    }
    
    /// Add file results
    pub fn add_file_result(&mut self, file_path: String, result: ValidationResult) {
        self.total_files += 1;
        
        if result.is_valid {
            self.valid_files += 1;
        }
        
        // Add errors
        for error in result.errors {
            self.errors.push(FileError {
                file_path: file_path.clone(),
                error,
            });
        }
        
        // Group dependencies by registry
        for dependency in result.dependencies {
            self.dependencies
                .entry(dependency.registry_type)
                .or_default()
                .push(dependency.resource_location);
        }
    }
    
    /// Set analysis time
    pub fn set_analysis_time(&mut self, time_ms: u32) {
        self.analysis_time_ms = time_ms;
    }
}

/// Minecraft Version - SIMPLIFIED VERSION (type alias)
/// Complex parsing is handled on the JavaScript side according to spec
pub type MinecraftVersion = String;

fn serialize_fx_hashmap<S>(
    map: &rustc_hash::FxHashMap<String, Vec<String>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut ser_map = serializer.serialize_map(Some(map.len()))?;
    for (key, value) in map {
        ser_map.serialize_entry(key, value)?;
    }
    ser_map.end()
}

fn deserialize_fx_hashmap<'de, D>(
    deserializer: D,
) -> Result<rustc_hash::FxHashMap<String, Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct FxHashMapVisitor;

    impl<'de> Visitor<'de> for FxHashMapVisitor {
        type Value = rustc_hash::FxHashMap<String, Vec<String>>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a map")
        }

        fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut map = rustc_hash::FxHashMap::default();

            while let Some((key, value)) = access.next_entry()? {
                map.insert(key, value);
            }

            Ok(map)
        }
    }

    deserializer.deserialize_map(FxHashMapVisitor)
} 