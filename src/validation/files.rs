use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) trait ProjectFiles {
    fn exists(&self, path: &Path) -> bool;
    fn read_to_string(&self, path: &Path) -> Result<String>;
}

pub(crate) struct FilesystemProjectFiles {
    root: PathBuf,
}

impl FilesystemProjectFiles {
    pub(crate) fn new(root: PathBuf) -> Self {
        Self { root }
    }
}

impl ProjectFiles for FilesystemProjectFiles {
    fn exists(&self, path: &Path) -> bool {
        self.root.join(path).as_path().exists()
    }

    fn read_to_string(&self, path: &Path) -> Result<String> {
        fs::read_to_string(self.root.join(path)).map_err(Into::into)
    }
}
