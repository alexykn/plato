use minijinja::Environment;

use super::filters;

pub(crate) fn new_template_environment() -> Environment<'static> {
    let mut env = Environment::new();
    filters::register_all(&mut env);
    env
}
