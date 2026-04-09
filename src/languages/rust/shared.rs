use crate::languages::rust::{RustPackageManager, RustProjectScope, RustProjectType};
use crate::util::is_installed;
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::fs::{read_dir, read_to_string};
use std::path::Path;

#[derive(Deserialize, Debug)]
pub(crate) struct CargoManifest {
    lib: Option<CargoLib>,
    bin: Option<Vec<CargoBin>>,
}

#[derive(Deserialize, Debug)]
struct CargoLib;

#[derive(Deserialize, Debug)]
struct CargoBin;

fn parse_cargo_manifest(cargo_manifest_path: &Path) -> Result<CargoManifest> {
    if !cargo_manifest_path.exists() {
        bail!("This shoud not have happened, how did we get here?!")
    }
    let content = read_to_string(cargo_manifest_path).context(format!(
        "Could not Cargo.toml at {}",
        cargo_manifest_path.display()
    ))?;
    let cargo_manifest: CargoManifest = toml::from_str(&content)?;
    Ok(cargo_manifest)
}

fn has_rust_bin_targets(target: &Path) -> bool {
    let bin_dir = target.join("src/bin");
    let Ok(entries) = read_dir(bin_dir) else {
        return false;
    };
    entries
        .flatten()
        .any(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("rs"))
}

pub(crate) fn get_rust_project_scope(target: &Path) -> RustProjectScope {
    use RustProjectScope::{Base, Build, Fetch};
    if !target.join("Cargo.toml").exists() {
        return Base;
    }
    if target.join("src/main.rs").exists()
        || target.join("src/lib.rs").exists()
        || has_rust_bin_targets(target)
    {
        return Build;
    }
    Fetch
}

pub(crate) fn get_rust_project_type(target: &Path) -> RustProjectType {
    use RustProjectType::{Binary, Library};
    let cargo_manifest_path = target.join("Cargo.toml");
    if !cargo_manifest_path.exists() {
        return Binary;
    }
    let cargo_manifest = parse_cargo_manifest(&cargo_manifest_path).ok();
    let has_lib_files = target.join("src/lib.rs").exists();
    let has_bin_files = target.join("src/main.rs").exists() || target.join("src/bin").is_dir();
    let has_lib_manifest = cargo_manifest
        .as_ref()
        .is_some_and(|manifest| manifest.lib.is_some());
    let has_bin_manifest = cargo_manifest
        .as_ref()
        .and_then(|manifest| manifest.bin.as_ref())
        .is_some_and(|bins| !bins.is_empty());
    let has_lib = has_lib_files || has_lib_manifest;
    let has_bin = has_bin_files || has_bin_manifest;
    if has_bin {
        Binary
    } else if has_lib {
        Library
    } else {
        Binary
    }
}

pub(crate) fn get_rust_package_manager() -> RustPackageManager {
    if is_installed("cargo") {
        return RustPackageManager::Cargo;
    }
    eprintln!("No supported rust package manager found for 'project_scope: auto'.");
    RustPackageManager::None
}
