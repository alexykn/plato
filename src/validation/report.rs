use anyhow::{Result, bail};

#[derive(Debug, Default)]
pub(crate) struct ValidationReport {
    issues: Vec<ValidationIssue>,
}

#[derive(Debug)]
pub(crate) struct ValidationIssue {
    pub(crate) code: &'static str,
    pub(crate) message: String,
    pub(crate) hint: Option<String>,
}

impl ValidationIssue {
    pub(crate) fn new(
        code: &'static str,
        message: impl Into<String>,
        hint: impl Into<Option<String>>,
    ) -> Self {
        Self {
            code,
            message: message.into(),
            hint: hint.into(),
        }
    }
}

impl ValidationReport {
    pub(crate) fn push(&mut self, issue: ValidationIssue) {
        self.issues.push(issue);
    }

    pub(crate) fn has_errors(&self) -> bool {
        !self.issues.is_empty()
    }

    pub(crate) fn print(&self) {
        if self.issues.is_empty() {
            println!("Validation passed.");
            return;
        }

        println!("Validation failed:");
        for issue in &self.issues {
            println!("\n[{}]\n{}", issue.code, issue.message);
            if let Some(hint) = &issue.hint {
                println!("\nHint: {hint}");
            }
        }
    }

    pub(crate) fn into_result(self) -> Result<()> {
        if !self.has_errors() {
            return Ok(());
        }

        let mut message = String::from("Validation failed:");
        for issue in self.issues {
            message.push_str("\n\n[");
            message.push_str(issue.code);
            message.push_str("]\n");
            message.push_str(&issue.message);
            if let Some(hint) = issue.hint {
                message.push_str("\n\nHint: ");
                message.push_str(&hint);
            }
        }

        bail!(message)
    }
}
