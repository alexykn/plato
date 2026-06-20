use anyhow::{Context, Result, bail};
use std::collections::HashMap;
use std::fs::{create_dir_all, read, write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::validation::files::ProjectFiles;

use super::content::FileContent;

pub(crate) struct RenderedWorkspace {
    files: HashMap<PathBuf, FileContent>,
}

impl RenderedWorkspace {
    pub(crate) fn new(files: HashMap<PathBuf, FileContent>) -> Self {
        Self { files }
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

    fn content(&self, path: &Path) -> Option<&FileContent> {
        self.files.get(path)
    }
}

impl ProjectFiles for RenderedWorkspace {
    fn exists(&self, path: &Path) -> bool {
        self.files.contains_key(path)
    }

    fn read_to_string(&self, path: &Path) -> Result<String> {
        match self.content(path) {
            Some(FileContent::Binary(bytes)) => String::from_utf8(bytes.to_vec())
                .with_context(|| format!("Rendered file {} is not valid UTF-8", path.display())),
            Some(FileContent::BinaryLazy {
                path: source_path,
                cache,
            }) => {
                if cache.get().is_none() {
                    let bytes = read(source_path).map(Rc::<[u8]>::from).with_context(|| {
                        format!("Failed to read binary file {}", source_path.display())
                    })?;
                    let _ = cache.set(bytes);
                }
                let bytes = cache
                    .get()
                    .context("Binary cache was not initialized after read")?;
                String::from_utf8(bytes.to_vec())
                    .with_context(|| format!("Rendered file {} is not valid UTF-8", path.display()))
            }
            Some(FileContent::None) => bail!("Rendered path {} is a directory", path.display()),
            Some(FileContent::Template(_)) => {
                bail!("Rendered path {} is an unrendered template", path.display())
            }
            None => bail!("Rendered file {} does not exist", path.display()),
        }
    }
}
