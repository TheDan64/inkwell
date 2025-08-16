
/// TODOC (ErisianArchitect):
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum Error {
    #[error("basic types must have names")]
    EmptyNameError,
}