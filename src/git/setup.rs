use anyhow::{Result, bail};
use std::ffi::OsStr;
use std::path::Path;

use crate::config::{GitAutoCrlfConfig, GitAutoCrlfMode, GitEolConfig};
use crate::util::execute_command;

#[derive(Debug, Clone, Copy)]
pub(crate) struct GitSetupOptions<'a> {
    pub(crate) initial_branch: Option<&'a str>,
    pub(crate) local_config: GitLocalConfig<'a>,
    pub(crate) initial_commit: Option<GitInitialCommit<'a>>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GitLocalConfig<'a> {
    pub(crate) user: GitUserConfig<'a>,
    pub(crate) commit: GitCommitLocalConfig,
    pub(crate) core: GitCoreConfig<'a>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GitUserConfig<'a> {
    pub(crate) name: Option<&'a str>,
    pub(crate) email: Option<&'a str>,
    pub(crate) signing_key: Option<&'a str>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GitCommitLocalConfig {
    pub(crate) gpgsign: Option<bool>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GitCoreConfig<'a> {
    pub(crate) hooks_path: Option<&'a Path>,
    pub(crate) autocrlf: Option<GitAutoCrlf>,
    pub(crate) eol: Option<GitEol>,
    pub(crate) filemode: Option<bool>,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct GitInitialCommit<'a> {
    pub(crate) message: &'a str,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum GitAutoCrlf {
    Bool(bool),
    Input,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum GitEol {
    Lf,
    Crlf,
    Native,
}

impl GitAutoCrlf {
    fn as_git_value(self) -> &'static str {
        match self {
            Self::Bool(true) => "true",
            Self::Bool(false) => "false",
            Self::Input => "input",
        }
    }
}

impl From<GitAutoCrlfConfig> for GitAutoCrlf {
    fn from(value: GitAutoCrlfConfig) -> Self {
        match value {
            GitAutoCrlfConfig::Bool(value) => Self::Bool(value),
            GitAutoCrlfConfig::Mode(GitAutoCrlfMode::Input) => Self::Input,
        }
    }
}

impl GitEol {
    fn as_git_value(self) -> &'static str {
        match self {
            Self::Lf => "lf",
            Self::Crlf => "crlf",
            Self::Native => "native",
        }
    }
}

impl From<GitEolConfig> for GitEol {
    fn from(value: GitEolConfig) -> Self {
        match value {
            GitEolConfig::Lf => Self::Lf,
            GitEolConfig::Crlf => Self::Crlf,
            GitEolConfig::Native => Self::Native,
        }
    }
}

pub(crate) fn setup_git_repository(target: &Path, options: GitSetupOptions<'_>) -> Result<()> {
    validate_options(&options)?;
    init_repository(target, options.initial_branch)?;
    apply_local_config(target, &options.local_config)?;

    if let Some(initial_commit) = options.initial_commit {
        create_initial_commit(target, initial_commit)?;
    }

    Ok(())
}

fn init_repository(target: &Path, initial_branch: Option<&str>) -> Result<()> {
    if let Some(initial_branch) = initial_branch {
        return execute_command("git", ["init", "--initial-branch", initial_branch], target);
    }

    execute_command("git", ["init"], target)
}

fn apply_local_config(target: &Path, config: &GitLocalConfig<'_>) -> Result<()> {
    apply_user_config(target, &config.user)?;
    apply_commit_config(target, config.commit)?;
    apply_core_config(target, &config.core)?;
    Ok(())
}

fn apply_user_config(target: &Path, config: &GitUserConfig<'_>) -> Result<()> {
    if let Some(name) = config.name {
        git_config(target, "user.name", name)?;
    }

    if let Some(email) = config.email {
        git_config(target, "user.email", email)?;
    }

    if let Some(signing_key) = config.signing_key {
        git_config(target, "user.signingkey", signing_key)?;
    }

    Ok(())
}

fn apply_commit_config(target: &Path, config: GitCommitLocalConfig) -> Result<()> {
    if let Some(gpgsign) = config.gpgsign {
        git_config(target, "commit.gpgsign", git_bool(gpgsign))?;
    }

    Ok(())
}

fn apply_core_config(target: &Path, config: &GitCoreConfig<'_>) -> Result<()> {
    if let Some(hooks_path) = config.hooks_path {
        git_config_path(target, "core.hooksPath", hooks_path)?;
    }

    if let Some(autocrlf) = config.autocrlf {
        git_config(target, "core.autocrlf", autocrlf.as_git_value())?;
    }

    if let Some(eol) = config.eol {
        git_config(target, "core.eol", eol.as_git_value())?;
    }

    if let Some(filemode) = config.filemode {
        git_config(target, "core.filemode", git_bool(filemode))?;
    }

    Ok(())
}

fn create_initial_commit(target: &Path, initial_commit: GitInitialCommit<'_>) -> Result<()> {
    execute_command("git", ["add", "--all"], target)?;
    execute_command(
        "git",
        ["commit", "--message", initial_commit.message],
        target,
    )
}

fn validate_options(options: &GitSetupOptions<'_>) -> Result<()> {
    ensure_optional_non_empty(options.initial_branch, "initial_branch")?;
    ensure_optional_non_empty(options.local_config.user.name, "user.name")?;
    ensure_optional_non_empty(options.local_config.user.email, "user.email")?;
    ensure_optional_non_empty(options.local_config.user.signing_key, "user.signing_key")?;

    if let Some(initial_commit) = options.initial_commit {
        ensure_non_empty(initial_commit.message, "commit.initial_message")?;
    }

    Ok(())
}

fn ensure_optional_non_empty(value: Option<&str>, field: &str) -> Result<()> {
    if let Some(value) = value {
        ensure_non_empty(value, field)?;
    }

    Ok(())
}

fn ensure_non_empty(value: &str, field: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("Git {field} cannot be empty");
    }

    Ok(())
}

fn git_config(target: &Path, key: &str, value: &str) -> Result<()> {
    execute_command("git", ["config", key, value], target)
}

fn git_config_path(target: &Path, key: &str, value: &Path) -> Result<()> {
    execute_command(
        "git",
        [OsStr::new("config"), OsStr::new(key), value.as_os_str()],
        target,
    )
}

fn git_bool(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{create_dir_all, read_to_string, remove_dir_all, write};
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new() -> Self {
            let id = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let counter = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir()
                .join(format!("plato-git-{}-{id}-{counter}", std::process::id()));
            create_dir_all(&path).unwrap();
            Self { path }
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = remove_dir_all(&self.path);
        }
    }

    #[test]
    fn setup_git_repository_uses_plain_init_without_initial_branch() {
        let temp_dir = TempDir::new();

        setup_git_repository(&temp_dir.path, default_options()).unwrap();

        assert!(temp_dir.path.join(".git").exists());
    }

    #[test]
    fn setup_git_repository_applies_configured_git_settings() {
        let temp_dir = TempDir::new();
        let hooks_path = Path::new(".githooks");
        let options = GitSetupOptions {
            initial_branch: Some("trunk"),
            local_config: GitLocalConfig {
                user: GitUserConfig {
                    name: Some("Jane Doe"),
                    email: Some("jane@example.com"),
                    signing_key: Some("ABC123"),
                },
                commit: GitCommitLocalConfig {
                    gpgsign: Some(false),
                },
                core: GitCoreConfig {
                    hooks_path: Some(hooks_path),
                    autocrlf: Some(GitAutoCrlf::Input),
                    eol: Some(GitEol::Lf),
                    filemode: Some(true),
                },
            },
            initial_commit: None,
        };

        setup_git_repository(&temp_dir.path, options).unwrap();

        let git_config = read_to_string(temp_dir.path.join(".git/config")).unwrap();
        assert!(git_config.contains("name = Jane Doe"));
        assert!(git_config.contains("email = jane@example.com"));
        assert!(git_config.contains("signingkey = ABC123"));
        assert!(git_config.contains("gpgsign = false"));
        assert!(git_config.contains("hooksPath = .githooks"));
        assert!(git_config.contains("autocrlf = input"));
        assert!(git_config.contains("eol = lf"));
        assert!(git_config.contains("filemode = true"));

        let head = read_to_string(temp_dir.path.join(".git/HEAD")).unwrap();
        assert_eq!(head.trim(), "ref: refs/heads/trunk");
    }

    #[test]
    fn setup_git_repository_can_create_initial_commit() {
        let temp_dir = TempDir::new();
        write(temp_dir.path.join("README.md"), "# Project\n").unwrap();
        let options = GitSetupOptions {
            initial_branch: Some("main"),
            local_config: GitLocalConfig {
                user: GitUserConfig {
                    name: Some("Jane Doe"),
                    email: Some("jane@example.com"),
                    signing_key: None,
                },
                commit: GitCommitLocalConfig { gpgsign: None },
                core: GitCoreConfig {
                    hooks_path: None,
                    autocrlf: None,
                    eol: None,
                    filemode: None,
                },
            },
            initial_commit: Some(GitInitialCommit {
                message: "Initial commit",
            }),
        };

        setup_git_repository(&temp_dir.path, options).unwrap();

        let head_ref = read_to_string(temp_dir.path.join(".git/refs/heads/main")).unwrap();
        assert!(!head_ref.trim().is_empty());
    }

    #[test]
    fn setup_git_repository_rejects_empty_config_values() {
        let temp_dir = TempDir::new();
        let options = GitSetupOptions {
            initial_branch: Some(" "),
            ..default_options()
        };

        let error = setup_git_repository(&temp_dir.path, options).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("Git initial_branch cannot be empty")
        );
    }

    fn default_options<'a>() -> GitSetupOptions<'a> {
        GitSetupOptions {
            initial_branch: None,
            local_config: GitLocalConfig {
                user: GitUserConfig {
                    name: None,
                    email: None,
                    signing_key: None,
                },
                commit: GitCommitLocalConfig { gpgsign: None },
                core: GitCoreConfig {
                    hooks_path: None,
                    autocrlf: None,
                    eol: None,
                    filemode: None,
                },
            },
            initial_commit: None,
        }
    }
}
