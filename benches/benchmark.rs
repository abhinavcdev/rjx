use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::process::Command;
use std::time::Duration;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use serde_json::Value;
use gq::parser::parse_query;
use gq::query::QueryEngine;


// Sample JSON data for benchmarks
const SMALL_JSON: &str = r#"
{
    "name": "John Doe",
    "age": 30,
    "address": {
        "street": "123 Main St",
        "city": "Anytown",
        "state": "CA",
        "zip": "12345"
    },
    "phones": [
        {"type": "home", "number": "555-1234"},
        {"type": "work", "number": "555-5678"}
    ]
}
"#;

const MEDIUM_JSON: &str = r#"
{
    "users": [
        {
            "id": 1,
            "name": "John Doe",
            "email": "john@example.com",
            "address": {
                "street": "123 Main St",
                "city": "Anytown",
                "state": "CA",
                "zip": "12345"
            },
            "phones": [
                {"type": "home", "number": "555-1234"},
                {"type": "work", "number": "555-5678"}
            ]
        },
        {
            "id": 2,
            "name": "Jane Smith",
            "email": "jane@example.com",
            "address": {
                "street": "456 Oak Ave",
                "city": "Somewhere",
                "state": "NY",
                "zip": "67890"
            },
            "phones": [
                {"type": "home", "number": "555-4321"},
                {"type": "mobile", "number": "555-8765"}
            ]
        }
    ],
    "products": [
        {
            "id": 101,
            "name": "Laptop",
            "price": 999.99,
            "in_stock": true
        },
        {
            "id": 102,
            "name": "Smartphone",
            "price": 699.99,
            "in_stock": true
        },
        {
            "id": 103,
            "name": "Tablet",
            "price": 399.99,
            "in_stock": false
        }
    ]
}
"#;

// Generate a large JSON dataset for more realistic benchmarks
fn generate_large_json() -> String {
    let mut large_json = String::from("{\n  \"items\": [\n");
    
    for i in 0..1000 {
        large_json.push_str(&format!(r#"    {{
      "id": {},
      "name": "Item {}",
      "value": {},
      "tags": ["tag1", "tag2", "tag3"],
      "metadata": {{
        "created": "2023-01-01T00:00:00Z",
        "updated": "2023-01-02T00:00:00Z",
        "status": "active",
        "rating": {},
        "features": [
          {{"name": "feature1", "enabled": true}},
          {{"name": "feature2", "enabled": false}},
          {{"name": "feature3", "enabled": true}}
        ]
      }}
    }}"#, i, i, i * 10, i % 5));
        
        if i < 999 {
            large_json.push_str(",\n");
        } else {
            large_json.push_str("\n");
        }
    }
    
    large_json.push_str("  ]\n}");
    large_json
}

// Benchmark queries
const QUERIES: &[(&str, &str)] = &[
    ("simple_property", ".name"),
    ("nested_property", ".address.city"),
    ("array_element", ".phones[0].number"),
    // Skip complex filter queries for now as they're not yet implemented
    // ("filter_array", ".phones[] | select(.type == \"home\") | .number"),
];

const MEDIUM_QUERIES: &[(&str, &str)] = &[
    ("users_names", ".users[0].name"),
    // Skip complex filter queries for now as they're not yet implemented
    // ("filter_products", ".products[] | select(.in_stock == true)"),
    // ("complex_filter", ".users[] | select(.address.state == \"CA\") | {name, phone: .phones[0].number}"),
];

const LARGE_QUERIES: &[(&str, &str)] = &[
    ("all_ids", ".items[0].id"),
    // Skip complex filter queries for now as they're not yet implemented
    // ("filter_rating", ".items[] | select(.metadata.rating > 3) | .name"),
    // ("complex_transform", ".items[] | select(.metadata.features[].enabled == true) | {id, name, features: [.metadata.features[] | select(.enabled == true) | .name]}"),
];

// Create temporary files for benchmarking
fn create_temp_files() -> (String, String, String) {
    let temp_dir = std::env::temp_dir();
    
    let small_path = temp_dir.join("gq_bench_small.json");
    let medium_path = temp_dir.join("gq_bench_medium.json");
    let large_path = temp_dir.join("gq_bench_large.json");
    
    let mut small_file = File::create(&small_path).unwrap();
    small_file.write_all(SMALL_JSON.as_bytes()).unwrap();
    
    let mut medium_file = File::create(&medium_path).unwrap();
    medium_file.write_all(MEDIUM_JSON.as_bytes()).unwrap();
    
    let large_json = generate_large_json();
    let mut large_file = File::create(&large_path).unwrap();
    large_file.write_all(large_json.as_bytes()).unwrap();
    
    (
        small_path.to_string_lossy().to_string(),
        medium_path.to_string_lossy().to_string(),
        large_path.to_string_lossy().to_string()
    )
}

// Check if jq is installed
fn is_jq_installed() -> bool {
    Command::new("jq")
        .arg("--version")
        .output()
        .is_ok()
}

// Benchmark GQ against JQ
fn benchmark_comparison(c: &mut Criterion) {
    // Check if jq is installed
    let jq_available = is_jq_installed();
    if !jq_available {
        println!("Warning: jq is not installed or not in PATH. Skipping jq benchmarks.");
    }
    
    // Create temporary files
    let (small_path, medium_path, large_path) = create_temp_files();
    
    // Parse JSON data for GQ benchmarks
    let small_json: Value = serde_json::from_str(SMALL_JSON).unwrap();
    let medium_json: Value = serde_json::from_str(MEDIUM_JSON).unwrap();
    let large_json: Value = serde_json::from_str(&generate_large_json()).unwrap();
    
    // Create a benchmark group for small JSON
    {
        let mut group = c.benchmark_group("small_json");
        group.measurement_time(Duration::from_secs(10));
        
        for (name, query) in QUERIES {
            // Benchmark GQ
            let parsed_query = parse_query(query).unwrap();
            let engine = QueryEngine::new();
            
            group.bench_with_input(BenchmarkId::new("gq", name), query, |b, q| {
                b.iter(|| {
                    let parsed = parse_query(black_box(q)).unwrap();
                    let engine = QueryEngine::new();
                    engine.execute(&parsed, &small_json).unwrap();
                });
            });
            
            // Benchmark JQ if available
            if jq_available {
                group.bench_with_input(BenchmarkId::new("jq", name), query, |b, q| {
                    b.iter(|| {
                        Command::new("jq")
                            .arg(black_box(q))
                            .arg(&small_path)
                            .output()
                            .unwrap();
                    });
                });
            }
        }
        
        group.finish();
    }
    
    // Create a benchmark group for medium JSON
    {
        let mut group = c.benchmark_group("medium_json");
        group.measurement_time(Duration::from_secs(10));
        
        for (name, query) in MEDIUM_QUERIES {
            // Benchmark GQ
            group.bench_with_input(BenchmarkId::new("gq", name), query, |b, q| {
                b.iter(|| {
                    let parsed = parse_query(black_box(q)).unwrap();
                    let engine = QueryEngine::new();
                    engine.execute(&parsed, &medium_json).unwrap();
                });
            });
            
            // Benchmark JQ if available
            if jq_available {
                group.bench_with_input(BenchmarkId::new("jq", name), query, |b, q| {
                    b.iter(|| {
                        Command::new("jq")
                            .arg(black_box(q))
                            .arg(&medium_path)
                            .output()
                            .unwrap();
                    });
                });
            }
        }
        
        group.finish();
    }
    
    // Create a benchmark group for large JSON
    {
        let mut group = c.benchmark_group("large_json");
        group.measurement_time(Duration::from_secs(15));
        group.sample_size(30); // Fewer samples for large JSON to keep runtime reasonable
        
        for (name, query) in LARGE_QUERIES {
            // Benchmark GQ
            group.bench_with_input(BenchmarkId::new("gq", name), query, |b, q| {
                b.iter(|| {
                    let parsed = parse_query(black_box(q)).unwrap();
                    let engine = QueryEngine::new();
                    engine.execute(&parsed, &large_json).unwrap();
                });
            });
            
            // Benchmark JQ if available
            if jq_available {
                group.bench_with_input(BenchmarkId::new("jq", name), query, |b, q| {
                    b.iter(|| {
                        Command::new("jq")
                            .arg(black_box(q))
                            .arg(&large_path)
                            .output()
                            .unwrap();
                    });
                });
            }
        }
        
        group.finish();
    }
    
    // Clean up temporary files
    if Path::new(&small_path).exists() {
        std::fs::remove_file(&small_path).ok();
    }
    if Path::new(&medium_path).exists() {
        std::fs::remove_file(&medium_path).ok();
    }
    if Path::new(&large_path).exists() {
        std::fs::remove_file(&large_path).ok();
    }
}

criterion_group!(benches, benchmark_comparison);
criterion_main!(benches);
