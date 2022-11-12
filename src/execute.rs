use std::error::Error;
use std::path::PathBuf;
use crate::errors::FileNotFoundError;
use crate::http_file_parser::parse_http_file;

pub fn execute_http_files(files: Vec<&PathBuf>) -> Result<(), Box<dyn Error>> {
    for p in files {
        if !p.is_file() {
            // let filename = p.file_name().unwrap().to_str().unwrap();
            return Err(Box::new(FileNotFoundError(p.to_path_buf())))
        }
        let requests = parse_http_file(p.to_path_buf());
        println!("{:#?}", requests)
    }
    return Ok(());
}
