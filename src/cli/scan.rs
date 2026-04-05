#[derive(Debug)]
pub enum FileContent {
    Template(String),
    Binary(Vec<u8>),
    Directory,
}
