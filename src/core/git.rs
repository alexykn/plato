use anyhow::{Context, Result, bail};
use directories::BaseDirs;
use fs2::FileExt;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions, create_dir_all};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::config::{GitProvider, GlobalConfig};
use crate::core::path::{reject_parent_components, resolve_safe_subpath};
use crate::util::execute_command;

#[derive(Debug, Clone)]
pub(crate) struct GitTemplateSpec {
    pub(crate) url: String,
    pub(crate) revision: Option<String>,
    pub(crate) subpath: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub(crate) struct GitSpecParts {
    pub(crate) url: String,
    pub(crate) inline_revision: Option<String>,
}

pub(crate) struct GitTemplateFetcher {
    cache_root: PathBuf,
}

impl GitTemplateFetcher {
    pub(crate) fn from_user_cache_dir() -> Result<Self> {
        let base_dirs = BaseDirs::new().context("Could not find home directory")?;
        Ok(Self::new(base_dirs.home_dir().join(".cache/plato/git")))
    }

    pub(crate) fn new(cache_root: PathBuf) -> Self {
        Self { cache_root }
    }

    pub(crate) fn prepare_checkout(&self, spec: &GitTemplateSpec) -> Result<GitCheckout> {
        create_dir_all(&self.cache_root).with_context(|| {
            format!(
                "Could not create Git cache dir {}",
                self.cache_root.display()
            )
        })?;

        let cache_path = self.cache_path(&spec.url);
        let lock_path = self.lock_path(&spec.url);
        let _lock = GitCacheLock::acquire(&lock_path)?;

        if cache_path.exists() {
            run_git(
                [
                    "fetch",
                    "--prune",
                    "--tags",
                    "origin",
                    "+refs/heads/*:refs/heads/*",
                ],
                &cache_path,
                "fetch remote template updates",
            )?;
        } else {
            let cache_path_arg = cache_path.to_string_lossy().into_owned();
            run_git(
                [
                    "clone",
                    "--bare",
                    spec.url.as_str(),
                    cache_path_arg.as_str(),
                ],
                &self.cache_root,
                "clone remote template into cache",
            )?;
        }

        let checkout_root = self.create_checkout_dir()?;
        let cache_path_arg = cache_path.to_string_lossy().into_owned();
        let checkout_arg = checkout_root.to_string_lossy().into_owned();
        run_git(
            ["clone", cache_path_arg.as_str(), checkout_arg.as_str()],
            &self.cache_root,
            "create temporary remote template checkout",
        )?;

        if let Some(revision) = &spec.revision {
            run_git(
                ["checkout", revision.as_str()],
                &checkout_root,
                "checkout remote template revision",
            )?;
        }

        let source_path = resolve_safe_subpath(&checkout_root, spec.subpath.as_deref())?;
        Ok(GitCheckout {
            source_path,
            cleanup: TempCheckout::new(checkout_root),
        })
    }

    fn cache_path(&self, url: &str) -> PathBuf {
        self.cache_root
            .join(format!("{}.git", blake3::hash(url.as_bytes()).to_hex()))
    }

    fn lock_path(&self, url: &str) -> PathBuf {
        self.cache_root
            .join(format!("{}.lock", blake3::hash(url.as_bytes()).to_hex()))
    }

    fn create_checkout_dir(&self) -> Result<PathBuf> {
        let temp_root = self.cache_root.join("checkouts");
        create_dir_all(&temp_root).with_context(|| {
            format!(
                "Could not create Git checkout temp dir {}",
                temp_root.display()
            )
        })?;

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("System clock is before UNIX epoch")?
            .as_nanos();
        let checkout_path = temp_root.join(format!("{}-{timestamp}", std::process::id()));
        create_dir_all(&checkout_path).with_context(|| {
            format!(
                "Could not create Git checkout dir {}",
                checkout_path.display()
            )
        })?;
        Ok(checkout_path)
    }
}

pub(crate) struct GitCheckout {
    pub(crate) source_path: PathBuf,
    cleanup: TempCheckout,
}

impl GitCheckout {
    pub(crate) fn into_cleanup(self) -> TempCheckout {
        self.cleanup
    }
}

pub(crate) struct TempCheckout {
    path: PathBuf,
}

impl TempCheckout {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl Drop for TempCheckout {
    fn drop(&mut self) {
        if self.path.exists() {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}

struct GitCacheLock {
    file: File,
}

impl GitCacheLock {
    fn acquire(lock_path: &Path) -> Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(lock_path)
            .with_context(|| format!("Could not open Git cache lock {}", lock_path.display()))?;
        file.lock_exclusive()
            .with_context(|| format!("Could not lock Git cache {}", lock_path.display()))?;
        Ok(Self { file })
    }
}

impl Drop for GitCacheLock {
    fn drop(&mut self) {
        let _ = self.file.unlock();
    }
}

pub(crate) fn run_git<I, S>(args: I, current_dir: &Path, context: &str) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    execute_command("git", args, current_dir)
        .with_context(|| format!("Could not {context}. Verify Git authentication, SSH agent, or credential helper setup if this is a private repository."))
}

pub(crate) fn parse_git_spec_parts(raw: &str, config: &GlobalConfig) -> Result<GitSpecParts> {
    let (spec, inline_revision) = split_inline_revision(raw)?;

    if is_rejected_local_spec(spec) {
        bail!("Local Git repository specs are not supported. Use --path for local templates.");
    }

    if is_url(spec) {
        reject_embedded_credentials(spec)?;
        validate_url_shape(spec)?;
        return Ok(GitSpecParts {
            url: spec.to_string(),
            inline_revision,
        });
    }

    if let Some((provider, path)) = split_provider_shorthand(spec) {
        validate_remote_path(path)?;
        return Ok(GitSpecParts {
            url: build_provider_url(provider, path, config),
            inline_revision,
        });
    }

    if spec.starts_with("git@") {
        validate_scp_like(spec)?;
        return Ok(GitSpecParts {
            url: spec.to_string(),
            inline_revision,
        });
    }

    if let Some((host, path)) = split_host_shorthand(spec) {
        validate_host(host)?;
        validate_remote_path(path)?;
        let url = build_host_url(host, path);
        eprintln!("WARNING: Interpreting {spec:?} as SSH remote {url:?} by adding git@.");
        return Ok(GitSpecParts {
            url,
            inline_revision,
        });
    }

    if spec.contains('/') {
        validate_remote_path(spec)?;
        return Ok(GitSpecParts {
            url: build_provider_url(config.plato.default_git_provider, spec, config),
            inline_revision,
        });
    }

    bail!("Git template spec {raw:?} is not a supported Git remote syntax")
}

pub(crate) fn merge_git_template_spec(
    raw: &str,
    config: &GlobalConfig,
    configured_rev: Option<&str>,
    configured_subpath: Option<&Path>,
    cli_rev: Option<&str>,
    cli_subpath: Option<&Path>,
) -> Result<GitTemplateSpec> {
    let parts = parse_git_spec_parts(raw, config)?;

    Ok(GitTemplateSpec {
        url: parts.url,
        revision: cli_rev
            .map(str::to_string)
            .or(parts.inline_revision)
            .or_else(|| configured_rev.map(str::to_string)),
        subpath: cli_subpath
            .map(Path::to_path_buf)
            .or_else(|| configured_subpath.map(Path::to_path_buf)),
    })
}

fn split_inline_revision(raw: &str) -> Result<(&str, Option<String>)> {
    let Some((spec, revision)) = raw.rsplit_once('#') else {
        return Ok((raw, None));
    };
    if revision.is_empty() {
        bail!("Git template revision after '#' must not be empty");
    }
    Ok((spec, Some(revision.to_string())))
}

fn is_url(spec: &str) -> bool {
    spec.starts_with("https://") || spec.starts_with("http://") || spec.starts_with("ssh://")
}

fn is_rejected_local_spec(spec: &str) -> bool {
    spec.starts_with("file://")
        || spec.starts_with('/')
        || spec.starts_with("./")
        || spec.starts_with("../")
}

fn split_provider_shorthand(spec: &str) -> Option<(GitProvider, &str)> {
    let (provider, path) = spec.split_once(':')?;
    let provider = match provider {
        "github" => GitProvider::Github,
        "gitlab" => GitProvider::Gitlab,
        "bitbucket" => GitProvider::Bitbucket,
        _ => return None,
    };
    Some((provider, path))
}

fn split_host_shorthand(spec: &str) -> Option<(&str, &str)> {
    let (host, path) = spec.split_once(':')?;
    if host.is_empty() || path.is_empty() || !path.contains('/') {
        return None;
    }
    Some((host, path))
}

fn validate_url_shape(url: &str) -> Result<()> {
    if url.starts_with("ssh://") || url.starts_with("http://") || url.starts_with("https://") {
        return Ok(());
    }
    bail!("Unsupported Git URL syntax")
}

fn reject_embedded_credentials(url: &str) -> Result<()> {
    let Some((scheme, rest)) = url.split_once("://") else {
        return Ok(());
    };
    if !matches!(scheme, "http" | "https" | "ssh") {
        return Ok(());
    }
    let authority = rest.split('/').next().unwrap_or_default();
    let Some(userinfo) = authority.split_once('@').map(|(userinfo, _)| userinfo) else {
        return Ok(());
    };
    if userinfo.contains(':') || scheme != "ssh" {
        bail!(
            "Git template URLs must not contain embedded credentials. Use SSH remotes or a Git credential helper instead."
        );
    }
    Ok(())
}

fn validate_scp_like(spec: &str) -> Result<()> {
    let Some(rest) = spec.strip_prefix("git@") else {
        bail!("SSH Git remotes must start with git@");
    };
    let Some((host, path)) = rest.split_once(':') else {
        bail!("SSH Git remote must use git@host:path syntax");
    };
    validate_host(host)?;
    validate_remote_path(path)
}

fn validate_host(host: &str) -> Result<()> {
    if host.trim().is_empty() || host.contains('/') || host.contains('@') || host.contains(':') {
        bail!("Invalid Git host shorthand host {host:?}");
    }
    Ok(())
}

fn validate_remote_path(path: &str) -> Result<()> {
    let path = Path::new(path);
    reject_parent_components(path, "Git remote path")?;
    if path.components().count() < 2 {
        bail!(
            "Git remote shorthand must include owner and repository, got {:?}",
            path.display().to_string()
        );
    }
    Ok(())
}

fn build_provider_url(provider: GitProvider, path: &str, config: &GlobalConfig) -> String {
    let host = config
        .plato
        .git_hosts
        .get(provider)
        .unwrap_or_else(|| public_host(provider));
    build_host_url(host, path)
}

fn build_host_url(host: &str, path: &str) -> String {
    if path.ends_with(".git") {
        return format!("git@{host}:{path}");
    }
    format!("git@{host}:{path}.git")
}

fn public_host(provider: GitProvider) -> &'static str {
    match provider {
        GitProvider::Github => "github.com",
        GitProvider::Gitlab => "gitlab.com",
        GitProvider::Bitbucket => "bitbucket.org",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_default_provider_shorthand() {
        let config = GlobalConfig::default();
        let spec = parse_git_spec_parts("owner/repo", &config).unwrap();
        assert_eq!(spec.url, "git@github.com:owner/repo.git");
    }

    #[test]
    fn parses_inline_revision() {
        let config = GlobalConfig::default();
        let spec = parse_git_spec_parts("gitlab:owner/repo#v1", &config).unwrap();
        assert_eq!(spec.url, "git@gitlab.com:owner/repo.git");
        assert_eq!(spec.inline_revision.as_deref(), Some("v1"));
    }

    #[test]
    fn rejects_empty_inline_revision() {
        let config = GlobalConfig::default();
        let error = parse_git_spec_parts("gitlab:owner/repo#", &config).unwrap_err();
        assert!(error.to_string().contains("must not be empty"));
    }

    #[test]
    fn supports_nested_gitlab_groups() {
        let config = GlobalConfig::default();
        let spec = parse_git_spec_parts("gitlab:owner/group/repo", &config).unwrap();
        assert_eq!(spec.url, "git@gitlab.com:owner/group/repo.git");
    }

    #[test]
    fn supports_host_shorthand() {
        let config = GlobalConfig::default();
        let spec = parse_git_spec_parts("gitlab.company.test:owner/group/repo", &config).unwrap();
        assert_eq!(spec.url, "git@gitlab.company.test:owner/group/repo.git");
    }

    #[test]
    fn supports_scp_like_ssh() {
        let config = GlobalConfig::default();
        let spec =
            parse_git_spec_parts("git@gitlab.company.test:owner/group/repo.git", &config).unwrap();
        assert_eq!(spec.url, "git@gitlab.company.test:owner/group/repo.git");
    }

    #[test]
    fn rejects_embedded_http_credentials_without_leaking_url() {
        let config = GlobalConfig::default();
        let error = parse_git_spec_parts("https://user:secret@example.com/org/repo.git", &config)
            .unwrap_err();
        assert!(error.to_string().contains("embedded credentials"));
        assert!(!error.to_string().contains("secret"));
    }

    #[test]
    fn rejects_embedded_ssh_password() {
        let config = GlobalConfig::default();
        let error = parse_git_spec_parts("ssh://user:secret@example.com/org/repo.git", &config)
            .unwrap_err();
        assert!(error.to_string().contains("embedded credentials"));
    }

    #[test]
    fn allows_ssh_user_without_password() {
        let config = GlobalConfig::default();
        let spec = parse_git_spec_parts("ssh://git@example.com/org/repo.git", &config).unwrap();
        assert_eq!(spec.url, "ssh://git@example.com/org/repo.git");
    }

    #[test]
    fn rejects_parent_component_but_allows_dotdot_name() {
        let config = GlobalConfig::default();
        parse_git_spec_parts("github:foo..bar/repo", &config).unwrap();
        let error = parse_git_spec_parts("github:owner/../repo", &config).unwrap_err();
        assert!(error.to_string().contains("'..'"));
    }

    #[test]
    fn cli_revision_overrides_inline_and_config() {
        let config = GlobalConfig::default();
        let spec = merge_git_template_spec(
            "owner/repo#inline",
            &config,
            Some("configured"),
            None,
            Some("cli"),
            None,
        )
        .unwrap();
        assert_eq!(spec.revision.as_deref(), Some("cli"));
    }
}
