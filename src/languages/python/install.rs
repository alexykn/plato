use anyhow::{Result, bail};

use crate::{config::PythonInstallConfig, languages::python::PythonProjectScope};

const MODERN_PROJECT_GROUPS_ERROR: &str = "python.install.groups require a modern pyproject project. Add a [project] table to pyproject.toml and set [python].package_manager = \"uv\", or remove python.install.groups.";
const REQUIREMENTS_FILE_INSTALL_ERROR: &str = "python.install options require a pyproject.toml with a [project] table when [python].project_scope = \"requirements\". Add project metadata and use [python].package_manager = \"uv\", or remove python.install.";

pub(crate) fn ensure_supported_scope(
    project_scope: PythonProjectScope,
    install: &PythonInstallConfig,
) -> Result<()> {
    if install.is_empty() || project_scope != PythonProjectScope::Base {
        return Ok(());
    }

    bail!(
        "python.install options require [python].project_scope = \"install\", \"requirements\", or \"auto\" resolving to one of those scopes."
    );
}

pub(crate) fn ensure_no_install_options_for_requirements_file(
    install: &PythonInstallConfig,
) -> Result<()> {
    if install.is_empty() {
        return Ok(());
    }

    bail!(REQUIREMENTS_FILE_INSTALL_ERROR);
}

pub(crate) fn ensure_no_groups_for_editable_install(install: &PythonInstallConfig) -> Result<()> {
    if install.groups.is_empty() {
        return Ok(());
    }

    bail!(MODERN_PROJECT_GROUPS_ERROR);
}

pub(crate) fn editable_install_target(extras: &[String]) -> String {
    if extras.is_empty() {
        return ".".to_string();
    }

    format!(".[{}]", extras.join(","))
}

pub(crate) fn extend_group_args(args: &mut Vec<String>, groups: &[String]) {
    for group in groups {
        args.push("--group".to_string());
        args.push(group.clone());
    }
}

pub(crate) fn extend_uv_sync_args(args: &mut Vec<String>, install: &PythonInstallConfig) {
    extend_group_args(args, &install.groups);

    for extra in &install.extras {
        args.push("--extra".to_string());
        args.push(extra.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn install_config(groups: &[&str], extras: &[&str]) -> PythonInstallConfig {
        PythonInstallConfig {
            groups: groups.iter().map(ToString::to_string).collect(),
            extras: extras.iter().map(ToString::to_string).collect(),
        }
    }

    #[test]
    fn editable_install_target_includes_extras_when_configured() {
        assert_eq!(editable_install_target(&[]), ".");
        assert_eq!(editable_install_target(&["cli".to_string()]), ".[cli]");
        assert_eq!(
            editable_install_target(&["cli".to_string(), "postgres".to_string()]),
            ".[cli,postgres]"
        );
    }

    #[test]
    fn uv_sync_args_include_groups_and_extras() {
        let mut args = vec!["sync".to_string()];
        let install = install_config(&["dev", "lint"], &["cli"]);

        extend_uv_sync_args(&mut args, &install);

        assert_eq!(
            args,
            [
                "sync", "--group", "dev", "--group", "lint", "--extra", "cli"
            ]
        );
    }

    #[test]
    fn install_options_reject_base_scope() {
        let install = install_config(&[], &["cli"]);

        assert!(ensure_supported_scope(PythonProjectScope::Install, &install).is_ok());
        assert!(ensure_supported_scope(PythonProjectScope::Requirements, &install).is_ok());
        assert!(ensure_supported_scope(PythonProjectScope::Base, &install).is_err());
    }

    #[test]
    fn empty_install_options_allow_any_scope() {
        let install = install_config(&[], &[]);

        assert!(ensure_supported_scope(PythonProjectScope::Install, &install).is_ok());
        assert!(ensure_supported_scope(PythonProjectScope::Base, &install).is_ok());
        assert!(ensure_supported_scope(PythonProjectScope::Requirements, &install).is_ok());
    }

    #[test]
    fn editable_install_rejects_groups() {
        let install = install_config(&["dev"], &[]);
        let error = ensure_no_groups_for_editable_install(&install).unwrap_err();

        assert!(error.to_string().contains("modern pyproject project"));
    }

    #[test]
    fn requirements_file_install_rejects_install_options() {
        let install = install_config(&[], &["cli"]);
        let error = ensure_no_install_options_for_requirements_file(&install).unwrap_err();

        assert!(
            error
                .to_string()
                .contains("project_scope = \"requirements\"")
        );
    }
}
