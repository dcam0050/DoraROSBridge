use std::{collections::HashMap, path::Path};

use anyhow::{Context, Result};
use glob::glob;
use tracing::warn;

use super::{action::parse_action_file, message::parse_message_file, service::parse_service_file};
use crate::types::Package;

fn get_ros_msgs_each_package<P: AsRef<Path>>(root_dir: P) -> Result<Vec<Package>> {
    let mut map: HashMap<String, Package> = HashMap::new();

    let ros_formats = vec!["msg", "srv", "action"];

    // Return empty vector if root_dir is empty
    if root_dir.as_ref() == Path::new("") {
        let empty_vec: Vec<Package> = vec![];
        warn!("AMENT_PREFIX_PATH pointed to ''");
        return Ok(empty_vec);
    }

    for ros_format in ros_formats {
        let pattern = root_dir.as_ref().to_string_lossy().to_string()
            + "/**/"
            + ros_format
            + "/*."
            + ros_format;
        let mut visited_files = vec![];
        for entry in glob(&pattern).context("Failed to read glob pattern")? {
            let path = entry.context("Could not glob given path")?;
            let file_name = path
                .clone()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();

            // Extract package name from path
            // Handle both source and install directory structures:
            // - Source: /path/to/package/msg/file.msg -> package
            // - Install: /path/to/install/package/share/package/msg/file.msg -> package
            let package = if let Some(msg_dir) = path.parent() {
                if let Some(package_dir) = msg_dir.parent() {
                    // Check if we're in an install directory structure
                    if package_dir.file_name().map(|n| n.to_str().unwrap()) == Some("share") {
                        // Install structure: /install/package/share/package/msg/file.msg
                        if let Some(install_package_dir) = package_dir.parent() {
                            if let Some(package_name) = install_package_dir.file_name() {
                                package_name.to_string_lossy().to_string()
                            } else {
                                continue;
                            }
                        } else {
                            continue;
                        }
                    } else {
                        // Source structure: /path/to/package/msg/file.msg
                        if let Some(package_name) = package_dir.file_name() {
                            package_name.to_string_lossy().to_string()
                        } else {
                            continue;
                        }
                    }
                } else {
                    continue;
                }
            } else {
                continue;
            };

            // Hack
            if file_name == "libstatistics_collector" {
                continue;
            } else if visited_files.contains(&(package.clone(), file_name.clone())) {
                warn!(
                    "found two versions of package: {:?}, message: {:?}. will skip the one in: {:#?}",
                    package, file_name, path
                );
                continue;
            } else {
                visited_files.push((package.clone(), file_name.clone()));
            }

            let p = map
                .entry(package.clone())
                .or_insert_with(|| Package::new(package.clone()));

            match ros_format {
                "msg" => {
                    match parse_message_file(&package, path.clone()) {
                        Ok(msg) => p.messages.push(msg),
                        Err(e) => {
                            warn!("Failed to parse message file {:?}: {:?}", path, e);
                            continue;
                        }
                    }
                }
                "srv" => {
                    match parse_service_file(&package, path.clone()) {
                        Ok(srv) => p.services.push(srv),
                        Err(e) => {
                            warn!("Failed to parse service file {:?}: {:?}", path, e);
                            continue;
                        }
                    }
                }
                "action" => {
                    match parse_action_file(&package, path.clone()) {
                        Ok(action) => p.actions.push(action),
                        Err(e) => {
                            warn!("Failed to parse action file {:?}: {:?}", path, e);
                            continue;
                        }
                    }
                }
                _ => todo!(),
            }
        }
    }
    
    // Only assert if we actually have paths to process and found some message files
    // Some directories in AMENT_PREFIX_PATH might not contain message files, which is normal
    if !root_dir.as_ref().to_string_lossy().is_empty() && map.is_empty() {
        // Check if this directory actually contains any message files
        let has_msg_files = glob(&(root_dir.as_ref().to_string_lossy().to_string() + "/**/*.msg"))
            .map(|entries| entries.count() > 0)
            .unwrap_or(false);
        
        if has_msg_files {
            debug_assert!(
                false,
                "it seens that no package was generated from your AMENT_PREFIX_PATH directory: {:?}",
                root_dir.as_ref()
            );
        }
    }

    let packages = map.into_values().collect();
    Ok(packages)
}

pub fn get_packages<P>(paths: &[P]) -> Result<Vec<Package>>
where
    P: AsRef<Path>,
{
    let mut packages = paths
        .iter()
        .map(get_ros_msgs_each_package)
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .flatten()
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>();

    packages.sort_by_key(|p| p.name.clone());
    packages.dedup_by_key(|p| p.name.clone());

    Ok(packages)
}
