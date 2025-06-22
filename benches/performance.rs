use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::collections::HashMap;
use voxel_rsmcdoc::validator::McDocValidator;

// Sample MCDOC content for benchmarks
const SIMPLE_MCDOC: &str = r#"
dispatch minecraft:resource[recipe] to struct Recipe {
    type: #[id="recipe_serializer"] string,
    result: #[id="item"] string,
}
"#;

const COMPLEX_MCDOC: &str = r#"
use ::java::world::item::ItemStack

dispatch minecraft:resource[recipe] to struct Recipe {
    type: #[id="recipe_serializer"] string,
    ...minecraft:recipe_serializer[[type]],
}

dispatch minecraft:recipe_serializer[crafting_shaped] to struct CraftingShaped {
    group?: string,
    pattern: [string @ 1..3] @ 1..3,
    key: struct {
        [string]: Ingredient,
    },
    result: ItemStack,
}

type Ingredient = (
    #[id="item"] string |
    struct {
        item: #[id="item"] string,
        count?: int @ 1..64,
    } |
)
"#;

// Sample JSON data for validation
fn get_simple_json() -> serde_json::Value {
    serde_json::json!({
        "type": "minecraft:crafting_shaped",
        "result": "minecraft:bread"
    })
}

fn get_complex_json() -> serde_json::Value {
    serde_json::json!({
        "type": "minecraft:crafting_shaped",
        "pattern": [
            "WWW",
            "   ",
            "   "
        ],
        "key": {
            "W": {
                "item": "minecraft:wheat"
            }
        },
        "result": {
            "item": "minecraft:bread",
            "count": 1
        }
    })
}

fn get_registries() -> HashMap<String, serde_json::Value> {
    let mut registries = HashMap::new();
    
    // Sample items registry
    registries.insert("item".to_string(), serde_json::json!([
        "minecraft:wheat",
        "minecraft:bread",
        "minecraft:diamond_sword",
        "minecraft:iron_ingot"
    ]));
    
    // Sample recipe serializers
    registries.insert("recipe_serializer".to_string(), serde_json::json!([
        "minecraft:crafting_shaped",
        "minecraft:crafting_shapeless",
        "minecraft:smelting"
    ]));
    
    registries
}

fn generate_datapack_files(size: usize) -> HashMap<String, Vec<u8>> {
    let mut files = HashMap::new();
    let recipe_json = serde_json::to_vec(&get_complex_json()).unwrap();
    
    for i in 0..size {
        files.insert(
            format!("data/test/recipes/recipe_{}.json", i),
            recipe_json.clone()
        );
    }
    
    files
}

// Benchmark: MCDOC parsing performance
fn bench_mcdoc_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("mcdoc_parsing");
    
    group.bench_function("simple_mcdoc", |b| {
        b.iter(|| {
            voxel_rsmcdoc::parse_mcdoc(black_box(SIMPLE_MCDOC))
        })
    });
    
    group.bench_function("complex_mcdoc", |b| {
        b.iter(|| {
            voxel_rsmcdoc::parse_mcdoc(black_box(COMPLEX_MCDOC))
        })
    });
    
    group.finish();
}

// Benchmark: Validator initialization and loading
fn bench_validator_setup(c: &mut Criterion) {
    let mut group = c.benchmark_group("validator_setup");
    
    group.bench_function("create_validator", |b| {
        b.iter(|| {
            McDocValidator::new()
        })
    });
    
    group.bench_function("load_mcdoc", |b| {
        let mut validator = McDocValidator::new();
        let mut mcdoc_files = HashMap::new();
        mcdoc_files.insert("recipe.mcdoc".to_string(), COMPLEX_MCDOC.to_string());
        
        b.iter(|| {
            let mut v = McDocValidator::new();
            for (name, content) in &mcdoc_files {
                let ast = voxel_rsmcdoc::parse_mcdoc(content).unwrap();
                let static_ast = Box::leak(Box::new(ast));
                v.load_parsed_mcdoc(name.clone(), static_ast).unwrap();
            }
            black_box(v)
        })
    });
    
    group.bench_function("load_registries", |b| {
        let mut validator = McDocValidator::new();
        let registries = get_registries();
        
        b.iter(|| {
            let mut v = McDocValidator::new();
            for (name, data) in &registries {
                v.load_registry(name.clone(), "1.21".to_string(), data).unwrap();
            }
            black_box(v)
        })
    });
    
    group.finish();
}

// Benchmark: JSON validation performance (core objective)
fn bench_json_validation(c: &mut Criterion) {
    // Setup validator with MCDOC and registries
    let mut validator = McDocValidator::new();
    let ast = voxel_rsmcdoc::parse_mcdoc(COMPLEX_MCDOC).unwrap();
    let static_ast = Box::leak(Box::new(ast));
    validator.load_parsed_mcdoc("recipe.mcdoc".to_string(), static_ast).unwrap();
    
    let registries = get_registries();
    for (name, data) in registries {
        validator.load_registry(name, "1.21".to_string(), &data).unwrap();
    }
    
    let mut group = c.benchmark_group("json_validation");
    
    group.bench_function("simple_json", |b| {
        let json = get_simple_json();
        b.iter(|| {
            validator.validate_json(black_box(&json), black_box("recipe"))
        })
    });
    
    group.bench_function("complex_json", |b| {
        let json = get_complex_json();
        b.iter(|| {
            validator.validate_json(black_box(&json), black_box("recipe"))
        })
    });
    
    group.finish();
}

// Benchmark: Datapack analysis (performance targets)
fn bench_datapack_analysis(c: &mut Criterion) {
    // Setup validator
    let mut validator = McDocValidator::new();
    let ast = voxel_rsmcdoc::parse_mcdoc(COMPLEX_MCDOC).unwrap();
    let static_ast = Box::leak(Box::new(ast));
    validator.load_parsed_mcdoc("recipe.mcdoc".to_string(), static_ast).unwrap();
    
    let registries = get_registries();
    for (name, data) in registries {
        validator.load_registry(name, "1.21".to_string(), &data).unwrap();
    }
    
    let mut group = c.benchmark_group("datapack_analysis");
    
    // Target: Small datapack <10ms (100 files, 2MB)
    let small_files = generate_datapack_files(100);
    group.bench_with_input(
        BenchmarkId::new("small_datapack", "100_files"),
        &small_files,
        |b, files| {
            b.iter(|| {
                let mut result = voxel_rsmcdoc::types::DatapackResult::new();
                for (path, data) in files {
                    let json: serde_json::Value = serde_json::from_slice(data).unwrap();
                    let validation = validator.validate_json(&json, "recipe");
                    result.add_file_result(path.clone(), validation);
                }
                black_box(result)
            })
        },
    );
    
    // Target: Medium datapack <50ms (500 files, 10MB)  
    let medium_files = generate_datapack_files(500);
    group.bench_with_input(
        BenchmarkId::new("medium_datapack", "500_files"),
        &medium_files,
        |b, files| {
            b.iter(|| {
                let mut result = voxel_rsmcdoc::types::DatapackResult::new();
                for (path, data) in files {
                    let json: serde_json::Value = serde_json::from_slice(data).unwrap();
                    let validation = validator.validate_json(&json, "recipe");
                    result.add_file_result(path.clone(), validation);
                }
                black_box(result)
            })
        },
    );
    
    // Target: Large datapack <100ms (1000 files, 25MB)
    let large_files = generate_datapack_files(1000);
    group.bench_with_input(
        BenchmarkId::new("large_datapack", "1000_files"),
        &large_files,
        |b, files| {
            b.iter(|| {
                let mut result = voxel_rsmcdoc::types::DatapackResult::new();
                for (path, data) in files {
                    let json: serde_json::Value = serde_json::from_slice(data).unwrap();
                    let validation = validator.validate_json(&json, "recipe");
                    result.add_file_result(path.clone(), validation);
                }
                black_box(result)
            })
        },
    );
    
    group.finish();
}

// Benchmark: Dependency extraction
fn bench_dependency_extraction(c: &mut Criterion) {
    let mut validator = McDocValidator::new();
    let ast = voxel_rsmcdoc::parse_mcdoc(COMPLEX_MCDOC).unwrap();
    let static_ast = Box::leak(Box::new(ast));
    validator.load_parsed_mcdoc("recipe.mcdoc".to_string(), static_ast).unwrap();
    
    let mut group = c.benchmark_group("dependency_extraction");
    
    group.bench_function("get_required_registries", |b| {
        let json = get_complex_json();
        b.iter(|| {
            validator.get_required_registries(black_box(&json), black_box("recipe"))
        })
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_mcdoc_parsing,
    bench_validator_setup,
    bench_json_validation,
    bench_datapack_analysis,
    bench_dependency_extraction
);

criterion_main!(benches); 