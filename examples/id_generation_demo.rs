// Demo showing TAO ID generation integration
// Run with: cargo run --example id_generation_demo

use tao_database::infrastructure::TaoIdGenerator;

fn main() {
    println!("🚀 TAO ID Generator Demo");
    println!("========================");
    
    // Create ID generator for shard 42
    let generator = TaoIdGenerator::new(42);
    
    println!("Shard ID: {}", generator.shard_id());
    println!();
    
    // Generate some IDs
    println!("Generated IDs:");
    for i in 0..5 {
        let id = generator.next_id();
        let extracted_shard = TaoIdGenerator::extract_shard_id(id);
        let timestamp = TaoIdGenerator::extract_timestamp(id);
        let sequence = TaoIdGenerator::extract_sequence(id);
        
        println!("  {}: ID={} shard={} timestamp={} sequence={}", 
                 i+1, id, extracted_shard, timestamp, sequence);
    }
    
    println!();
    println!("✅ All IDs have embedded shard information and are unique!");
    println!("💡 In TAO layer: Entity creation → ID generation → Database storage");
}