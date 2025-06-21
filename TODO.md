# TODO - Problèmes Identifiés et Actions Requises

## 📊 STATUT ACTUEL WASM

**Localisation** : `target/wasm32-unknown-unknown/release/`

- ✅ **WASM généré** : Compilation réussie
- ❌ **Taille excessive** : 354KB vs <100KB requis (3.5x trop gros)
- ❌ **wasm-opt échoue** : Features WASM incompatibles
- ❌ **Pas de wasm-pack** : Installation échoue (dlltool.exe manquant)

**Analyse des tailles** :

- `voxel_rsmcdoc.wasm` : 411KB (build standard)
- `voxel_rsmcdoc_optimized.wasm` : 357KB (optimisations Cargo)
- `voxel_rsmcdoc_ultra.wasm` : 354KB (**meilleur actuel**)

## 🚨 CRITIQUE - Action Immédiate Requise

### 1. Bundle WASM Trop Volumineux

- **Problème** : WASM actuel 354KB vs objectif <100KB (3.5x trop gros)
- **Localisation** : `target/wasm32-unknown-unknown/release/`
  - `voxel_rsmcdoc.wasm` : 411KB (standard)
  - `voxel_rsmcdoc_optimized.wasm` : 357KB (optimisé)
  - `voxel_rsmcdoc_ultra.wasm` : 354KB (meilleur)
- **Impact** : Violation specs performance - bundle trop gros pour production
  web
- **Actions** :
  - [ ] Configurer optimisations taille maximale dans Cargo.toml
  - [ ] Tester `opt-level = "z"` pour toutes dépendances
  - [ ] ⚠️ wasm-opt échoue : features incompatibles (`trunc_sat`, `bulk-memory`)
  - [ ] Installer et utiliser `wasm-pack` au lieu de compilation directe
  - [ ] Identifier et éliminer dépendances lourdes (serde_json = 180KB+)
  - [ ] Tester compression gzip pour taille réelle (objectif <100KB compressé)
  - [ ] Évaluer retrait de features non-essentielles

### 2. Unsafe Transmute Dangereux

- **Fichier** : `src/wasm.rs:60`
- **Code problématique** : `unsafe { std::mem::transmute(ast) }`
- **Risque** : Undefined behavior, corruption mémoire
- **Solution** :
  - [ ] Remplacer par `Arc<T>` ou `Rc<T>` pour partage sûr
  - [ ] Ou utiliser `Box::leak()` si nécessaire (moins safe)
  - [ ] Tester avec différentes approches de lifetime management
  - [ ] Documenter la solution choisie

## ⚠️ IMPORTANT - À Corriger Rapidement

### 3. Audit Dépendances WASM

- **Problème** : Dépendances potentiellement surdimensionnées
- **Actions** :
  - [ ] Analyser `serde_json` - tester `default-features = false`
  - [ ] Vérifier si `memchr` est dupliquée (incluse dans serde_json)
  - [ ] Évaluer retrait de `console_error_panic_hook` en production
  - [ ] Tester impact `wasm-bindgen-futures` (optionnel?)
  - [ ] Mesurer taille bundle après chaque retrait

### 4. Optimisations Cargo.toml Manquantes

- **Actions** :
  - [ ] Ajouter `strip = "symbols"` pour toutes dépendances
  - [ ] Configurer `panic = "abort"` global
  - [ ] Tester `codegen-units = 1` pour LTO maximal
  - [ ] Évaluer `overflow-checks = false` en release

## 📈 OPTIMISATIONS - Performance et Qualité

### 5. Documentation Technique

- **Actions** :
  - [ ] Documenter stratégie lifetime management
  - [ ] Expliquer choix FxHashMap vs HashMap
  - [ ] Documenter pipeline validation complet
  - [ ] Ajouter benchmarks taille bundle dans CI

### 6. Tests de Performance

- **Actions** :
  - [ ] Ajouter tests benchmark avec Criterion
  - [ ] Mesurer temps parsing fichiers réels MCDOC
  - [ ] Valider objectifs <10ms, <50ms, <100ms selon taille
  - [ ] Test memory usage avec différents datasets

### 7. WASM-Specific Optimizations

- **Actions** :
  - [ ] Tester `wee_alloc` allocator pour réduire taille
  - [ ] Analyser avec `twiggy` pour identifier gros symbols
  - [ ] Évaluer `wasm-opt` post-compilation
  - [ ] Tester différents targets WASM (web vs bundler)

## 🔧 AMÉLIORATIONS TECHNIQUES

### 8. Error Handling Robustesse

- **Actions** :
  - [ ] Ajouter timeout pour `analyze_datapack` (éviter blocage)
  - [ ] Améliorer messages d'erreur pour debugging WASM
  - [ ] Tester error recovery avec fichiers MCDOC corrompus
  - [ ] Valider sérialisation erreurs JS ↔ Rust

### 9. API TypeScript Complétude

- **Actions** :
  - [ ] Générer bindings TypeScript automatiques
  - [ ] Valider conformité exacte avec specs `developpement-plan.md`
  - [ ] Tester tous cas d'usage de `webapp-usage-examples.md`
  - [ ] Documenter exemples d'intégration React/Vue

### 10. Production Readiness

- **Actions** :
  - [ ] Configurer CI/CD pour build WASM automatique
  - [ ] Setup tests cross-platform (Windows/Linux/macOS)
  - [ ] Valider compatibilité navigateurs modernes
  - [ ] Créer exemples démo complets

## 📋 VALIDATION FINALE

### Critères de Succès

- [ ] Bundle WASM < 100KB compressé (actuellement 354KB = 3.5x trop gros)
- [ ] Aucun `unsafe` code ou justification documentée
- [ ] Tous tests passent sans warnings
- [ ] Performance conforme aux specs (10ms/50ms/100ms)
- [ ] API TypeScript 100% fonctionnelle
- [ ] Documentation complète utilisateur/développeur

### Tests de Régression

- [ ] Parsing MCDOC complexe (loot tables, recipes)
- [ ] Validation JSON avec registries multiples
- [ ] Extract dependencies précises et complètes
- [ ] Gestion erreurs robuste et informative
- [ ] Integration datapack réels (vanilla + mods)

---

**Note** : Prioriser les items CRITIQUE avant toute mise en production. Les
autres peuvent être traités itérativement selon les besoins utilisateur.
