// Simplified build.rs - Entity generation now handled by entc binary
// The unified workflow is: Schema -> entc -> Thrift + Enhanced Entities

fn main() {
    // Create directory for complete entities
    std::fs::create_dir_all("src/models").expect("Failed to create models directory");
    
    // Track schema changes to trigger regeneration
    println!("cargo:rerun-if-changed=src/schemas/");
    println!("cargo:rerun-if-changed=src/ent_schema.rs");
    println!("cargo:rerun-if-changed=src/ent_codegen.rs");
    println!("cargo:rerun-if-changed=build.rs");
    
    // Note: Actual entity generation happens via `cargo run --bin entc generate`
    // This ensures the developer has a single command workflow:
    // 1. Write Ent schemas in src/schemas/
    // 2. Run `entc generate` 
    // 3. Build with `cargo build`
}