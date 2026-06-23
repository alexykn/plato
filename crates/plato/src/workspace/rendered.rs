use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs::{create_dir_all, read, write};
use std::path::{Component, Path, PathBuf};
use std::rc::Rc;

use super::content::FileContent;

pub(crate) struct RenderedWorkspace {
    files: HashMap<PathBuf, FileContent>,
}

impl RenderedWorkspace {
    pub(crate) fn new(files: HashMap<PathBuf, FileContent>) -> Self {
        Self { files }
    }

    pub(crate) fn contains_directory(&self, path: &Path) -> bool {
        let path = normalize_directory_path(path);
        if path.as_os_str().is_empty() {
            return true;
        }

        if self
            .files
            .get(&path)
            .is_some_and(|content| matches!(content, FileContent::None))
        {
            return true;
        }

        self.files
            .keys()
            .any(|candidate| candidate != &path && candidate.starts_with(&path))
    }

    pub(crate) fn flush_to_disk(&self, target: &Path) -> Result<()> {
        for (path, content) in &self.files {
            let full_path = target.join(path);
            match content {
                FileContent::BinaryLazy {
                    path: source_path,
                    cache,
                } => {
                    if cache.get().is_none() {
                        let bytes = read(source_path).map(Rc::<[u8]>::from).with_context(|| {
                            format!("Failed to read binary file {}", source_path.display())
                        })?;
                        let _ = cache.set(bytes);
                    }
                    let bytes = cache
                        .get()
                        .context("Binary cache was not initialized after read")?;
                    if let Some(parent) = full_path.parent() {
                        create_dir_all(parent)?;
                    }
                    write(full_path, bytes.as_ref())?;
                }
                FileContent::Binary(bytes) => {
                    if let Some(parent) = full_path.parent() {
                        create_dir_all(parent)?;
                    }
                    write(full_path, bytes.as_ref())?;
                }
                FileContent::None => {
                    create_dir_all(full_path)?;
                }
                FileContent::Template(_) => {
                    eprintln!(
                        "WARNING: Found unrendered template at {}. Skipping.",
                        path.display()
                    );
                }
            }
        }
        Ok(())
    }
}

fn normalize_directory_path(path: &Path) -> PathBuf {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(PathBuf::from(value)),
            _ => None,
        })
        .fold(PathBuf::new(), |mut normalized, component| {
            normalized.push(component);
            normalized
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_rendered_directories_from_descendants() {
        let workspace = RenderedWorkspace::new(HashMap::from([(
            PathBuf::from("backend/pyproject.toml"),
            FileContent::Binary(Rc::<[u8]>::from(Vec::<u8>::new())),
        )]));

        assert!(workspace.contains_directory(Path::new(".")));
        assert!(workspace.contains_directory(Path::new("backend")));
        assert!(workspace.contains_directory(Path::new("./backend")));
        assert!(!workspace.contains_directory(Path::new("frontend")));
        assert!(!workspace.contains_directory(Path::new("backend/pyproject.toml")));
    }
}
