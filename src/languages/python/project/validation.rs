use anyhow::Result;

use crate::config::{PythonInstallConfig, PythonPackageManagerConfig, PythonProjectScopeConfig};
use crate::languages::python::{PythonPackageManager, PythonProjectScope, PythonSetupContext};
use crate::validation::files::ProjectFiles;
use crate::validation::project::ValidationProjectContext;
use crate::validation::report::{ValidationIssue, ValidationReport};

use super::{
    metadata::{PythonProjectMetadata, load_python_project_metadata},
    plan::{PythonSetupMode, PythonSetupPlan, resolve_python_setup_plan},
};

pub(crate) fn validate_python_project(
    ctx: &ValidationProjectContext,
    files: &impl ProjectFiles,
) -> Result<ValidationReport> {
    let metadata = load_python_project_metadata(files)?;
    let python_ctx = PythonSetupContext {
        target_path: std::env::current_dir()?.join(&ctx.project_name),
        config: ctx.config.clone(),
        project_scope: resolve_project_scope(files, ctx),
        package_manager: resolve_package_manager(ctx),
    };
    let plan = resolve_python_setup_plan(&python_ctx, &metadata)?;

    Ok(validate_python_setup(&python_ctx, files, &metadata, plan))
}

pub(crate) fn validate_python_setup(
    ctx: &PythonSetupContext,
    files: &impl ProjectFiles,
    metadata: &PythonProjectMetadata,
    plan: PythonSetupPlan,
) -> ValidationReport {
    let mut issues = ValidationReport::default();

    validate_install_options(ctx, metadata, plan, &mut issues);
    validate_readme(files, metadata, &mut issues);

    issues
}

fn validate_install_options(
    ctx: &PythonSetupContext,
    metadata: &PythonProjectMetadata,
    plan: PythonSetupPlan,
    issues: &mut ValidationReport,
) {
    let install = &ctx.config.python.install;

    if install.is_empty() {
        return;
    }

    if plan.mode == PythonSetupMode::Base {
        issues.push(ValidationIssue::new(
            "python-install-options-unsupported-scope",
            "python.install options require a Python setup scope that installs dependencies.",
            Some(
                "Set [python].project_scope to \"install\", \"requirements\", or \"auto\" resolving to one of those scopes, or remove python.install."
                    .to_string(),
            ),
        ));
        return;
    }

    validate_groups(install, metadata, plan, issues);
    validate_extras(install, metadata, plan, issues);
}

fn validate_groups(
    install: &PythonInstallConfig,
    metadata: &PythonProjectMetadata,
    plan: PythonSetupPlan,
    issues: &mut ValidationReport,
) {
    if install.groups.is_empty() {
        return;
    }

    if !matches!(plan.mode, PythonSetupMode::UvSync { .. }) {
        issues.push(ValidationIssue::new(
            "python-install-groups-unsupported",
            "python.install.groups require a modern pyproject project setup.",
            Some(
                "Add a [project] table and matching [dependency-groups] entries to pyproject.toml, use [python].package_manager = \"uv\", or remove python.install.groups."
                    .to_string(),
            ),
        ));
        return;
    }

    let Some(pyproject) = metadata.pyproject() else {
        return;
    };

    for group in &install.groups {
        if pyproject.dependency_groups.contains(group) {
            continue;
        }

        issues.push(ValidationIssue::new(
            "python-install-group-missing",
            format!(
                "Configured python.install.groups contains \"{group}\", but pyproject.toml has no [dependency-groups].{group} entry."
            ),
            Some(format!(
                "Add {group} = [...] under [dependency-groups], or remove \"{group}\" from python.install.groups."
            )),
        ));
    }
}

fn validate_extras(
    install: &PythonInstallConfig,
    metadata: &PythonProjectMetadata,
    plan: PythonSetupPlan,
    issues: &mut ValidationReport,
) {
    if install.extras.is_empty() {
        return;
    }

    if matches!(plan.mode, PythonSetupMode::RequirementsFile { .. }) {
        issues.push(ValidationIssue::new(
            "python-install-extras-unsupported",
            "python.install.extras cannot be applied to requirements-file setup.",
            Some(
                "Use a pyproject.toml [project] table with [project.optional-dependencies] and a modern setup path, or remove python.install.extras."
                    .to_string(),
            ),
        ));
        return;
    }

    if !matches!(plan.mode, PythonSetupMode::UvSync { .. }) {
        return;
    }

    let Some(pyproject) = metadata.pyproject() else {
        return;
    };

    for extra in &install.extras {
        if pyproject.optional_dependencies.contains(extra) {
            continue;
        }

        issues.push(ValidationIssue::new(
            "python-install-extra-missing",
            format!(
                "Configured python.install.extras contains \"{extra}\", but pyproject.toml has no [project.optional-dependencies].{extra} entry."
            ),
            Some(format!(
                "Add {extra} = [...] under [project.optional-dependencies], or remove \"{extra}\" from python.install.extras."
            )),
        ));
    }
}

fn validate_readme(
    files: &impl ProjectFiles,
    metadata: &PythonProjectMetadata,
    issues: &mut ValidationReport,
) {
    let Some(pyproject) = metadata.pyproject() else {
        return;
    };
    let Some(readme_path) = pyproject.readme_path.as_ref() else {
        return;
    };

    if files.exists(readme_path) {
        return;
    }

    issues.push(ValidationIssue::new(
        "python-readme-missing",
        format!(
            "{} references readme file {}, but that file does not exist.",
            pyproject.path.display(),
            readme_path.display()
        ),
        Some(format!(
            "Create {} in the rendered project, update [project].readme, or remove the readme reference.",
            readme_path.display()
        )),
    ));
}

fn resolve_package_manager(ctx: &ValidationProjectContext) -> PythonPackageManager {
    match ctx.config.python.package_manager {
        PythonPackageManagerConfig::Pip => PythonPackageManager::Pip,
        PythonPackageManagerConfig::Uv | PythonPackageManagerConfig::Auto => {
            PythonPackageManager::Uv
        }
    }
}

fn resolve_project_scope(
    files: &impl ProjectFiles,
    ctx: &ValidationProjectContext,
) -> PythonProjectScope {
    match ctx.config.python.project_scope {
        PythonProjectScopeConfig::Base => PythonProjectScope::Base,
        PythonProjectScopeConfig::Install => PythonProjectScope::Install,
        PythonProjectScopeConfig::Requirements => PythonProjectScope::Requirements,
        PythonProjectScopeConfig::Auto => {
            get_python_project_scope_from_files(files, &ctx.project_name)
        }
    }
}

fn get_python_project_scope_from_files(
    files: &impl ProjectFiles,
    project_name: &str,
) -> crate::languages::python::PythonProjectScope {
    use crate::languages::python::PythonProjectScope::{Base, Install, Requirements};

    let has_pyproject = files.exists(std::path::Path::new("pyproject.toml"));
    let has_requirements = files.exists(std::path::Path::new("requirements.txt"));
    let normalized_project_name = project_name.replace('-', "_");

    let has_src_package = files.exists(std::path::Path::new(&format!(
        "src/{project_name}/__init__.py"
    ))) || files.exists(std::path::Path::new(&format!(
        "src/{normalized_project_name}/__init__.py"
    )));

    let has_flat_package = files
        .exists(std::path::Path::new(&format!("{project_name}/__init__.py")))
        || files.exists(std::path::Path::new(&format!(
            "{normalized_project_name}/__init__.py"
        )));

    if has_pyproject && (has_src_package || has_flat_package) {
        return Install;
    }
    if has_pyproject || has_requirements {
        return Requirements;
    }
    Base
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        config::{Config, PythonInstallConfig},
        languages::{
            SetupPlan,
            python::{
                PythonPackageManager, PythonProjectScope,
                project::{
                    metadata::{PyProjectState, PyProjectTomlMetadata},
                    plan::PythonInstaller,
                },
            },
        },
    };
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    fn install(groups: &[&str], extras: &[&str]) -> PythonInstallConfig {
        PythonInstallConfig {
            groups: groups.iter().map(ToString::to_string).collect(),
            extras: extras.iter().map(ToString::to_string).collect(),
        }
    }

    fn ctx(install: PythonInstallConfig, target_path: PathBuf) -> PythonSetupContext {
        let mut config = Config::default();
        config.python.install = install;

        PythonSetupContext {
            target_path,
            config,
            project_scope: PythonProjectScope::Install,
            package_manager: PythonPackageManager::Uv,
        }
    }

    fn metadata(
        groups: &[&str],
        extras: &[&str],
        readme_path: Option<PathBuf>,
    ) -> PythonProjectMetadata {
        PythonProjectMetadata {
            pyproject: PyProjectState::Present(PyProjectTomlMetadata {
                path: PathBuf::from("pyproject.toml"),
                has_project_table: true,
                dependency_groups: groups.iter().map(ToString::to_string).collect(),
                optional_dependencies: extras.iter().map(ToString::to_string).collect(),
                readme_path,
            }),
        }
    }

    fn temp_dir(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("plato-validation-{name}-{unique}"));
        fs::create_dir(&path).unwrap();
        path
    }

    struct TestFiles {
        readme_exists: bool,
    }

    impl TestFiles {
        const fn empty() -> Self {
            Self {
                readme_exists: false,
            }
        }

        const fn with_readme() -> Self {
            Self {
                readme_exists: true,
            }
        }
    }

    impl ProjectFiles for TestFiles {
        fn exists(&self, path: &Path) -> bool {
            path == Path::new("README.md") && self.readme_exists
        }

        fn read_to_string(&self, _path: &Path) -> Result<String> {
            unreachable!()
        }
    }

    #[test]
    fn accepts_groups_and_extras_for_uv_sync_when_defined() {
        let target = temp_dir("valid-sync");
        fs::write(target.join("README.md"), "readme").unwrap();
        let ctx = ctx(install(&["dev"], &["cli"]), target);
        let metadata = metadata(&["dev"], &["cli"], Some(PathBuf::from("README.md")));
        let plan = SetupPlan::new(PythonSetupMode::UvSync {
            install_project: true,
        });

        validate_python_setup(&ctx, &TestFiles::with_readme(), &metadata, plan)
            .into_result()
            .unwrap();
    }

    #[test]
    fn rejects_groups_for_editable_install() {
        let ctx = ctx(install(&["dev"], &[]), temp_dir("groups-editable"));
        let metadata = metadata(&[], &[], None);
        let plan = SetupPlan::new(PythonSetupMode::EditableInstall {
            installer: PythonInstaller::Pip,
        });

        let error = validate_python_setup(&ctx, &TestFiles::empty(), &metadata, plan)
            .into_result()
            .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("python-install-groups-unsupported")
        );
    }

    #[test]
    fn rejects_missing_group_for_uv_sync() {
        let ctx = ctx(install(&["dev"], &[]), temp_dir("group-missing"));
        let metadata = metadata(&[], &[], None);
        let plan = SetupPlan::new(PythonSetupMode::UvSync {
            install_project: true,
        });

        let error = validate_python_setup(&ctx, &TestFiles::empty(), &metadata, plan)
            .into_result()
            .unwrap_err();

        assert!(error.to_string().contains("python-install-group-missing"));
    }

    #[test]
    fn accepts_extras_for_editable_install_without_metadata_check() {
        let ctx = ctx(install(&[], &["cli"]), temp_dir("editable-extra"));
        let metadata = metadata(&[], &[], None);
        let plan = SetupPlan::new(PythonSetupMode::EditableInstall {
            installer: PythonInstaller::Pip,
        });

        validate_python_setup(&ctx, &TestFiles::with_readme(), &metadata, plan)
            .into_result()
            .unwrap();
    }

    #[test]
    fn rejects_extras_for_requirements_file() {
        let ctx = ctx(install(&[], &["cli"]), temp_dir("requirements-extra"));
        let metadata = metadata(&[], &[], None);
        let plan = SetupPlan::new(PythonSetupMode::RequirementsFile {
            installer: PythonInstaller::Pip,
        });

        let error = validate_python_setup(&ctx, &TestFiles::empty(), &metadata, plan)
            .into_result()
            .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("python-install-extras-unsupported")
        );
    }

    #[test]
    fn rejects_missing_extra_for_uv_sync() {
        let ctx = ctx(install(&[], &["cli"]), temp_dir("extra-missing"));
        let metadata = metadata(&[], &[], None);
        let plan = SetupPlan::new(PythonSetupMode::UvSync {
            install_project: true,
        });

        let error = validate_python_setup(&ctx, &TestFiles::empty(), &metadata, plan)
            .into_result()
            .unwrap_err();

        assert!(error.to_string().contains("python-install-extra-missing"));
    }

    #[test]
    fn rejects_missing_readme() {
        let ctx = ctx(install(&[], &[]), temp_dir("missing-readme"));
        let metadata = metadata(&[], &[], Some(PathBuf::from("README.md")));
        let plan = SetupPlan::new(PythonSetupMode::Base);

        let error = validate_python_setup(&ctx, &TestFiles::empty(), &metadata, plan)
            .into_result()
            .unwrap_err();

        assert!(error.to_string().contains("python-readme-missing"));
    }

    #[test]
    fn rejects_install_options_for_base_scope() {
        let ctx = ctx(install(&[], &["cli"]), temp_dir("base-options"));
        let metadata = metadata(&[], &["cli"], None);
        let plan = SetupPlan::new(PythonSetupMode::Base);

        let error = validate_python_setup(&ctx, &TestFiles::empty(), &metadata, plan)
            .into_result()
            .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("python-install-options-unsupported-scope")
        );
    }
}
