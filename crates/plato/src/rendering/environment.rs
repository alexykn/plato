use minijinja::{Environment, UndefinedBehavior};

use super::filters;

pub(crate) fn new_template_environment() -> Environment<'static> {
    let mut env = Environment::new();
    env.set_undefined_behavior(UndefinedBehavior::Strict);
    filters::register_all(&mut env);
    env
}
