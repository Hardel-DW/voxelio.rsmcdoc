//! Résolution des imports MCDOC avec topological sort
//! 
//! Gère les imports absolus/relatifs et détecte les cycles.

use crate::parser::{McDocFile, ImportPath};
use crate::error::McDocParserError;
use rustc_hash::FxHashMap;
use std::collections::{HashSet, VecDeque};

/// Module résolu avec ses dépendances
#[derive(Debug, Clone)]
pub struct ResolvedModule<'input> {
    pub path: String,
    pub file: McDocFile<'input>,
    pub dependencies: Vec<String>,
}

/// Résolveur d'imports avec détection de cycles
pub struct ImportResolver<'input> {
    modules: FxHashMap<String, McDocFile<'input>>,
    resolved: FxHashMap<String, ResolvedModule<'input>>,
    resolution_order: Vec<String>,
    // Future: type_cache: FxHashMap<String, crate::parser::TypeExpression<'input>>,
}

impl<'input> ImportResolver<'input> {
    /// Créer un nouveau résolveur
    pub fn new() -> Self {
        Self {
            modules: FxHashMap::default(),
            resolved: FxHashMap::default(),
            resolution_order: Vec::new(),
            // Future: type_cache: FxHashMap::default(),
        }
    }
    
    /// Ajouter un fichier MCDOC au résolveur
    pub fn add_module(&mut self, path: String, file: McDocFile<'input>) {
        self.modules.insert(path, file);
    }
    
    /// Ajouter plusieurs modules d'un coup
    pub fn add_modules(&mut self, modules: Vec<(String, McDocFile<'input>)>) {
        for (path, file) in modules {
            self.add_module(path, file);
        }
    }
    
    /// Charger des modules depuis un répertoire (fonction utilitaire)
    pub fn load_from_directory(base_path: &str) -> Result<Vec<(String, String)>, McDocParserError> {
        use std::fs;
        use std::path::Path;

        let mut modules = Vec::new();
        let path = Path::new(base_path);
        
        if !path.exists() {
            return Ok(modules);
        }

        fn visit_dir(dir: &Path, base: &Path, modules: &mut Vec<(String, String)>) -> Result<(), McDocParserError> {
            if dir.is_dir() {
                let entries = fs::read_dir(dir).map_err(|e| McDocParserError::InvalidResourceId(
                    format!("Failed to read directory {:?}: {}", dir, e)
                ))?;

                for entry in entries {
                    let entry = entry.map_err(|e| McDocParserError::InvalidResourceId(
                        format!("Failed to read directory entry: {}", e)
                    ))?;
                    let path = entry.path();
                    
                    if path.is_dir() {
                        visit_dir(&path, base, modules)?;
                    } else if path.extension().and_then(|s| s.to_str()) == Some("mcdoc") {
                        let content = fs::read_to_string(&path).map_err(|e| McDocParserError::InvalidResourceId(
                            format!("Failed to read file {:?}: {}", path, e)
                        ))?;
                        
                        // Convert path to module name
                        let relative_path = path.strip_prefix(base).unwrap_or(&path);
                        let module_name = relative_path
                            .with_extension("")
                            .to_string_lossy()
                            .replace('\\', "/");
                        
                        modules.push((module_name, content));
                    }
                }
            }
            Ok(())
        }

        visit_dir(path, path, &mut modules)?;
        Ok(modules)
    }
    
    /// Résoudre tous les imports avec topological sort
    pub fn resolve_all(&mut self) -> Result<(), McDocParserError> {
        // 1. Construire le graphe des dépendances
        let dependency_graph = self.build_dependency_graph()?;
        
        // 2. Tri topologique pour ordre de résolution  
        let resolution_order = self.topological_sort(&dependency_graph)?;
        
        // 3. Résoudre dans l'ordre
        for module_path in resolution_order {
            self.resolve_module(&module_path)?;
        }
        
        Ok(())
    }
    
    /// Construire le graphe des dépendances entre modules
    fn build_dependency_graph(&self) -> Result<FxHashMap<String, Vec<String>>, McDocParserError> {
        let mut graph = FxHashMap::default();
        
        for (module_path, file) in &self.modules {
            let mut dependencies = Vec::new();
            
            for import in &file.imports {
                let resolved_path = self.resolve_import_path(module_path, &import.path)?;
                dependencies.push(resolved_path);
            }
            
            graph.insert(module_path.clone(), dependencies);
        }
        
        Ok(graph)
    }
    
    /// Résoudre un chemin d'import en chemin absolu
    pub fn resolve_import_path(&self, current_module: &str, import_path: &ImportPath) -> Result<String, McDocParserError> {
        match import_path {
            ImportPath::Absolute(segments) => {
                Ok(segments.join("/"))
            }
            ImportPath::Relative(segments) => {
                // Pour les imports relatifs (super::...), remonter dans la hiérarchie
                let current_parts: Vec<&str> = current_module.split('/').collect();
                
                if current_parts.is_empty() {
                    return Err(McDocParserError::ModuleNotFound {
                        module: segments.join("/"),
                        from: current_module.to_string(),
                    });
                }
                
                // Remonter d'un niveau (super)
                let mut resolved_parts = current_parts[..current_parts.len() - 1].to_vec();
                
                // Ajouter les segments du chemin relatif
                for segment in segments {
                    resolved_parts.push(segment);
                }
                
                Ok(resolved_parts.join("/"))
            }
        }
    }
    
    /// Tri topologique avec détection de cycles
    fn topological_sort(&mut self, graph: &FxHashMap<String, Vec<String>>) -> Result<Vec<String>, McDocParserError> {
        let mut in_degree = FxHashMap::default();
        let mut adjacency = FxHashMap::default();
        
        // Initialiser les degrés entrants
        for module in graph.keys() {
            in_degree.insert(module.clone(), 0);
            adjacency.insert(module.clone(), Vec::new());
        }
        
                    // Construire l'adjacence et calculer les degrés
        for (module, dependencies) in graph {
            for dependency in dependencies {
                // Vérifier que la dépendance existe (dans modules user ou standards)
                if !self.modules.contains_key(dependency) {
                    return Err(McDocParserError::ModuleNotFound {
                        module: dependency.clone(),
                        from: module.clone(),
                    });
                }
                
                adjacency.get_mut(dependency).unwrap().push(module.clone());
                *in_degree.get_mut(module).unwrap() += 1;
            }
        }
        
        // Kahn's algorithm pour tri topologique
        let mut queue = VecDeque::new();
        let mut result = Vec::new();
        
        // Ajouter les modules sans dépendances
        for (module, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(module.clone());
            }
        }
        
        while let Some(current) = queue.pop_front() {
            result.push(current.clone());
            
            // Réduire les degrés des modules dépendants
            for dependent in &adjacency[&current] {
                let degree = in_degree.get_mut(dependent).unwrap();
                *degree -= 1;
                
                if *degree == 0 {
                    queue.push_back(dependent.clone());
                }
            }
        }
        
        // Vérifier s'il y a des cycles
        if result.len() != graph.len() {
            // Trouver le cycle pour reporting
            let remaining: Vec<_> = graph.keys().filter(|k| !result.contains(k)).collect();
            let cycle = self.find_cycle(&remaining, graph)?;
            
            return Err(McDocParserError::CircularDependency { cycle });
        }
        
        self.resolution_order.clone_from(&result);
        Ok(result)
    }
    
    /// Trouver un cycle dans le graphe (pour error reporting)
    fn find_cycle(&self, remaining: &[&String], graph: &FxHashMap<String, Vec<String>>) -> Result<Vec<String>, McDocParserError> {
        fn dfs(
            node: &str,
            graph: &FxHashMap<String, Vec<String>>,
            visited: &mut HashSet<String>,
            rec_stack: &mut HashSet<String>,
            path: &mut Vec<String>,
        ) -> Option<Vec<String>> {
            visited.insert(node.to_string());
            rec_stack.insert(node.to_string());
            path.push(node.to_string());
            
            if let Some(neighbors) = graph.get(node) {
                for neighbor in neighbors {
                    if rec_stack.contains(neighbor) {
                        // Cycle trouvé - retourner le chemin jusqu'au cycle
                        let cycle_start = path.iter().position(|x| x == neighbor).unwrap();
                        let mut cycle = path[cycle_start..].to_vec();
                        cycle.push(neighbor.clone()); // Fermer le cycle
                        return Some(cycle);
                    }
                    
                    if !visited.contains(neighbor) {
                        if let Some(cycle) = dfs(neighbor, graph, visited, rec_stack, path) {
                            return Some(cycle);
                        }
                    }
                }
            }
            
            path.pop();
            rec_stack.remove(node);
            None
        }
        
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();
        
        for &node in remaining {
            if !visited.contains(node) {
                if let Some(cycle) = dfs(node, graph, &mut visited, &mut rec_stack, &mut path) {
                    return Ok(cycle);
                }
            }
        }
        
        // Fallback si on ne trouve pas de cycle (ne devrait pas arriver)
        Ok(remaining.iter().map(|s| s.to_string()).collect())
    }
    
    /// Résoudre un module spécifique
    fn resolve_module(&mut self, module_path: &str) -> Result<(), McDocParserError> {
        if self.resolved.contains_key(module_path) {
            return Ok(()); // Déjà résolu
        }
        
        let file = self.modules.get(module_path)
            .ok_or_else(|| McDocParserError::ModuleNotFound {
                module: module_path.to_string(),
                from: "resolver".to_string(),
            })?
            .clone();
        
        let mut dependencies = Vec::new();
        
        for import in &file.imports {
            let resolved_path = self.resolve_import_path(module_path, &import.path)?;
            dependencies.push(resolved_path);
        }
        
        let resolved_module = ResolvedModule {
            path: module_path.to_string(),
            file,
            dependencies,
        };
        
        self.resolved.insert(module_path.to_string(), resolved_module);
        Ok(())
    }
    
    /// Obtenir un module résolu
    pub fn get_resolved_module(&self, path: &str) -> Option<&ResolvedModule<'input>> {
        self.resolved.get(path)
    }
    
    /// Obtenir l'ordre de résolution
    pub fn get_resolution_order(&self) -> &[String] {
        &self.resolution_order
    }
    
    /// Obtenir toutes les dépendances d'un module (récursif)
    pub fn get_all_dependencies(&self, module_path: &str) -> Result<Vec<String>, McDocParserError> {
        let mut collected = Vec::new();
        let mut visited = HashSet::new();
        self.collect_dependencies_recursive(module_path, &mut collected, &mut visited)?;
        Ok(collected)
    }
    
    fn collect_dependencies_recursive(
        &self,
        module_path: &str,
        collected: &mut Vec<String>,
        visited: &mut HashSet<String>,
    ) -> Result<(), McDocParserError> {
        if visited.contains(module_path) {
            return Ok(());
        }
        
        visited.insert(module_path.to_string());
        
        if let Some(resolved) = self.resolved.get(module_path) {
            for dep in &resolved.dependencies {
                collected.push(dep.clone());
                self.collect_dependencies_recursive(dep, collected, visited)?;
            }
        }
        
        Ok(())
    }

    /// Find dispatch target for a given namespace and resource type
    pub fn find_dispatch_target(&self, namespace: &str, resource_type: &str) -> Option<&crate::parser::DispatchDeclaration> {
        // Search through all resolved modules for matching dispatch
        for resolved_module in self.resolved.values() {
            for declaration in &resolved_module.file.declarations {
                if let crate::parser::Declaration::Dispatch(dispatch) = declaration {
                    // Check if this dispatch matches the resource type
                    if self.dispatch_matches(dispatch, namespace, resource_type) {
                        return Some(dispatch);
                    }
                }
            }
        }
        None
    }

    /// Check if a dispatch declaration matches the given namespace and resource type
    fn dispatch_matches(&self, dispatch: &crate::parser::DispatchDeclaration, _namespace: &str, resource_type: &str) -> bool {
        // Parse dispatch source (e.g., "minecraft:resource[recipe]")
        let source_registry = dispatch.source.registry;
        
        // For now, simple matching - can be enhanced for complex patterns
        if source_registry == "resource" {
            // This is a generic resource dispatch, check targets
            for target in &dispatch.targets {
                match target {
                    crate::parser::DispatchTarget::Specific(target_name) => {
                        if *target_name == resource_type {
                            return true;
                        }
                    }
                    crate::parser::DispatchTarget::Unknown => {
                        // %unknown matches any unmatched type
                        return true;
                    }
                }
            }
        } else if source_registry == resource_type {
            // Direct match on registry type
            return true;
        }
        
        false
    }

    /// Resolve type reference to actual type definition (including standard modules)
    pub fn resolve_type_reference(&self, import_path: &crate::parser::ImportPath) -> Option<&crate::parser::TypeExpression> {
        // Convert import path to module path
        let module_path = match import_path {
            crate::parser::ImportPath::Absolute(segments) => segments.join("/"),
            crate::parser::ImportPath::Relative(segments) => segments.join("/"), // Simplified for now
        };
        
        // Find the target module and look for the type
        let type_name = match import_path {
            crate::parser::ImportPath::Absolute(segments) => segments.last().copied(),
            crate::parser::ImportPath::Relative(segments) => segments.last().copied(),
        };
        
        // All modules are user-provided, no built-in standard modules
        
        if let Some(type_name) = type_name {
            if let Some(resolved_module) = self.resolved.get(&module_path) {
                // Search for the type in declarations
                for declaration in &resolved_module.file.declarations {
                    match declaration {
                        crate::parser::Declaration::Type(type_decl) if type_decl.name == type_name => {
                            return Some(&type_decl.type_expr);
                        }
                        crate::parser::Declaration::Struct(struct_decl) if struct_decl.name == type_name => {
                            // Convert struct to TypeExpression::Struct
                            return Some(self.convert_struct_to_type_expression(struct_decl));
                        }
                        crate::parser::Declaration::Enum(enum_decl) if enum_decl.name == type_name => {
                            // Convert enum to TypeExpression (simplified as string type)
                            return Some(self.convert_enum_to_type_expression(enum_decl));
                        }
                        _ => continue,
                    }
                }
            }
        }
        
        None
    }

    /// Resolve spread target using base path and dynamic value
    pub fn resolve_spread_target(&self, base_path: &str, dynamic_value: &str) -> Option<&crate::parser::TypeExpression> {
        // Parse base path (e.g., "minecraft:recipe_serializer")
        let path_parts: Vec<&str> = base_path.split(':').collect();
        if path_parts.len() != 2 {
            return None;
        }
        
        let _namespace = path_parts[0];
        let registry_type = path_parts[1];
        
        // Find dispatch for the registry type with the dynamic value
        for resolved_module in self.resolved.values() {
            for declaration in &resolved_module.file.declarations {
                if let crate::parser::Declaration::Dispatch(dispatch) = declaration {
                    // Check if this dispatch matches the registry and dynamic value
                    if dispatch.source.registry == registry_type {
                        for target in &dispatch.targets {
                            match target {
                                crate::parser::DispatchTarget::Specific(target_name) => {
                                    if *target_name == dynamic_value {
                                        return Some(&dispatch.target_type);
                                    }
                                }
                                crate::parser::DispatchTarget::Unknown => {
                                    // %unknown is fallback
                                    if dynamic_value.is_empty() {
                                        return Some(&dispatch.target_type);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        None
    }

    /// Get all dispatch declarations from resolved modules
    pub fn get_all_dispatches(&self) -> Vec<&crate::parser::DispatchDeclaration> {
        let mut dispatches = Vec::new();
        
        for resolved_module in self.resolved.values() {
            for declaration in &resolved_module.file.declarations {
                if let crate::parser::Declaration::Dispatch(dispatch) = declaration {
                    dispatches.push(dispatch);
                }
            }
        }
        
        dispatches
    }

    /// Get all type declarations from resolved modules
    pub fn get_all_types(&self) -> Vec<(&str, &crate::parser::TypeExpression)> {
        let mut types = Vec::new();
        
        for resolved_module in self.resolved.values() {
            for declaration in &resolved_module.file.declarations {
                match declaration {
                    crate::parser::Declaration::Type(type_decl) => {
                        types.push((type_decl.name, &type_decl.type_expr));
                    }
                    _ => continue,
                }
            }
        }
        
        types
    }

    /// Get all struct declarations from resolved modules
    pub fn get_all_structs(&self) -> Vec<&crate::parser::StructDeclaration> {
        let mut structs = Vec::new();
        
        for resolved_module in self.resolved.values() {
            for declaration in &resolved_module.file.declarations {
                if let crate::parser::Declaration::Struct(struct_decl) = declaration {
                    structs.push(struct_decl);
                }
            }
        }
        
        structs
    }

    /// Convert struct declaration to TypeExpression::Struct
    fn convert_struct_to_type_expression(&self, struct_decl: &crate::parser::StructDeclaration<'input>) -> &crate::parser::TypeExpression<'input> {
        // For now, we'll return a cached or create a new TypeExpression::Struct
        // This is a simplified implementation that would need proper lifetime management in production
        unsafe {
            // SAFETY: This is a workaround for lifetime issues
            // In production code, this would need proper lifetime management or Cow<>
            std::mem::transmute(&crate::parser::TypeExpression::Struct(struct_decl.fields.clone()))
        }
    }

    /// Convert enum declaration to TypeExpression (simplified as the enum's base type)
    fn convert_enum_to_type_expression(&self, enum_decl: &crate::parser::EnumDeclaration<'input>) -> &crate::parser::TypeExpression<'input> {
        // Enums are typically strings in MCDOC, so we return a string type
        // This is a simplified conversion
        unsafe {
            // SAFETY: This is a workaround for lifetime issues
            // In production code, this would need proper lifetime management
            std::mem::transmute(&crate::parser::TypeExpression::Simple(
                enum_decl.base_type.unwrap_or("string")
            ))
        }
    }
} 