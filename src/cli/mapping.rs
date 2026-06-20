use anyhow::{Result, anyhow};
use std::path::PathBuf;

use plato::InitSource;

const DEFAULT_VALIDATION_PROJECT_NAME: &str = "plato-validation";

pub(crate) fn map_init_source_args(
    template_name: Option<String>,
    project_name: Option<String>,
    path: Option<PathBuf>,
    git: bool,
) -> Result<(InitSource, String)> {
    if let Some(template_path) = path {
        if git {
            return Err(anyhow!("--git and --path cannot be used together."));
        }
        let project_name = template_name.ok_or_else(|| {
            anyhow!("When passing --path 'path' only a single additional arg 'project_name' is expected.")
        })?;
        return Ok((InitSource::TemplatePath { template_path }, project_name));
    }

    let project_name = project_name.ok_or_else(|| {
        anyhow!(
            "When running without --path please pass 'template_name' and 'project_name' as args."
        )
    })?;
    let template_name = template_name.ok_or_else(|| {
        anyhow!(
            "When running without --path please pass 'template_name' and 'project_name' as args."
        )
    })?;
    if git {
        return Ok((
            InitSource::GitTemplate {
                git_spec: template_name,
            },
            project_name,
        ));
    }
    Ok((InitSource::NamedTemplate { template_name }, project_name))
}

pub(crate) fn map_validate_source_args(
    template_name: Option<String>,
    project_name: Option<String>,
    path: Option<PathBuf>,
    git: bool,
) -> Result<(InitSource, String)> {
    if let Some(template_path) = path {
        if git {
            return Err(anyhow!("--git and --path cannot be used together."));
        }

        let project_name = match (template_name, project_name) {
            (None, None) => DEFAULT_VALIDATION_PROJECT_NAME.to_string(),
            (Some(project_name), None) | (None, Some(project_name)) => project_name,
            (Some(_), Some(_)) => {
                return Err(anyhow!(
                    "When passing --path to 'plato val', pass at most one optional project_name arg."
                ));
            }
        };

        return Ok((InitSource::TemplatePath { template_path }, project_name));
    }

    let template_name = template_name.ok_or_else(|| {
        anyhow!("When running 'plato val' without --path please pass a template_name arg.")
    })?;
    let project_name = project_name.unwrap_or_else(|| DEFAULT_VALIDATION_PROJECT_NAME.to_string());
    if git {
        return Ok((
            InitSource::GitTemplate {
                git_spec: template_name,
            },
            project_name,
        ));
    }
    Ok((InitSource::NamedTemplate { template_name }, project_name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_named_template_args() {
        let result = map_init_source_args(
            Some("template_name".to_string()),
            Some("project_name".to_string()),
            None,
            false,
        )
        .unwrap();

        let InitSource::NamedTemplate { template_name } = result.0 else {
            panic!("Expected InitSource::NamedTemplate");
        };

        assert_eq!(template_name, "template_name");
        assert_eq!(result.1, "project_name");
    }

    #[test]
    fn maps_path_template_args() {
        let result = map_init_source_args(
            Some("project_name".to_string()),
            None,
            Some(PathBuf::from("/some/path")),
            false,
        )
        .unwrap();

        let InitSource::TemplatePath { template_path } = result.0 else {
            panic!("Expected InitSource::TemplatePath");
        };

        assert_eq!(template_path, PathBuf::from("/some/path"));
        assert_eq!(result.1, "project_name");
    }

    #[test]
    fn maps_validation_without_project_name() {
        let result =
            map_validate_source_args(Some("template_name".to_string()), None, None, false).unwrap();

        let InitSource::NamedTemplate { template_name } = result.0 else {
            panic!("Expected InitSource::NamedTemplate");
        };

        assert_eq!(template_name, "template_name");
        assert_eq!(result.1, DEFAULT_VALIDATION_PROJECT_NAME);
    }

    #[test]
    fn maps_path_validation_without_project_name() {
        let result =
            map_validate_source_args(None, None, Some(PathBuf::from("/some/path")), false).unwrap();

        let InitSource::TemplatePath { template_path } = result.0 else {
            panic!("Expected InitSource::TemplatePath");
        };

        assert_eq!(template_path, PathBuf::from("/some/path"));
        assert_eq!(result.1, DEFAULT_VALIDATION_PROJECT_NAME);
    }
}
