use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use directories::BaseDirs;
use minijinja::Environment;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::{env, fs, process};
use walkdir::WalkDir;

static ALLOWED_CMD_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(uv|python\d*(?:\.\d+)*)$").expect("Invalid regex pattern"));

/// Plato: A cool project templating tool
#[derive(Parser, Debug)]
#[command(name = "plato")]
#[command(about = "Scaffolds projects from ~/.config/plato", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize new project from a template
    Init {
        /// The name of the template (e.g., py3.12)
        template: String,

        /// The name of the new project directory
        project_name: String,
    },
}

#[derive(Deserialize, Debug)]
struct PlatoToml {
    ttype: String,
    version: Option<String>,
}

#[derive(Serialize)]
struct TemplateContext<'a> {
    project_name: &'a str,
    project_version: &'a str,
    version: &'a str,
}

#[derive(Debug, PartialEq)]
pub enum FileContent {
    Binary(Vec<u8>),
    Template(String),
    None, // Here None means it's a directory
}

#[derive(Debug, Clone, Copy)]
enum TemplateType {
    Python,
    Rust,
    Base,
}

#[derive(Debug, Clone, Copy)]
enum PythonPackageManager {
    Pip,
    Uv,
    None,
}

#[derive(Debug, Clone, Copy)]
enum ProjectScope {
    Requirements,
    Install,
    Base,
}

#[derive(Deserialize, Debug)]
struct PyProject {
    project: Option<ProjectTable>,
    // The new standard (PEP 735)
    #[serde(rename = "dependency-groups")]
    dependency_groups: Option<HashMap<String, Vec<String>>>,
    // The specific tool tables
    tool: Option<ToolTable>,
}

#[derive(Deserialize, Debug)]
struct ProjectTable {
    dependencies: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
struct ToolTable {
    // Handling the legacy uv fields
    uv: Option<UvTable>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
struct UvTable {
    dev_dependencies: Option<Vec<String>>,
}

struct ProjectGuard {
    path: PathBuf,
    success: bool,
}

impl ProjectGuard {
    fn new(path: PathBuf) -> Self {
        Self {
            path,
            success: false,
        }
    }

    fn release(&mut self) {
        self.success = true
    }
}

impl Drop for ProjectGuard {
    fn drop(&mut self) {
        if !self.success {
            eprintln!("Project Setup did not finish. Cleaning up {:?}", self.path);
            if self.path.exists() {
                let _ = fs::remove_dir_all(&self.path);
            }
        }
    }
}

fn get_python_project_scope(target: &Path, project_name: &str) -> ProjectScope {
    if target.join("pyproject.toml").exists()
        && target
            .join(format!("src/{project_name}/__init__.py"))
            .exists()
    {
        ProjectScope::Install
    } else if target.join("pyproject.toml").exists() || target.join("requirements.txt").exists() {
        ProjectScope::Requirements
    } else {
        ProjectScope::Base
    }
}

fn get_python_manager(version: &str) -> PythonPackageManager {
    if is_installed("uv") {
        return PythonPackageManager::Uv;
    }
    if is_installed(&format!("python{version}").to_string()) {
        return PythonPackageManager::Pip;
    }
    PythonPackageManager::None
}

fn get_config_dir() -> PathBuf {
    let base_dirs = BaseDirs::new().expect("Could not find home directory");
    let mut config_path = base_dirs.home_dir().to_path_buf();
    config_path.push(".config");
    config_path.push("plato");
    config_path
}

fn is_installed(cmd: &str) -> bool {
    process::Command::new(cmd).arg("--help").output().is_ok()
}

fn execute_command(cmd: &str, args: &[&str], target: &Path) -> Result<()> {
    let cmd_name = Path::new(cmd)
        .file_name()
        .and_then(|result| result.to_str())
        .unwrap_or(cmd);
    if !ALLOWED_CMD_RE.is_match(cmd_name) {
        bail!("Selected command '{cmd}' is not allowed");
    }
    process::Command::new(cmd)
        .args(args)
        .current_dir(target)
        .status()
        .context(format!("Unable to run command {cmd}"))?;
    Ok(())
}

fn setup_uv_project(version: &str, target: &Path, scope: ProjectScope) -> Result<()> {
    execute_command("uv", &["venv", "--python", version], target)?;
    match scope {
        ProjectScope::Install => {
            ensure_readme(target)?;
            execute_command("uv", &["sync"], target)
        }
        ProjectScope::Requirements => {
            execute_command("uv", &["sync", "--no-install-project"], target)
        }
        ProjectScope::Base => Ok(()),
    }
}

fn ensure_readme(target: &Path) -> Result<()> {
    let readme = target.join("README.md");
    if !readme.exists() {
        println!(
            "Generating basic readme at {} to satisfy pip",
            &readme.display()
        );
        fs::write(readme, "# Basic readme generated by Plato")?;
    }
    Ok(())
}

fn parse_pyproject(pyproject_path: &Path) -> Result<PyProject> {
    if !pyproject_path.exists() {
        bail!("This shoud not have happened, how did we get here?!")
    }
    let content = fs::read_to_string(pyproject_path).context(format!(
        "Could not pyproject toml at {}",
        pyproject_path.display()
    ))?;
    let pyrproject: PyProject = toml::from_str(&content)?;
    Ok(pyrproject)
}

fn requirements_from_pyproject(target: &Path) -> Result<Vec<String>> {
    let pyproject_path = target.join("pyproject.toml");
    let pyproject = parse_pyproject(&pyproject_path)?;
    let mut requirements: Vec<String> = Vec::new();
    if let Some(project_deps) = pyproject.project.and_then(|x| x.dependencies) {
        requirements.extend(project_deps);
    }
    if let Some(group_deps) = pyproject.dependency_groups {
        for (_, dep_list) in group_deps {
            requirements.extend(dep_list);
        }
    }
    if let Some(uv_deps) = pyproject
        .tool
        .and_then(|t| t.uv)
        .and_then(|uv| uv.dev_dependencies)
    {
        requirements.extend(uv_deps);
    }
    Ok(requirements)
}

fn dev_groups_from_pyproject(target: &Path) -> Result<Vec<String>> {
    let pyproject_path = target.join("pyproject.toml");
    let pyproject = parse_pyproject(&pyproject_path)?;
    let mut dev_groups: Vec<String> = Vec::new();
    if let Some(group_deps) = pyproject.dependency_groups {
        for (group, _) in group_deps {
            dev_groups.push(group);
        }
    }
    Ok(dev_groups)
}

fn ensure_requirements(target: &Path) -> Result<bool> {
    let req_file = target.join("requirements.txt");
    if !req_file.exists() {
        let plato_hidden_path = target.join(".plato/");
        let plato_req_file = target.join(".plato/requirements.txt");
        let requirements = requirements_from_pyproject(target)?;
        let file_content = requirements.join("\n");
        let _ = fs::create_dir_all(plato_hidden_path);
        let _ = fs::write(&plato_req_file, file_content);

        println!(
            "Generating requirements.txt at {} to satisfy pip",
            &plato_req_file.display()
        );
        return Ok(true);
    }
    Ok(false)
}

fn pip_install_requirements(target: &Path) -> Result<()> {
    let python_pip_exec = target.join(".venv/bin/python");
    let requirements = match target.join("requirements.txt").exists() {
        true => target.join("requirements.txt"),
        false if ensure_requirements(target)? => target.join(".plato/requirements.txt"),
        false => bail!("Could not find or generate requirements.txt"),
    };
    let python_pip_args = [
        "-m",
        "pip",
        "install",
        "-r",
        &requirements.to_string_lossy(),
    ];
    execute_command(&python_pip_exec.to_string_lossy(), &python_pip_args, target)?;
    Ok(())
}

fn pip_install_project(target: &Path) -> Result<()> {
    let python_pip_exec = target.join(".venv/bin/python");
    let mut python_pip_args = vec!["-m", "pip", "install", "-e", "."];
    let dev_groups = dev_groups_from_pyproject(target)?;

    for group in &dev_groups {
        python_pip_args.push("--group");
        python_pip_args.push(group);
    }

    ensure_readme(target)?;
    execute_command(&python_pip_exec.to_string_lossy(), &python_pip_args, target)?;
    Ok(())
}

fn setup_pip_project(version: &str, target: &Path, scope: ProjectScope) -> Result<()> {
    let python_command = format!("python{version}");
    let python_venv_args = ["-m", "venv", ".venv"];
    execute_command(&python_command, &python_venv_args, target)?;

    match scope {
        ProjectScope::Install => pip_install_project(target),
        ProjectScope::Requirements => pip_install_requirements(target),
        ProjectScope::Base => Ok::<(), anyhow::Error>(()),
    }?;
    Ok(())
}

fn setup_git(target: &Path) -> Result<()> {
    process::Command::new("git")
        .arg("init")
        .current_dir(target)
        .status()?;
    Ok(())
}

fn parse_plato_toml(source: &Path) -> Result<(TemplateType, String)> {
    let toml_path = source.join("plato.toml");
    if !toml_path.exists() {
        bail!("Missing plato.toml in {}", source.display());
    }

    let content = fs::read_to_string(&toml_path).context(format!(
        "Could not read plato toml at {}",
        toml_path.display()
    ))?;

    let config: PlatoToml = toml::from_str(&content).context(format!(
        "Invalid format in plato toml at {}",
        toml_path.display()
    ))?;

    let ttype = match config.ttype.to_lowercase().as_str() {
        "python" | "py" => TemplateType::Python,
        "rust" | "rs" => TemplateType::Rust,
        _ => TemplateType::Base,
    };

    let version = config.version.unwrap_or_else(|| String::from("latest"));

    Ok((ttype, version))
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

fn build_target_map(
    raw_map: HashMap<PathBuf, FileContent>,
    path_context: &HashMap<&str, &str>,
    template_context: &impl Serialize,
) -> Result<HashMap<PathBuf, FileContent>> {
    let target_map = render_paths(raw_map, path_context);
    let rendered_map = render_templates(target_map, template_context)?;
    Ok(rendered_map)
}

fn scan_source_map(source: &Path) -> Result<HashMap<PathBuf, FileContent>> {
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

        let content = if path.is_dir() {
            FileContent::None
        } else {
            match path.extension().and_then(|s| s.to_str()) {
                Some("j2" | "mj") => FileContent::Template(fs::read_to_string(path)?),
                _ => FileContent::Binary(fs::read(path)?),
            }
        };

        raw_map.insert(rel_path, content);
    }

    Ok(raw_map)
}

fn flush_to_disk(target_map: &HashMap<PathBuf, FileContent>, target: &Path) -> Result<()> {
    for (path, content) in target_map {
        let full_path = target.join(path);
        match content {
            FileContent::Binary(bytes) => {
                if let Some(parent) = full_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(full_path, bytes)?;
            }
            FileContent::None => {
                fs::create_dir_all(full_path)?;
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

fn setup_base_workspace(
    project_name: &str,
    project_version: &String,
    source: &Path,
    target: &Path,
) -> Result<HashMap<PathBuf, FileContent>> {
    let source_map = scan_source_map(source)?;

    let mut path_context = HashMap::new();
    path_context.insert("project_name", project_name);
    path_context.insert("project_version", project_version);

    let template_context = TemplateContext {
        project_name,
        project_version,
        version: project_version,
    };

    let target_map = build_target_map(source_map, &path_context, &template_context)?;
    flush_to_disk(&target_map, target)?;
    Ok(target_map)
}

fn setup_python_workspace(project_name: &str, version: &str, target: &Path) -> Result<()> {
    let scope = get_python_project_scope(target, project_name);
    match get_python_manager(version) {
        PythonPackageManager::Uv => setup_uv_project(version, target, scope),
        PythonPackageManager::Pip => setup_pip_project(version, target, scope),
        PythonPackageManager::None => {
            eprintln!("No compatible python package manager found");
            Ok(())
        }
    }?;
    Ok(())
}

fn setup_rust_workspace(_project_name: &str, _version: &str, _target: &Path) -> Result<()> {
    Ok(())
}

/// Run the CLI.
///
/// # Errors
/// Returns an error if argument parsing, template loading, filesystem access,
/// template rendering, or project setup fails.
pub fn run() -> Result<()> {
    let cli = Cli::parse();
    let pwd = env::current_dir()?;

    match &cli.command {
        Commands::Init {
            template,
            project_name,
        } => {
            let target = pwd.join(project_name);
            let mut guard = ProjectGuard::new(target.clone());
            let source = get_config_dir().join(template);
            let (ttype, version) = parse_plato_toml(&source)?;
            setup_base_workspace(project_name, &version, &source, &target)?;
            match ttype {
                TemplateType::Python => setup_python_workspace(project_name, &version, &target),
                TemplateType::Rust => setup_rust_workspace(project_name, &version, &target),
                TemplateType::Base => Ok(()),
            }?;
            setup_git(&target)?;
            guard.release();
            Ok(())
        }
    }
}
