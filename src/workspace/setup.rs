use anyhow::{Context, Ok, Result};
use minijinja::Environment;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::{create_dir_all, read, read_to_string, write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use walkdir::WalkDir;

use crate::workspace::TemplateContext;

pub(crate) enum FileContent {
    BinaryLazy {
        path: PathBuf,
        cache: OnceLock<Arc<[u8]>>,
    },
    Binary(Arc<[u8]>),
    Template(Arc<str>),
    None,
}

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
    pub(super) fn scan(source_path: &Path) -> Result<Self> {
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
                        FileContent::Template(Arc::<str>::from(text))
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

    pub(super) fn render_paths(self, context: &TemplateContext) -> Result<Self> {
        let mut target_map = HashMap::new();
        for (rel_path, content) in self.content {
            let mut path_str = rel_path.to_string_lossy().into_owned();
            for (keyword, replacement) in &context.context {
                let pattern = format!("#{keyword}#");
                path_str = path_str.replace(&pattern, replacement);
            }

            let new_path = PathBuf::from(path_str);
            if target_map.insert(new_path.clone(), content).is_some() {
                return Err(anyhow::anyhow!(
                    "Duplicate path after rendering: {}",
                    new_path.display()
                ));
            }
        }
        deduplicate_dirmap(&mut target_map);
        Ok(Self {
            content: target_map,
        })
    }

    pub(super) fn render_templates(self, context: &impl Serialize) -> Result<Self> {
        let mut rendered_map = HashMap::new();
        let env = Environment::new();
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
                            FileContent::Binary(Arc::from(rendered.into_bytes())),
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

    pub(super) fn flush_to_disk(self, target: &Path) -> Result<()> {
        for (path, content) in self.content {
            let full_path = target.join(&path);
            match content {
                FileContent::BinaryLazy {
                    path: source_path,
                    cache,
                } => {
                    if cache.get().is_none() {
                        let bytes =
                            read(&source_path).map(Arc::<[u8]>::from).with_context(|| {
                                format!("Failed to read binary file {}", source_path.display())
                            })?;
                        let _ = cache.set(bytes);
                    }
                    let bytes = cache
                        .get()
                        .context("Binary cache was not initialized after read")?;
                    if let Some(parent) = full_path.parent() {
                        create_dir_all(parent)?;
                    }
                    write(full_path, bytes.as_ref())?;
                }
                FileContent::Binary(bytes) => {
                    if let Some(parent) = full_path.parent() {
                        create_dir_all(parent)?;
                    }
                    write(full_path, bytes.as_ref())?;
                }
                FileContent::None => {
                    create_dir_all(full_path)?;
                }
                FileContent::Template(_) => {
                    eprintln!(
                        "WARNING: Found unrendered template at {}. Skipping.",
                        path.display()
                    );
                }
            }
        }
        Ok(())
    }
}
