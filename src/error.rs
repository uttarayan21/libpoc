use std::backtrace::Backtrace;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct Error {
    pub kind: ErrorKind,
    pub backtrace: Backtrace,
}

impl<E: Into<ErrorKind>> From<E> for Error {
    fn from(e: E) -> Self {
        Self {
            kind: e.into(),
            backtrace: Backtrace::capture(),
        }
    }
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.kind)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ErrorKind {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("{0}")]
    LibrawError(#[from] libraw_r::error::LibrawError),
    #[error("{0}")]
    Other(Box<dyn std::error::Error>),
}

impl Error {
    pub fn capture(e: impl Into<Box<dyn std::error::Error>>) -> Self {
        Self {
            kind: ErrorKind::Other(e.into()),
            backtrace: Backtrace::capture(),
        }
    }
}
