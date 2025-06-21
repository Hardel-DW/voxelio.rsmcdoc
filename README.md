# Voxel-RSMCDOC

Rust MCDOC parser for ultra-fast Minecraft datapack validation with webapp
integration via WASM.

## 🎯 What is MCDOC?

**MCDOC** is a DSL (Domain Specific Language) created by the Minecraft community
to precisely describe the JSON structure of datapacks/resourcepacks. It allows
to:

- **Validate** : Check that all JSONs comply with Minecraft specifications
- **Detect errors** : Incorrect types, violated constraints, invalid registries
- **Extract dependencies** : Used Resource Locations (items, blocks, recipes,
  etc.)

### MCDOC vs JSON Example

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

**Validation:** ✅ Correct types, `minecraft:diamond_sword` exists in registry
`item`.

## 🚀 Performance Objectives

### **REALISTIC** Performance Targets

| Datapack Size      | Parse MCDOC | Complete Validation | Total Pipeline |
| ------------------ | ----------- | ------------------- | -------------- |
| Small (100 files)  | <2ms        | <10ms               | **<10ms**      |
| Medium (500 files) | <8ms        | <50ms               | **<50ms**      |
| Large (1000 files) | <15ms       | <100ms              | **<100ms**     |

## 🔧 TypeScript/WASM API

```typescript
export class McDocValidator {
    // Setup : Loads MCDOC and resolves imports
    static async init(
        mcdocFiles: Record<string, string>,
    ): Promise<McDocValidator>;

    // Registry Discovery : Which registries are needed (FAST - cache)
    getRequiredRegistries(json: object, resourceType: string): string[];

    // Validation : With loaded registries
    validate(json: object, version?: string): ValidationResult;

    // Datapack Analysis : Parallel validation
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

## 📊 3-Step Workflow

### 1. **Registry Discovery** (Ultra-fast)

```typescript
// Local analysis - no network calls
const requiredRegistries = validator.getRequiredRegistries(
    recipeJson,
    "recipe",
);
// Result: ["item", "recipe_serializer", "crafting_book_category"]
```

### 2. **Validation** (With loaded registries)

```typescript
// To validate a single JSON file. Just need to provide the JSON
const result = validator.validate(recipeJson);
// Result: { isValid: true, errors: [], dependencies: [...] }
```

### 3. **Datapack Analysis** (Parallel validation)

```typescript
const result = await validator.analyzeDatapack(files);
// Result: { isValid: true, errors: [], dependencies: [...] }
```

## 📚 Documentation

| File                                                                 | Content                                   |
| -------------------------------------------------------------------- | ----------------------------------------- |
| [`docs/mcdoc-format.md`](docs/mcdoc-format.md)                       | Complete MCDOC syntax with examples       |
| [`docs/developpement-plan.md`](docs/developpement-plan.md)           | Development rules and optimizations       |
| [`docs/wasm-integration-plan.md`](docs/wasm-integration-plan.md)     | WASM architecture and TypeScript bindings |
| [`docs/webapp-usage-examples.md`](docs/webapp-usage-examples.md)     | React/Vue examples with Workers           |
| [`docs/examples-and-test-cases.md`](docs/examples-and-test-cases.md) | Realistic test cases and benchmarks       |

## 🎪 Technical Highlights

- **No hardcoding** : Registries and MCDOC 100% external via parameters
- **Modular MCDOC** : Imports with cycle resolution (topological sort)
- **Zero-copy parsing** : Lifetimes to avoid unnecessary allocations
- **Error recovery** : Continue parsing despite syntax errors
- **Optimized WASM** : <100KB bundle, ultra-fast performance
- **Breeze Integration** : Compatible with existing ecosystem

## 📦 Installation

```bash
# Webapp only (TypeScript/WASM)
npm install @voxel/rsmcdoc
```

## 🔬 TypeScript Usage

```typescript
import { McDocValidator } from "@voxel/rsmcdoc";

// 1. Initialize with MCDOC schemas
const validator = await McDocValidator.init(mcdocFiles);

// 2. Load Minecraft registries
validator.loadRegistries(registries, "1.21");

// 3. Validate a JSON
const result = validator.validate(recipeJson, "recipe");

// 4. Analyze a complete datapack
const datapackResult = await validator.analyzeDatapack(files);
```

---

**Realistic performance targets**, **clear architecture**, **complete MCDOC
syntax** - ready for production!
