use std::path::PathBuf;
use dora_ros2_bridge_msg_gen;

fn main() {
    // Test the exact same logic as the build.rs
    let paths = ament_prefix_paths();
    println!("Found {} paths in AMENT_PREFIX_PATH", paths.len());
    
    for (i, path) in paths.iter().enumerate() {
        println!("Path {}: {:?}", i, path);
    }
    
    // Test the package discovery
    match dora_ros2_bridge_msg_gen::get_packages(paths.as_slice()) {
        Ok(packages) => {
            println!("Successfully found {} packages", packages.len());
            for package in packages {
                println!("Package: {} ({} messages, {} services, {} actions)", 
                    package.name, 
                    package.messages.len(), 
                    package.services.len(), 
                    package.actions.len());
            }
        }
        Err(e) => {
            println!("Error getting packages: {:?}", e);
        }
    }
}

fn ament_prefix_paths() -> Vec<PathBuf> {
    let ament_prefix_path: String = match std::env::var("AMENT_PREFIX_PATH") {
        Ok(path) => path,
        Err(std::env::VarError::NotPresent) => {
            println!("cargo:warning='AMENT_PREFIX_PATH not set'");
            String::new()
        }
        Err(std::env::VarError::NotUnicode(s)) => {
            panic!(
                "AMENT_PREFIX_PATH is not valid unicode: `{}`",
                s.to_string_lossy()
            );
        }
    };

    let paths: Vec<_> = ament_prefix_path.split(':').map(PathBuf::from).collect();
    paths
}
