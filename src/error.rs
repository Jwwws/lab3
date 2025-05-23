use std::fmt;
use std::io;

#[derive(Debug)]
pub enum ShellError {
    Io(io::Error),
    ParseError(String),
    CommandError(String),
}

impl fmt::Display for ShellError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ShellError::Io(err) => write!(f, "IO错误: {}", err),
            ShellError::ParseError(err) => write!(f, "解析错误: {}", err),
            ShellError::CommandError(err) => write!(f, "命令错误: {}", err),
        }
    }
}

impl std::error::Error for ShellError {}

impl From<io::Error> for ShellError {
    fn from(err: io::Error) -> Self {
        ShellError::Io(err)
    }
}
