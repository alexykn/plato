use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};
use std::path::{Path, PathBuf};

use crate::core::config::{GlobalConfig, TemplateEntry};
use crate::core::path::expand_tilde;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TemplateKind {
    Path,
    Git,
}

impl Display for TemplateKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path => write!(f, "path"),
            Self::Git => write!(f, "git"),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TemplateRecord {
    pub(crate) name: String,
    pub(crate) entry: TemplateEntry,
    pub(crate) config_override: Option<PathBuf>,
}

impl TemplateRecord {
    pub(crate) fn kind(&self) -> TemplateKind {
        match &self.entry {
            TemplateEntry::Path { .. } => TemplateKind::Path,
            TemplateEntry::Git { .. } => TemplateKind::Git,
        }
    }

    pub(crate) fn override_path(&self) -> Option<&Path> {
        self.config_override.as_deref()
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct TemplateRegistry {
    records: BTreeMap<String, TemplateRecord>,
}

impl TemplateRegistry {
    pub(crate) fn from_config(config: &GlobalConfig) -> Self {
        let records = config
            .templates
            .iter()
            .map(|(name, entry)| {
                let config_override = config
                    .template_configs
                    .get(name)
                    .map(|path| expand_tilde(path).unwrap_or_else(|_| path.clone()));
                (
                    name.clone(),
                    TemplateRecord {
                        name: name.clone(),
                        entry: entry.clone(),
                        config_override,
                    },
                )
            })
            .collect();
        Self { records }
    }

    pub(crate) fn get(&self, name: &str) -> Option<&TemplateRecord> {
        self.records.get(name)
    }

    pub(crate) fn list(&self, verbose: bool) -> String {
        let mut output = String::new();
        output.push_str("NAME TYPE CONFIG\n");
        for record in self.records.values() {
            let config_status = if record.config_override.is_some() {
                "override"
            } else {
                "source/default"
            };
            output.push_str(&format!(
                "{} {} {}\n",
                record.name,
                record.kind(),
                config_status
            ));
            if verbose {
                output.push_str(&format_verbose_record(record));
            }
        }
        output
    }
}

fn format_verbose_record(record: &TemplateRecord) -> String {
    let mut output = String::new();
    match &record.entry {
        TemplateEntry::Path { path } => {
            output.push_str(&format!("  path = {}\n", path.display()));
        }
        TemplateEntry::Git { git, rev, subpath } => {
            output.push_str(&format!("  git = {git}\n"));
            if let Some(rev) = rev {
                output.push_str(&format!("  rev = {rev}\n"));
            }
            if let Some(subpath) = subpath {
                output.push_str(&format!("  subpath = {}\n", subpath.display()));
            }
        }
    }
    if let Some(config_override) = &record.config_override {
        output.push_str(&format!("  config = {}\n", config_override.display()));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::TemplateEntry;
    use std::collections::HashMap;

    #[test]
    fn lists_templates_in_stable_order() {
        let mut config = GlobalConfig::default();
        config.templates = HashMap::from([
            (
                "zed".to_string(),
                TemplateEntry::Path {
                    path: PathBuf::from("/tmp/zed"),
                },
            ),
            (
                "api".to_string(),
                TemplateEntry::Git {
                    git: "github:owner/repo".to_string(),
                    rev: None,
                    subpath: None,
                },
            ),
        ]);
        let registry = TemplateRegistry::from_config(&config);
        let output = registry.list(false);
        assert!(output.find("api git").unwrap() < output.find("zed path").unwrap());
    }

    #[test]
    fn verbose_list_includes_source_details() {
        let mut config = GlobalConfig::default();
        config.templates.insert(
            "api".to_string(),
            TemplateEntry::Git {
                git: "github:owner/repo".to_string(),
                rev: Some("main".to_string()),
                subpath: Some(PathBuf::from("templates/api")),
            },
        );
        config
            .template_configs
            .insert("api".to_string(), PathBuf::from("~/api.toml"));
        let output = TemplateRegistry::from_config(&config).list(true);
        assert!(output.contains("git = github:owner/repo"));
        assert!(output.contains("rev = main"));
        assert!(output.contains("subpath = templates/api"));
        assert!(output.contains("config = "));
    }
}
