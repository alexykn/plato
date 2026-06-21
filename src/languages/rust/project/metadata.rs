use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CargoManifestMetadata {
    pub(crate) exists: bool,
}

impl CargoManifestMetadata {
    pub(crate) fn from_project_dir(project_dir: &Path) -> Self {
        Self {
            exists: project_dir.join("Cargo.toml").exists(),
        }
    }
}
