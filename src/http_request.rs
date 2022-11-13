#[derive(Debug)]
pub struct HttpRequest {
    pub request_no: u32,
    pub name: String,
    pub unresolved_url: String,
    pub unresolved_headers: Vec<String>,
    pub unresolved_body: Vec<String>,
    pub options: Vec<String>,
}

