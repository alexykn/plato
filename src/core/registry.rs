use anyhow::{Result, anyhow, bail};
use std::collections::{BTreeMap, HashMap};
use std::fs::read_dir;
use std::path::{Component, PathBuf};

pub struct TemplateRegistry {
    content: HashMap<String, (PathBuf, bool)>,
}

impl TemplateRegistry {
    /// Builds a registry from the global template directory and any configured extra directories.
    ///
    /// # Errors
    /// Returns an error if reading template directories fails in a way that prevents building the registry.
    pub fn build(
        global_template_dir: &PathBuf,
        extra_template_dirs: &Vec<PathBuf>,
    ) -> Result<Self> {
        let mut dirs_to_check = vec![global_template_dir];
        for dir in extra_template_dirs {
            if !dir.exists() || !dir.is_dir() {
                println!(
                    "WARNING: Defined extra directory {} does not exist, skipping.",
                    dir.display()
                );
                continue;
            }
            if dir
                .components()
                .any(|c| matches!(c, Component::CurDir | Component::ParentDir))
                || !dir.is_absolute()
            {
                println!(
                    "WARNING: Malformed extra directory {}, skipping.",
                    dir.display()
                );
                continue;
            }
            dirs_to_check.push(dir);
        }
        let mut templates: HashMap<String, (PathBuf, bool)> = HashMap::new();
        for dir in dirs_to_check {
            if let Ok(entries) = read_dir(dir) {
                let iter = entries
                    .filter_map(Result::ok)
                    .filter(|entry| entry.path().is_dir())
                    .map(|entry| {
                        let dir = entry.path();
                        let name = entry.file_name().to_string_lossy().into_owned();
                        let valid = entry.path().join("plato.toml").exists();
                        (name, (dir, valid))
                    });
                templates.extend(iter);
            }
        }
        Ok(Self { content: templates })
    }

    /// Lists the templates in the given Plato config directory.
    ///
    /// # Errors
    /// Returns an error if the registry is invalid or cannot be displayed.
    pub fn display(&self) -> Result<()> {
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
        }
        Ok(())
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
    pub fn get(&self, name: &str) -> Result<&(PathBuf, bool)> {
        self.check_self_is_empty()?;
        self.content
            .get(name)
            .ok_or_else(|| anyhow!("No template found for {name:?}"))
    }

    /// Returns all discovered templates.
    ///
    /// # Errors
    /// Returns an error if the registry is empty.
    pub fn get_all(&self) -> Result<&HashMap<String, (PathBuf, bool)>> {
        self.check_self_is_empty()?;
        Ok(&self.content)
    }

    /// Returns the directory of a valid template.
    ///
    /// # Errors
    /// Returns an error if the registry is empty, the named template does not exist,
    /// or the template does not contain a `plato.toml` file.
    pub fn get_config_path(&self, name: &str) -> Result<&PathBuf> {
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
