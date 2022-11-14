use crate::http_file_parser::parse_http_file;
use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;
use crate::http_request_executor::ExecutionContext;

pub fn execute_http_files(files: Vec<&PathBuf>) -> Result<()> {
    let context = ExecutionContext::new()?;
    for p in files {
        if !p.is_file() {
            // let filename = p.file_name().unwrap().to_str().unwrap();
            return Err(anyhow!(format!("file {:?} not found", p.to_path_buf())));
        }
        for request in parse_http_file(p.to_path_buf())
            .context(format!("while parsing file {}", p.display()))?
            .iter()
        {
            crate::http_request_executor::execute_http_request(request, &context)?;
        }
    }
    return Ok(());
}
