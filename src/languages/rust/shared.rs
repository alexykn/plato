use crate::languages::rust::RustPackageManager::{Cargo, None};
use crate::languages::rust::RustProjectScope::{Base, Build, Fetch};
use crate::languages::rust::RustProjectType::{Binary, Library};
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

fn has_rust_bin_targets(target: &Path, manifest: Option<&CargoManifest>) -> bool {
    let bin_dir = target.join("src/bin");

    let has_default_bin = target.join("src/main.rs").is_file();

    let has_valid_bin_in_bin_dir = read_dir(bin_dir).is_ok_and(|entries| {
        entries.flatten().any(|entry| {
            entry.path().extension().is_some_and(|ext| {
                matches!(ext.to_str(), Some("rs")) && entry.file_type().is_ok_and(|ft| ft.is_file())
            })
        })
    });

    let has_valid_bin_entries_from_manifest = manifest
        .and_then(|manifest| manifest.bin.as_ref())
        .is_some_and(|bins| {
            bins.iter().any(|entry| {
                entry.path.as_ref().is_some_and(|path| {
                    path.extension().is_some_and(|ext| {
                        matches!(ext.to_str(), Some("rs")) && target.join(path).is_file()
                    })
                })
            })
        });

    has_default_bin || has_valid_bin_in_bin_dir || has_valid_bin_entries_from_manifest
}

fn has_rust_lib_targets(target: &Path, manifest: Option<&CargoManifest>) -> bool {
    let has_default_lib = target.join("src/lib.rs").is_file();

    let has_valid_lib_entry_from_manifest = manifest
        .and_then(|manifest| manifest.lib.as_ref())
        .and_then(|lib| lib.path.as_ref())
        .is_some_and(|path| {
            path.extension().is_some_and(|ext| {
                matches!(ext.to_str(), Some("rs")) && target.join(path).is_file()
            })
        });

    has_default_lib || has_valid_lib_entry_from_manifest
}

pub(crate) fn get_rust_project_scope(target: &Path) -> Result<RustProjectScope> {
    let cargo_manifest_path = target.join("Cargo.toml");
    let cargo_manifest = parse_cargo_manifest(&cargo_manifest_path)?;

    if !cargo_manifest_path.exists() {
        return Ok(Base);
    }
    if has_rust_lib_targets(target, Some(&cargo_manifest))
        || has_rust_bin_targets(target, Some(&cargo_manifest))
    {
        return Ok(Build);
    }
    Ok(Fetch)
}

pub(crate) fn get_rust_project_type(target: &Path) -> Result<RustProjectType> {
    let cargo_manifest_path = target.join("Cargo.toml");
    let cargo_manifest = parse_cargo_manifest(&cargo_manifest_path)?;

    if !cargo_manifest_path.exists() {
        return Ok(Binary);
    }
    if has_rust_bin_targets(target, Some(&cargo_manifest)) {
        Ok(Binary)
    } else if has_rust_lib_targets(target, Some(&cargo_manifest)) {
        Ok(Library)
    } else {
        Ok(Binary)
    }
}

pub(crate) fn get_rust_package_manager() -> RustPackageManager {
    if is_installed("cargo") {
        return Cargo;
    }
    eprintln!("No supported rust package manager found for 'project_scope: auto'.");
    None
}
