use std::error::Error;
use std::fmt;
use std::path::PathBuf;

pub struct FileNotFoundError(pub PathBuf);
pub struct FileIoError(pub String);

pub struct ParseError {
    pub error: String,
    pub file: PathBuf,
    pub line_no: u32,
    pub line: String,
}

impl fmt::Debug for FileNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "file {:?} not found", self.0)
    }
}

impl fmt::Display for FileNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "file {:?} not found", self.0)
    }
}

impl fmt::Debug for FileIoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error while reading file {:?}", self.0)
    }
}

impl fmt::Display for FileIoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "error while reading file {:?}", self.0)
    }
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} while parsing file {:?} in line #{}:\n{}", self.error, self.file, self.line_no, self.line)
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} while parsing file {:?} in line #{}:\n{}", self.error, self.file, self.line_no, self.line)
    }
}

impl Error for FileNotFoundError {}
impl Error for FileIoError {}
impl Error for ParseError {}
