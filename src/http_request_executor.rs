use anyhow::{anyhow, Result};
use reqwest::header::{HeaderName, HeaderValue};
use crate::http_request::HttpRequest;

pub struct ExecutionContext<'a> {
    pub client: &'a reqwest::blocking::Client,
}

impl HttpRequest {
    pub fn name(&self) -> String {
        return format!("#{} {}", self.request_no, self.name);
    }
    pub fn method(&self) -> Result<reqwest::Method> {
        let split_url: Vec<&str> = self.unresolved_url.split_whitespace().collect();
        return if split_url.len() == 1 {
            Ok(reqwest::Method::GET)
        } else if split_url.len() >= 2 {
            match split_url[0].to_lowercase().as_str() {
                "get" => Ok(reqwest::Method::GET),
                "put" => Ok(reqwest::Method::PUT),
                _ => Err(anyhow!("{} is a unknown http method", split_url[0]))
            }
        } else {
            Err(anyhow!("Invalid URL, http method couldn't be determined {}", self.unresolved_url))
        };
    }
    pub fn uri(&self) -> Result<reqwest::Url> {
        let split_url: Vec<&str> = self.unresolved_url.split_whitespace().collect();
        return if split_url.len() == 1 {
            Ok(split_url[0].parse::<reqwest::Url>().map_err(|e| anyhow!("{:?} @ '{}'", e, split_url[0]))?)
        } else if split_url.len() >= 2 {
            Ok(split_url[1].parse::<reqwest::Url>().map_err(|e| anyhow!("{:?} @ '{}'", e, split_url[1]))?)
        } else {
            Err(anyhow!("No URL given"))
        };
    }

    pub fn headers(&self) -> Result<reqwest::header::HeaderMap> {
        let mut map = reqwest::header::HeaderMap::new();
        self.unresolved_headers.iter()
            .map(|it| { it.split_once(":").unwrap() })
            .map(|(key, value)| { (HeaderName::try_from(key).unwrap(), HeaderValue::try_from(value).unwrap()) })
            .for_each(|(key, value)| { map.append(key, value); });
        return Ok(map);
    }

    pub fn body(&self) -> Result<String> {
        // TODO FR resolve body if function
        return Ok(self.unresolved_body.join("\n"));
    }
}

pub fn execute_http_request(http_request: &HttpRequest, context: &ExecutionContext<'_>) -> Result<()> {
    println!("{}", http_request.name());
    let client = context.client;
    let req = client
        .request(http_request.method()?, http_request.uri()?)
        .headers(http_request.headers()?)
        .body(http_request.body()?);

    // println!("{}", req.body());
    let res = req.send().unwrap();

    let headers: String = res.headers().iter().map(|e| { format!("{}: {}", e.0.to_string(), e.1.to_str().unwrap()) }).collect::<Vec<String>>().join("\n");
    println!("{}\n", http_request.uri()?);
    println!("{}\n", headers);
    println!("{}", res.text().unwrap());

    Ok(())
}
