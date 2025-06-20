use std::process::Command;
use std::path::Path;

fn main() {
    let thrift_files = [
        "thrift/tao_core.thrift",
        "thrift/associations.thrift",
        "schemas/ent_user.thrift",
        "schemas/ent_post.thrift",
        "schemas/ent_comment.thrift",
        "schemas/ent_group.thrift",
        "schemas/ent_page.thrift",
    
        "schemas/ent_event.thrift",];

    // Create the generated models directory
    std::fs::create_dir_all("src/models").expect("Failed to create models directory");

    // Generate Rust code for each thrift file
    for thrift_file in &thrift_files {
        if Path::new(thrift_file).exists() {
            let output = Command::new("thrift")
                .arg("--gen")
                .arg("rs")
                .arg("-out")
                .arg("src/models")
                .arg(thrift_file)
                .output()
                .expect("Failed to execute thrift compiler");

            if !output.status.success() {
                panic!(
                    "Thrift compilation failed for {}: {}",
                    thrift_file,
                    String::from_utf8_lossy(&output.stderr)
                );
            }

            println!("cargo:rerun-if-changed={}", thrift_file);
        }
    }

    println!("cargo:rerun-if-changed=build.rs");
}