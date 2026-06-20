use anyhow::{Context, Result};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;
use walkdir::WalkDir;

use crate::config::PathReplacementConfig;
use crate::rendering::new_template_environment;
use crate::workspace::TemplateContext;
use crate::workspace::content::FileContent;
use crate::workspace::path_rewrite::{PathRewritePlan, SourcePathEntry, SourcePathKind};
use crate::workspace::rendered::RenderedWorkspace;

fn deduplicate_dirmap(map: &mut HashMap<PathBuf, FileContent>) {
    let all_paths: Vec<PathBuf> = map.keys().cloned().collect();
    map.retain(|path, content| {
        if !matches!(content, FileContent::None) {
            return true;
        }
        let has_children = all_paths
            .iter()
            .any(|other| other != path && other.starts_with(path));
        !has_children
    });
}

pub(crate) struct WorkspaceBuilder {
    content: HashMap<PathBuf, FileContent>,
}

impl WorkspaceBuilder {
    pub(crate) fn from_source(source_path: &Path) -> Result<Self> {
        let mut raw_map = HashMap::new();
        for entry in WalkDir::new(source_path)
            .into_iter()
            .filter_map(std::result::Result::ok)
        {
            let path = entry.path();
            let rel_path = path.strip_prefix(source_path)?.to_path_buf();
            if rel_path.as_os_str().is_empty() {
                continue;
            }
            if matches!(
                path.file_name().and_then(|name| name.to_str()),
                Some("plato.toml")
            ) {
                continue;
            }
            let content = if path.is_dir() {
                FileContent::None
            } else {
                match path.extension().and_then(|s| s.to_str()) {
                    Some("j2" | "mj") => {
                        let text = read_to_string(path).with_context(|| {
                            format!("Failed to read template {}", path.display())
                        })?;
                        FileContent::Template(Rc::<str>::from(text))
                    }
                    _ => FileContent::BinaryLazy {
                        path: path.to_path_buf(),
                        cache: OnceLock::new(),
                    },
                }
            };
            raw_map.insert(rel_path, content);
        }
        Ok(Self { content: raw_map })
    }

    pub(crate) fn rewrite_paths(
        self,
        context: &TemplateContext,
        path_replacements: &BTreeMap<String, PathReplacementConfig>,
    ) -> Result<Self> {
        let source_entries = self
            .content
            .iter()
            .map(|(path, content)| SourcePathEntry {
                path: path.clone(),
                kind: if matches!(content, FileContent::None) {
                    SourcePathKind::Directory
                } else {
                    SourcePathKind::File
                },
            })
            .collect::<Vec<_>>();
        let rewrite_plan =
            PathRewritePlan::from_config(path_replacements, context, &source_entries)?;
        let mut target_map = HashMap::new();
        for (rel_path, content) in self.content {
            let new_path = rewrite_plan.rewrite(&rel_path);
            if target_map.insert(new_path.clone(), content).is_some() {
                return Err(anyhow::anyhow!(
                    "Duplicate path after rewrite: {}",
                    new_path.display()
                ));
            }
        }
        deduplicate_dirmap(&mut target_map);
        Ok(Self {
            content: target_map,
        })
    }

    pub(crate) fn render_templates(self, context: &impl Serialize) -> Result<Self> {
        let mut rendered_map = HashMap::new();
        let env = new_template_environment();
        for (path, content) in self.content {
            match content {
                FileContent::Template(raw_text) => {
                    let rendered = env
                        .render_str(&raw_text, context)
                        .with_context(|| format!("Failed to render {}", path.display()))?;
                    let new_path = path.with_extension("");
                    if rendered_map
                        .insert(
                            new_path.clone(),
                            FileContent::Binary(Rc::from(rendered.into_bytes())),
                        )
                        .is_some()
                    {
                        return Err(anyhow::anyhow!(
                            "Duplicate file after rendering: {}",
                            new_path.display()
                        ));
                    }
                }
                other => {
                    rendered_map.insert(path, other);
                }
            }
        }

        Ok(Self {
            content: rendered_map,
        })
    }

    pub(crate) fn build(self) -> RenderedWorkspace {
        RenderedWorkspace::new(self.content)
    }
}
