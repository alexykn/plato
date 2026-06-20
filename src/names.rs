use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ProjectNameSet {
    raw: String,
    kebab: String,
    snake: String,
    pascal: String,
}

impl ProjectNameSet {
    pub(crate) fn derive(project_name: &str) -> Self {
        let tokens = tokenize_name(project_name);

        Self {
            raw: project_name.to_string(),
            kebab: tokens.join("-"),
            snake: tokens.join("_"),
            pascal: to_pascal_case(&tokens),
        }
    }

    pub(crate) fn insert_context(&self, context: &mut HashMap<String, String>) {
        context.insert("project_name".to_string(), self.raw.clone());
        context.insert("project_kebab".to_string(), self.kebab.clone());
        context.insert("project_snake".to_string(), self.snake.clone());
        context.insert("project_pascal".to_string(), self.pascal.clone());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PythonNameSet {
    distribution: String,
    package: String,
    cli: String,
}

impl PythonNameSet {
    pub(crate) fn from_project(project: &ProjectNameSet) -> Self {
        Self {
            distribution: project.kebab.clone(),
            package: project.snake.clone(),
            cli: project.kebab.clone(),
        }
    }

    pub(crate) fn insert_context(&self, context: &mut HashMap<String, String>) {
        context.insert(
            "python_distribution_name".to_string(),
            self.distribution.clone(),
        );
        context.insert("python_package_name".to_string(), self.package.clone());
        context.insert("python_cli_name".to_string(), self.cli.clone());
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RustNameSet {
    package: String,
    crate_identifier: String,
    binary: String,
}

impl RustNameSet {
    pub(crate) fn from_project(project: &ProjectNameSet) -> Self {
        Self {
            package: project.kebab.clone(),
            crate_identifier: project.snake.clone(),
            binary: project.kebab.clone(),
        }
    }

    pub(crate) fn insert_context(&self, context: &mut HashMap<String, String>) {
        context.insert("rust_package_name".to_string(), self.package.clone());
        context.insert("rust_crate_name".to_string(), self.crate_identifier.clone());
        context.insert("rust_binary_name".to_string(), self.binary.clone());
    }
}

fn tokenize_name(name: &str) -> Vec<String> {
    let chars = name.chars().collect::<Vec<_>>();
    let mut tokens = Vec::new();
    let mut token = String::new();

    for (index, current) in chars.iter().copied().enumerate() {
        if !current.is_ascii_alphanumeric() {
            push_token(&mut tokens, &mut token);
            continue;
        }

        if starts_new_token(&chars, index, &token) {
            push_token(&mut tokens, &mut token);
        }

        token.push(current.to_ascii_lowercase());
    }

    push_token(&mut tokens, &mut token);
    tokens
}

fn starts_new_token(chars: &[char], index: usize, token: &str) -> bool {
    if token.is_empty() {
        return false;
    }

    let current = chars[index];
    if !current.is_ascii_uppercase() {
        return false;
    }

    let previous = chars[index - 1];
    if previous.is_ascii_lowercase() || previous.is_ascii_digit() {
        return true;
    }

    previous.is_ascii_uppercase() && chars.get(index + 1).is_some_and(char::is_ascii_lowercase)
}

fn push_token(tokens: &mut Vec<String>, token: &mut String) {
    if token.is_empty() {
        return;
    }

    tokens.push(std::mem::take(token));
}

fn to_pascal_case(tokens: &[String]) -> String {
    tokens
        .iter()
        .map(|token| {
            let mut chars = token.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };

            let mut pascal_token = String::new();
            pascal_token.push(first.to_ascii_uppercase());
            pascal_token.extend(chars);
            pascal_token
        })
        .collect()
}
