use crate::languages::rust::{RustPackageManager, RustProjectScope, RustProjectType};
use crate::util::is_installed;
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::fs::{read_dir, read_to_string};
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug)]
pub(crate) struct CargoManifest {
    lib: Option<CargoLib>,
    bin: Option<Vec<CargoBin>>,
}

#[derive(Deserialize, Debug)]
struct CargoLib {
    path: Option<PathBuf>,
}

#[derive(Deserialize, Debug)]
struct CargoBin {
    path: Option<PathBuf>,
}

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
    let cargo_manifest_path = target.join("Cargo.toml");

    let has_default_bin = target.join("src/main.rs").is_file();

    let has_valid_bin_in_bin_dir = read_dir(bin_dir).is_ok_and(|entries| {
        entries.flatten().any(|entry| {
            entry.path().extension().is_some_and(|ext| {
                matches!(ext.to_str(), Some("rs")) && entry.file_type().is_ok_and(|ft| ft.is_file())
            })
        })
    });

    let entries_from_manifest = parse_cargo_manifest(&cargo_manifest_path)
        .ok()
        .and_then(|manifest| manifest.bin)
        .unwrap_or_default();

    let has_valid_bin_entries_from_manifest = entries_from_manifest.into_iter().any(|entry| {
        entry.path.is_some_and(|path| {
            path.extension().is_some_and(|ext| {
                matches!(ext.to_str(), Some("rs")) && target.join(&path).is_file()
            })
        })
    });

    has_default_bin || has_valid_bin_in_bin_dir || has_valid_bin_entries_from_manifest
}

fn has_rust_lib_targets(target: &Path) -> bool {
    let cargo_manifest_path = target.join("Cargo.toml");
    let has_default_lib = target.join("src/lib.rs").is_file();

    let has_valid_lib_entry_from_manifest = parse_cargo_manifest(&cargo_manifest_path)
        .ok()
        .and_then(|manifest| manifest.lib)
        .and_then(|lib| lib.path)
        .is_some_and(|path| {
            path.extension().is_some_and(|ext| {
                matches!(ext.to_str(), Some("rs")) && target.join(&path).is_file()
            })
        });

    has_default_lib || has_valid_lib_entry_from_manifest
}

pub(crate) fn get_rust_project_scope(target: &Path) -> RustProjectScope {
    use RustProjectScope::{Base, Build, Fetch};
    if !target.join("Cargo.toml").exists() {
        return Base;
    }
    if has_rust_lib_targets(target) || has_rust_bin_targets(target) {
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
    if has_rust_bin_targets(target) {
        Binary
    } else if has_rust_lib_targets(target) {
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
