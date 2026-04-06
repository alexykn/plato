use anyhow::{Context, Result};
use minijinja::Environment;
use serde::Serialize;
use std::collections::HashMap;
use std::fs::{create_dir_all, read, read_to_string, write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, PartialEq)]
pub(crate) enum FileContent {
    Binary(Vec<u8>),
    Template(String),
    None, // Here None means it's a directory
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

fn render_paths(
    raw_map: HashMap<PathBuf, FileContent>,
    context: &HashMap<&str, &str>,
) -> HashMap<PathBuf, FileContent> {
    let mut target_map = HashMap::new();
    for (rel_path, content) in raw_map {
        let mut path_str = rel_path.to_string_lossy().into_owned();
        for (keyword, replacement) in context {
            let pattern = format!("#{keyword}#");
            path_str = path_str.replace(&pattern, replacement);
        }
        target_map.insert(PathBuf::from(path_str), content);
    }
    deduplicate_dirmap(&mut target_map);
    target_map
}

fn render_templates(
    target_map: HashMap<PathBuf, FileContent>,
    context: &impl Serialize,
) -> Result<HashMap<PathBuf, FileContent>> {
    let mut rendered_map = HashMap::new();
    let env = Environment::new();
    for (path, content) in target_map {
        match content {
            FileContent::Template(raw_text) => {
                let rendered = env
                    .render_str(&raw_text, context)
                    .context(format!("Failed to render {}", path.display()))?;
                let new_path = path.with_extension("");
                rendered_map.insert(new_path, FileContent::Binary(rendered.into_bytes()));
            }
            _ => {
                rendered_map.insert(path, content);
            }
        }
    }
    Ok(rendered_map)
}

pub(super) fn build_target_map(
    raw_map: HashMap<PathBuf, FileContent>,
    path_context: &HashMap<&str, &str>,
    template_context: &impl Serialize,
) -> Result<HashMap<PathBuf, FileContent>> {
    let target_map = render_paths(raw_map, path_context);
    let rendered_map = render_templates(target_map, template_context)?;
    Ok(rendered_map)
}

pub(super) fn scan_source_map(source: &Path) -> Result<HashMap<PathBuf, FileContent>> {
    let mut raw_map = HashMap::new();

    for entry in WalkDir::new(source)
        .into_iter()
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        let rel_path = path.strip_prefix(source)?.to_path_buf();

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
                Some("j2" | "mj") => FileContent::Template(read_to_string(path)?),
                _ => FileContent::Binary(read(path)?),
            }
        };

        raw_map.insert(rel_path, content);
    }

    Ok(raw_map)
}

pub(super) fn flush_to_disk(
    target_map: &HashMap<PathBuf, FileContent>,
    target: &Path,
) -> Result<()> {
    for (path, content) in target_map {
        let full_path = target.join(path);
        match content {
            FileContent::Binary(bytes) => {
                if let Some(parent) = full_path.parent() {
                    create_dir_all(parent)?;
                }
                write(full_path, bytes)?;
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
