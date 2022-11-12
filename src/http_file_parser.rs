use std::error::Error;
use std::fs::File;
use std::path::PathBuf;
use std::io::{BufRead, BufReader};
use crate::errors::{FileIoError};

#[derive(Debug)]
pub struct HttpRequest {
    request_no: u32,
    name: String,
    url: String,
    headers: Vec<String>,
    body: Vec<String>,
    options: Vec<String>,
}

// Represents state as determined by the latest parsed line
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum ParseState {
    Unknown,
    NewRequest,
    Uri,
    Header,
    AfterHeaders,
    Body
}

// Determined type of line based of latest parse state
#[derive(Debug, Eq, PartialEq)]
enum LineType {
    NewRequest,
    ConfigOption,
    Comment,
    Empty,
    Unknown,
}

fn parse_url(line: String) -> String {
    return line.as_str().trim().to_string();
}

fn parse_name(line: String) -> String {
    return line.strip_prefix("###").unwrap().trim().to_string();
}

fn get_line_type(line: &String) -> LineType {
    return if line.starts_with("###") {
        LineType::NewRequest
    } else if line.starts_with("# @") {
        LineType::ConfigOption
    } else if line.starts_with("#") {
        LineType::Comment
    } else if line.is_empty() {
        LineType::Empty
    } else {
        LineType::Unknown
    };
}

pub fn parse_http_file(http_file_path: PathBuf) -> Result<Vec<HttpRequest>, Box<dyn Error>> {
    let file = match File::open(&http_file_path) {
        Err(reason) => return Err(Box::new(FileIoError(format!("couldn't open {}: {}", http_file_path.display(), reason)))),
        Ok(file) => file,
    };

    let mut http_requests = Vec::new();

    let mut parse_state: ParseState = ParseState::Unknown;
    let mut line_no: u32 = 1;
    let mut request_no: u32 = 1;

    let buf_reader = BufReader::new(file);

    let mut name: String;
    let mut url = "".to_string();
    let mut headers: Vec<String> = Vec::new();
    let mut body: Vec<String> = Vec::new();
    let mut options: Vec<String> = Vec::new();

    for result_line in buf_reader.lines() {
        let line = result_line.unwrap();
        let line_type = get_line_type(&line);

        if line_type == LineType::Unknown {
            if parse_state == ParseState::Unknown || parse_state == ParseState::NewRequest {
                url = parse_url(line);
                parse_state = ParseState::Uri;
            } else if parse_state == ParseState::Uri {
                headers.push(line.clone());
                parse_state = ParseState::Header;
            } else if parse_state == ParseState::AfterHeaders {
                // could also be handler
                body.push(line.clone());
            } else if parse_state == ParseState::Body {
                body.push(line.clone());
            } else {
                panic!("Unknown state {:?}/{:?} in line {}", parse_state, line_type, line_no)
            }
        } else if line_type == LineType::Empty {
            if parse_state == ParseState::Header {
                parse_state = ParseState::AfterHeaders;
            } else if parse_state == ParseState::Body {
                body.push(line.clone());
            }
        } else if line_type == LineType::NewRequest {
            // TODO FR repeated requests?
            name = parse_name(line);

            if url.is_empty() && parse_state == ParseState::Unknown {
                // the very first time it might be a initial marker
                parse_state = ParseState::NewRequest;
                continue;
            }
            parse_state = ParseState::NewRequest;

            http_requests.push(
                HttpRequest {
                    request_no,
                    name: name.clone(),
                    url: url.clone(),
                    headers: headers.clone(),
                    body: body.clone(),
                    options: options.clone()
                }
            );
            name.clear();
            url.clear();
            headers.clear();
            body.clear();
            options.clear();
            request_no += 1;
        } else if line_type == LineType::ConfigOption {
            options.push(line.clone())
        } else if line_type == LineType::Comment {
            // TODO FR or ignore
        }
        line_no += 1;
    }
    return Ok(http_requests);
}
