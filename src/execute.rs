use std::path::PathBuf;
use crate::errors::AutomateError;

pub fn execute_http_files(files: Vec<&PathBuf>) -> Result<(), AutomateError> {
    for p in &files {
        if !p.is_file() {
            let filename = p.file_name().unwrap().to_str().unwrap();
            return Err(AutomateError::new(format!("'{}' is not a file", &filename).as_str()))
        }
    }
    return Ok(());
}
