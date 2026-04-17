use anyhow::{Result, anyhow, bail};
use std::collections::{BTreeMap, HashMap};
use std::path::{Component, PathBuf};

pub(crate) struct TemplateRegistry {
    content: HashMap<String, (PathBuf, bool)>,
}

impl TemplateRegistry {
    /// Builds a registry from the global template directory and any configured extra directories.
    ///
    /// # Errors
    /// Returns an error if reading template directories fails in a way that prevents building the registry.
    pub(crate) fn build(global_template_dir: &PathBuf, extra_template_dirs: &Vec<PathBuf>) -> Self {
        let mut dirs_to_check = vec![global_template_dir];
        for dir in extra_template_dirs {
            if !dir.exists() || !dir.is_dir() {
                eprintln!(
                    "WARNING: Defined extra directory {} does not exist, skipping.",
                    dir.display()
                );
                continue;
            }
            if dir
                .components()
                .any(|c| matches!(c, Component::CurDir | Component::ParentDir))
            {
                eprintln!(
                    "WARNING: Malformed extra directory {}, skipping.",
                    dir.display()
                );
                continue;
            }
            dirs_to_check.push(dir);
        }
        let mut templates: HashMap<String, (PathBuf, bool)> = HashMap::new();
        for dir in &dirs_to_check {
            let Ok(entries) = std::fs::read_dir(dir) else {
                eprintln!("WARNING: cannot read dir {}", dir.display());
                continue;
            };
            templates.extend(entries.filter_map(|result| {
                let entry = result.inspect_err(|e| println!("WARNING: {e}")).ok()?;
                // file_type() is cached by read_dir on Unix/Windows, no extra stat
                let ft = entry.file_type().ok()?;
                if !ft.is_dir() {
                    return None;
                }
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().into_owned();
                let valid = path.join("plato.toml").is_file();

                Some((name, (path, valid)))
            }));
        }
        Self { content: templates }
    }

    /// Lists the templates in the given Plato config directory.
    ///
    /// # Errors
    /// Returns an error if the registry is invalid or cannot be displayed.
    pub(crate) fn display(&self) {
        let mut ordered_map: BTreeMap<PathBuf, Vec<(&String, &bool)>> = BTreeMap::new();
        for (name, (path, is_valid)) in &self.content {
            let Some(parent) = path.parent() else {
                continue;
            };
            ordered_map
                .entry(parent.to_path_buf())
                .or_default()
                .push((name, is_valid));
        }

        let max_length = self
            .content
            .keys()
            .map(std::string::String::len)
            .max()
            .unwrap_or(0);

        for (folder, mut templates) in ordered_map {
            println!("{}", folder.display());

            templates.sort_by_key(|(name, _)| *name);
            for (name, is_valid) in templates {
                let status = if *is_valid {
                    "ok"
                } else {
                    "plato.toml missing"
                };
                println!(" - {name:<max_length$} | {status}");
            }
            println!();
        }
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
    pub(crate) fn get(&self, name: &str) -> Result<&(PathBuf, bool)> {
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
        let (path, is_valid) = self
            .content
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("No template found for {name:?}"))?;

        if !is_valid {
            bail!(
                "Template at {} does not contain a plato.toml",
                path.display()
            )
        }
        Ok(path)
    }
}
