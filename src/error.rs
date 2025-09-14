#[derive(Debug)]
pub enum Error {
    ParseError(String),
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::ParseError(s) => write!(f, "{s}"),
        }
    }
}
