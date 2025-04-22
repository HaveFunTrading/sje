use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("parse error {0}")]
    Parse(String),
    #[error("missing field `{0}`")]
    MissingField(&'static str),
    #[error("other error {0}")]
    Other(String),
}

impl Error {
    pub fn other(msg: impl AsRef<str>) -> Self {
        Error::Other(msg.as_ref().to_string())
    }
}

impl From<Error> for std::io::Error {
    fn from(err: Error) -> Self {
        Self::other(err)
    }
}
