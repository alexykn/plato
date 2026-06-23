use std::{fs, path::PathBuf};

#[derive(Debug, Clone, Copy)]
pub(crate) enum CleanupPolicy {
    RemoveTargetOnFailure,
    PreserveTargetOnFailure,
}

pub(crate) struct ProjectGuard {
    path: PathBuf,
    success: bool,
    cleanup_policy: CleanupPolicy,
}

impl ProjectGuard {
    pub(crate) fn new(path: PathBuf, cleanup_policy: CleanupPolicy) -> Self {
        Self {
            path,
            success: false,
            cleanup_policy,
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

        if matches!(self.cleanup_policy, CleanupPolicy::PreserveTargetOnFailure) {
            eprintln!(
                "Project setup did not finish. Preserving pre-existing target {}",
                self.path.display()
            );
            return;
        }

        eprintln!(
            "Project setup did not finish. Cleaning up {}",
            self.path.display()
        );
        if self.path.exists()
            && let Err(error) = fs::remove_dir_all(&self.path)
        {
            eprintln!("Failed to clean up {}: {error}", self.path.display());
        }
    }
}
