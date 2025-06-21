//! Types publics pour l'API MCDOC

use serde::{Deserialize, Serialize, Serializer, Deserializer};
use crate::error::{ErrorType, ParseError};
use serde::ser::SerializeMap;
use serde::de::{Visitor, MapAccess};

/// Dépendance registry extraite d'un JSON
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McDocDependency {
    /// Resource location (e.g., "minecraft:diamond_sword")
    pub resource_location: String,
    /// Type de registry (e.g., "item", "block", "recipe")
    pub registry_type: String,
    /// Chemin dans le JSON source (e.g., "result", "ingredients[0]")
    pub source_path: String,
    /// Fichier source optionnel pour datapack analysis
    pub source_file: Option<String>,
    /// Indique si c'est une référence tag (#minecraft:swords)
    pub is_tag: bool,
}

/// Erreur de validation MCDOC
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McDocError {
    /// Nom du fichier où l'erreur s'est produite
    pub file: String,
    /// Chemin dans la structure JSON
    pub path: String,
    /// Message d'erreur détaillé
    pub message: String,
    /// Type d'erreur pour catégorisation
    pub error_type: ErrorType,
    /// Ligne dans le fichier (si disponible)
    pub line: Option<u32>,
    /// Colonne dans le fichier (si disponible)
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

/// Résultat de validation d'un fichier JSON unique
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    /// Le JSON est-il valide selon le schema MCDOC ?
    pub is_valid: bool,
    /// Erreurs de validation détaillées
    pub errors: Vec<McDocError>,
    /// Dépendances registries extraites
    pub dependencies: Vec<McDocDependency>,
}

impl ValidationResult {
    /// Créer un résultat de validation réussie
    pub fn success(dependencies: Vec<McDocDependency>) -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            dependencies,
        }
    }
    
    /// Créer un résultat de validation échouée
    pub fn failure(errors: Vec<McDocError>) -> Self {
        Self {
            is_valid: false,
            errors,
            dependencies: Vec::new(),
        }
    }
    
    /// Ajouter une erreur au résultat
    pub fn add_error(&mut self, error: McDocError) {
        self.errors.push(error);
        self.is_valid = false;
    }
    
    /// Ajouter une dépendance au résultat
    pub fn add_dependency(&mut self, dependency: McDocDependency) {
        self.dependencies.push(dependency);
    }
}

/// Résultat d'analyse complète d'un datapack
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatapackResult {
    /// Nombre total de fichiers analysés
    pub total_files: usize,
    /// Nombre de fichiers valides
    pub valid_files: usize,
    /// Erreurs de validation par fichier
    pub errors: Vec<FileError>,
    /// Toutes les dépendances groupées par registry  
    #[serde(serialize_with = "serialize_fx_hashmap", deserialize_with = "deserialize_fx_hashmap")]
    pub dependencies: rustc_hash::FxHashMap<String, Vec<String>>,
    /// Temps de traitement total en millisecondes
    pub analysis_time_ms: u32,
}

/// Erreur dans un fichier spécifique du datapack
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileError {
    /// Chemin du fichier
    pub file_path: String,
    /// Erreur de validation
    pub error: McDocError,
}

impl DatapackResult {
    /// Créer un nouveau résultat vide
    pub fn new() -> Self {
        Self {
            total_files: 0,
            valid_files: 0,
            errors: Vec::new(),
            dependencies: rustc_hash::FxHashMap::default(),
            analysis_time_ms: 0,
        }
    }
    
    /// Ajouter les résultats d'un fichier
    pub fn add_file_result(&mut self, file_path: String, result: ValidationResult) {
        self.total_files += 1;
        
        if result.is_valid {
            self.valid_files += 1;
        }
        
        // Ajouter les erreurs
        for error in result.errors {
            self.errors.push(FileError {
                file_path: file_path.clone(),
                error,
            });
        }
        
        // Grouper les dépendances par registry
        for dependency in result.dependencies {
            self.dependencies
                .entry(dependency.registry_type)
                .or_default()
                .push(dependency.resource_location);
        }
    }
    
    /// Définir le temps d'analyse
    pub fn set_analysis_time(&mut self, time_ms: u32) {
        self.analysis_time_ms = time_ms;
    }
}

/// Version Minecraft - VERSION SIMPLIFIÉE (type alias)
/// La parsing complexe est gérée côté JavaScript selon spec
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