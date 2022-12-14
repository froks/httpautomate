use crate::http_file_parser::parse_http_file;
use crate::http_request_executor::ExecutionContext;
use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;

pub fn execute_http_files(
    files: Vec<&PathBuf>,
    env_files: Vec<&PathBuf>,
    environment: &String,
) -> Result<()> {
    let context = ExecutionContext::new(env_files, environment)?;
    for p in files {
        if !p.is_file() {
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
