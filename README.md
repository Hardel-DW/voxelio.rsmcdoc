# Voxel-RSMCDOC

Rust MCDOC parser for ultra-fast Minecraft datapack validation.

```bash
pnpm install @voxelio/rsmcdoc
```

## What is MCDOC?

**MCDOC** is a DSL (Domain Specific Language) created by the Minecraft community
to precisely describe the JSON structure of datapacks/resourcepacks. It allows
to:

- **Validate** : Check that all JSONs comply with Minecraft specifications
- **Detect errors** : Incorrect types, violated constraints, invalid registries
- **Extract dependencies** : Used Resource Locations (items, blocks, recipes,
  etc.)

## Technical Highlights

- **No hardcoding** : Registries and MCDOC 100% external via parameters
- **Modular MCDOC** : Imports with cycle resolution (topological sort)
- **Zero-copy parsing** : Lifetimes to avoid unnecessary allocations
- **Error recovery** : Continue parsing despite syntax errors
- **Optimized WASM** : <100KB bundle, ultra-fast performance
- **Breeze Integration** : Compatible with existing ecosystem

## Performance

| Datapack Size      | Parse MCDOC | Complete Validation | Total Pipeline |
| ------------------ | ----------- | ------------------- | -------------- |
| Small (100 files)  | <2ms        | <10ms               | **<10ms**      |
| Medium (500 files) | <8ms        | <50ms               | **<50ms**      |
| Large (1000 files) | <15ms       | <100ms              | **<100ms**     |

## Documentation

| File                                                                 | Content                                   |
| -------------------------------------------------------------------- | ----------------------------------------- |
| [`docs/mcdoc-format.md`](docs/mcdoc-format.md)                       | Complete MCDOC syntax with examples       |
| [`docs/developpement-plan.md`](docs/developpement-plan.md)           | Development rules and optimizations       |
| [`docs/wasm-integration-plan.md`](docs/wasm-integration-plan.md)     | WASM architecture and TypeScript bindings |
| [`docs/webapp-usage-examples.md`](docs/webapp-usage-examples.md)     | React/Vue examples with Workers           |
| [`docs/examples-and-test-cases.md`](docs/examples-and-test-cases.md) | Realistic test cases and benchmarks       |

## TypeScript Usage

```typescript
import { DatapackValidator } from "@voxel/rsmcdoc";

// 1. Initialize with registries and MCDOC schemas
const validator = await DatapackValidator.init(registries, mcdocFiles, "1.21");

// 2. Validate a single JSON file
const result = await validator.validate(recipeJson, "minecraft:recipe");

// 3. Analyze a complete datapack
const datapackResult = await validator.analyzeDatapack(files);
```

<details>
<summary>Typescript API</summary>

```typescript
export class DatapackValidator {
    /**
     * Initializes the validator with Minecraft registries and MCDOC schemas.
     * @param registries A map of registry names to their content (e.g., {"minecraft:item": ["minecraft:stone", "minecraft:diamond"]}).
     * @param mcdocFiles A map of MCDOC file names to their string content.
     * @param version The Minecraft version to validate against (e.g., "1.21").
     */
    static async init(
        registries: Record<string, any>,
        mcdocFiles: Record<string, string>,
        version: string,
    ): Promise<DatapackValidator>;

    /**
     * Validates a single JSON object against a specific MCDOC resource type.
     * @param json The JSON object to validate.
     * @param resourceType The type of resource (e.g., "minecraft:recipe", "minecraft:loot_table").
     * @param version Optional Minecraft version to override the one from init.
     */
    async validate(
        json: object,
        resourceType: string,
    ): Promise<ValidationResult>;

    /**
     * Analyzes an entire datapack provided as a map of file paths to their byte content.
     * @param files A map of file paths to their content as Uint8Array.
     */
    async analyzeDatapack(
        files: Record<string, Uint8Array>,
    ): Promise<DatapackResult>;
}

interface ValidationResult {
    isValid: boolean;
    errors: McDocError[];
    dependencies: McDocDependency[];
}
```

Mapping example between JSON file type and ResourceType for JSON Validation:

| JSON file type | resourceType            | Corresponding MCDOC schema               |
| -------------- | ----------------------- | ---------------------------------------- |
| Recettes       | "minecraft:recipe"      | dispatch minecraft:resource[recipe]      |
| Loot Tables    | "minecraft:loot_table"  | dispatch minecraft:resource[loot_table]  |
| Advancements   | "minecraft:advancement" | dispatch minecraft:resource[advancement] |
| Tags           | "minecraft:tag"         | dispatch minecraft:resource[tag]         |
| Enchantements  | "minecraft:enchantment" | dispatch minecraft:resource[enchantment] |
| Damage Types   | "minecraft:damage_type" | dispatch minecraft:resource[damage_type] |
| Chat Types     | "minecraft:chat_type"   | dispatch minecraft:resource[chat_type]   |

</details>

## How to Bundle WASM

You can use the ps1 to bundle the package.

```bash
# 1. Build WASM module and generate JS bindings
cargo build --target wasm32-unknown-unknown --no-default-features --features wasm --release
wasm-bindgen --out-dir package --web --typescript target/wasm32-unknown-unknown/release/voxel_rsmcdoc.wasm
```

### Prerequisites

```bash
# Install Rust WASM target and wasm-bindgen-cli (if not already installed)
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
```
