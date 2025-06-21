# Voxel-RSMCDOC

Parser MCDOC en Rust pour validation ultra-rapide de datapacks Minecraft avec
intégration webapp via WASM.

## 🎯 Qu'est-ce que MCDOC ?

**MCDOC** est un DSL (Domain Specific Language) créé par la communauté Minecraft
pour décrire précisément la structure des fichiers JSON de
datapacks/resourcepacks. Il permet de :

- **Valider** : Vérifier que tous les JSON respectent les spécifications
  Minecraft
- **Détecter erreurs** : Types incorrects, contraintes violées, registres
  invalides
- **Extraire dépendances** : Resource Locations utilisées (items, blocks,
  recipes, etc.)

### Exemple MCDOC vs JSON

**MCDOC Schema:**

```mcdoc
dispatch minecraft:resource[recipe] to struct Recipe {
    type: #[id="recipe_serializer"] string,
    result: #[id="item"] string,
    ingredients: [#[id="item"] string],
}
```

**JSON Datapack:**

```json
{
    "type": "minecraft:crafting_shaped",
    "result": "minecraft:diamond_sword",
    "ingredients": ["minecraft:diamond", "minecraft:stick"]
}
```

**Validation:** ✅ Types corrects, `minecraft:diamond_sword` existe dans
registry `item`.

## 🚀 Objectifs Performance

### Performances **RÉALISTES** visées

| Taille Datapack       | Parse MCDOC | Validation Complète | Total Pipeline |
| --------------------- | ----------- | ------------------- | -------------- |
| Small (100 fichiers)  | <2ms        | <10ms               | **<10ms**      |
| Medium (500 fichiers) | <8ms        | <50ms               | **<50ms**      |
| Large (1000 fichiers) | <15ms       | <100ms              | **<100ms**     |

## 🔧 API TypeScript/WASM

```typescript
export class McDocValidator {
    // Setup : Charge MCDOC et résout imports
    static async init(
        mcdocFiles: Record<string, string>,
    ): Promise<McDocValidator>;

    // Registry Discovery : Quels registres nécessaires (RAPIDE - cache)
    getRequiredRegistries(json: object, resourceType: string): string[];

    // Validation : Avec registres chargés
    validate(json: object, version?: string): ValidationResult;

    // Analyse Datapack : Validation parallèle
    async analyzeDatapack(
        files: Record<string, Uint8Array>,
        version?: string,
    ): Promise<DatapackResult>;
}

interface ValidationResult {
    isValid: boolean;
    errors: McDocError[];
    dependencies: McDocDependency[];
}
```

## 📊 Workflow en 3 étapes

### 1. **Registry Discovery** (Ultra-rapide)

```typescript
// Analyse locale - pas de network calls
const requiredRegistries = validator.getRequiredRegistries(
    recipeJson,
    "recipe",
);
// Result: ["item", "recipe_serializer", "crafting_book_category"]
```

### 2. **Validation** (Avec registres chargés)

```typescript
// Pour valider un seul fichier JSON. Juste besoin de fournir le JSON
const result = validator.validate(recipeJson);
// Result: { isValid: true, errors: [], dependencies: [...] }
```

### 3. **Analyse Datapack** (Validation parallèle)

```typescript
const result = await validator.analyzeDatapack(files);
// Result: { isValid: true, errors: [], dependencies: [...] }
```

## 📚 Documentation

| Fichier                                                              | Contenu                                          |
| -------------------------------------------------------------------- | ------------------------------------------------ |
| [`docs/mcdoc-format.md`](docs/mcdoc-format.md)                       | Syntaxe MCDOC complète avec exemples             |
| [`docs/developpement-plan.md`](docs/developpement-plan.md)           | Les régles de développement et les optimisations |
| [`docs/wasm-integration-plan.md`](docs/wasm-integration-plan.md)     | Architecture WASM et bindings TypeScript         |
| [`docs/webapp-usage-examples.md`](docs/webapp-usage-examples.md)     | Exemples React/Vue avec Workers                  |
| [`docs/examples-and-test-cases.md`](docs/examples-and-test-cases.md) | Cas de test et benchmarks réalistes              |

## 🎪 Points forts techniques

- **Aucun hardcoding** : Registres et MCDOC 100% externes via paramètres
- **MCDOC modulaires** : Imports avec résolution de cycles (topological sort)
- **Zero-copy parsing** : Lifetimes pour éviter allocations inutiles
- **Error recovery** : Continue parsing malgré erreurs syntaxiques
- **WASM optimisé** : <100KB bundle, performance ultra-rapide
- **Integration Breeze** : Compatible avec écosystème existant

## 📦 Installation

```bash
# Webapp uniquement (TypeScript/WASM)
npm install @voxel/rsmcdoc
```

## 🔬 Usage TypeScript

```typescript
import { McDocValidator } from "@voxel/rsmcdoc";

// 1. Initialiser avec schemas MCDOC
const validator = await McDocValidator.init(mcdocFiles);

// 2. Charger les registries Minecraft
validator.loadRegistries(registries, "1.21");

// 3. Valider un JSON
const result = validator.validate(recipeJson, "recipe");

// 4. Analyser un datapack complet
const datapackResult = await validator.analyzeDatapack(files);
```

---

**Performance targets réalistes**, **architecture claire**, **MCDOC syntax
complète** - ready for production!
