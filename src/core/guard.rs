use std::{fs, path::PathBuf};

pub(crate) struct ProjectGuard {
    path: PathBuf,
    success: bool,
}

impl ProjectGuard {
    pub(crate) fn new(path: PathBuf) -> Self {
        Self {
            path,
            success: false,
        }
    }

    pub(crate) fn release(&mut self) {
        self.success = true;
    }
}

impl Drop for ProjectGuard {
    fn drop(&mut self) {
        if self.success {
            return;
        }

        eprintln!(
            "Project setup did not finish. Cleaning up {}",
            self.path.display()
        );
        if self.path.exists() {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
