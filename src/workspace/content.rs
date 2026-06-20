use std::path::PathBuf;
use std::rc::Rc;
use std::sync::OnceLock;

pub(crate) enum FileContent {
    BinaryLazy {
        path: PathBuf,
        cache: OnceLock<Rc<[u8]>>,
    },
    Binary(Rc<[u8]>),
    Template(Rc<str>),
    None,
}
