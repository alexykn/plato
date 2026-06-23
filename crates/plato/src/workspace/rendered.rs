use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs::{create_dir_all, read, write};
use std::path::{Path, PathBuf};
use std::rc::Rc;

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
}
