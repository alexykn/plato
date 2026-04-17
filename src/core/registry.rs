use anyhow::{Result, anyhow, bail};
use std::collections::{BTreeMap, HashMap};
use std::fmt::{self, Display, Formatter};
use std::fs::ReadDir;
use std::path::{Component, Path, PathBuf};

use crate::core::config::parse_config;

#[derive(Debug, Clone)]
pub(crate) enum TemplateStatus {
    Valid,
    MissingConfig,
    MalformedConfig,
}

impl Display for TemplateStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::Valid => "ok",
            Self::MissingConfig => "plato.toml missing",
            Self::MalformedConfig => "plato.toml malformed",
        };
        write!(f, "{label}")
    }
}

impl Display for TemplateRegistry {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ordered_map = build_ordered_template_map(&self.content);
        let max_length = self.content.keys().map(String::len).max().unwrap_or(0);

        for (folder, mut templates) in ordered_map {
            writeln!(f, "{}", folder.display())?;

            templates.sort_by_key(|(name, _)| name.clone());
            for (name, status) in templates {
                writeln!(f, " - {name:<max_length$} | {status}")?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

pub(crate) struct TemplateRegistry {
    content: HashMap<String, (PathBuf, TemplateStatus)>,
}

impl TemplateRegistry {
    /// Builds a registry from the global template directory and any configured extra directories.
    ///
    /// # Errors
    /// Returns an error if reading template directories fails in a way that prevents building the registry.
    pub(crate) fn build(global_template_dir: &PathBuf, extra_template_dirs: &[PathBuf]) -> Self {
        let mut dirs_to_check = vec![global_template_dir];
        let validated_extra_dirs = validate_extra_dirs(extra_template_dirs);
        dirs_to_check.extend(validated_extra_dirs);

        let mut templates: HashMap<String, (PathBuf, TemplateStatus)> = HashMap::new();
        for dir in dirs_to_check {
            let Ok(entries) = std::fs::read_dir(dir) else {
                eprintln!("WARNING: cannot read dir {}", dir.display());
                continue;
            };
            templates.extend(yield_templates_from_dir(entries));
        }
        Self { content: templates }
    }

    fn check_self_is_empty(&self) -> Result<()> {
        if self.content.is_empty() {
            bail!("Template registry is empty. Check your ~/.config/plato folder!")
        }
        Ok(())
    }

    /// Returns the template directory and validity flag for a named template.
    ///
    /// # Errors
    /// Returns an error if the registry is empty or the named template does not exist.
    pub(crate) fn get(&self, name: &str) -> Result<&(PathBuf, TemplateStatus)> {
        self.check_self_is_empty()?;
        self.content
            .get(name)
            .ok_or_else(|| anyhow!("No template found for {name:?}"))
    }

    /// Returns the directory of a valid template.
    ///
    /// # Errors
    /// Returns an error if the registry is empty, the named template does not exist,
    /// or the template does not contain a `plato.toml` file.
    pub(crate) fn get_config_path(&self, name: &str) -> Result<&PathBuf> {
        self.check_self_is_empty()?;
        let (path, status) = self
            .content
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("No template found for {name:?}"))?;

        // This function is just for getting the path and also used for opening
        // the file in a editor so we should not error out on malformed config
        match status {
            TemplateStatus::Valid | TemplateStatus::MalformedConfig => Ok(path),
            TemplateStatus::MissingConfig => bail!(
                "Template at {} does not contain a 'plato.toml'",
                path.display()
            ),
        }
    }
}

fn is_valid_dir(dir: &Path) -> bool {
    dir.exists() && dir.is_dir()
}

fn is_regular_dir(dir: &Path) -> bool {
    !dir.components()
        .any(|comp| matches!(comp, Component::CurDir | Component::ParentDir))
}

fn validate_extra_dirs(extra_template_dirs: &[PathBuf]) -> Vec<&PathBuf> {
    let mut validated_dirs = vec![];
    for dir in extra_template_dirs {
        if !is_valid_dir(dir) || !is_regular_dir(dir) {
            eprintln!("Malformed or nonexistent extra directory {}", dir.display());
            continue;
        }
        validated_dirs.push(dir);
    }
    validated_dirs
}

fn validate_template(template_path: &Path) -> TemplateStatus {
    if !template_path.join("plato.toml").is_file() {
        return TemplateStatus::MissingConfig;
    }
    if parse_config(template_path).is_err() {
        return TemplateStatus::MalformedConfig;
    }
    TemplateStatus::Valid
}

fn build_ordered_template_map(
    content: &HashMap<String, (PathBuf, TemplateStatus)>,
) -> BTreeMap<PathBuf, Vec<(String, TemplateStatus)>> {
    let mut ordered_map: BTreeMap<PathBuf, Vec<(String, TemplateStatus)>> = BTreeMap::new();
    for (name, (path, status)) in content {
        let Some(parent) = path.parent() else {
            continue;
        };
        ordered_map
            .entry(parent.to_path_buf())
            .or_default()
            .push((name.clone(), status.clone()));
    }
    ordered_map
}

fn yield_templates_from_dir(
    entries: ReadDir,
) -> impl Iterator<Item = (String, (PathBuf, TemplateStatus))> {
    entries.filter_map(|result| {
        let entry = result.inspect_err(|e| eprintln!("WARNING: {e}")).ok()?;
        // file_type() is cached by read_dir on Unix/Windows, no extra stat
        let ft = entry.file_type().ok()?;
        if !ft.is_dir() {
            return None;
        }
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        let status = validate_template(&path);

        Some((name, (path, status)))
    })
}
