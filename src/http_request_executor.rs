use crate::http_request::HttpRequest;
use anyhow::{anyhow, Result};
use reqwest::header::{HeaderName, HeaderValue};

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
        } else if split_url.len() == 2 {
            match split_url[0].to_lowercase().as_str() {
                "get" => Ok(reqwest::Method::GET),
                "post" => Ok(reqwest::Method::POST),
                "put" => Ok(reqwest::Method::PUT),
                "delete" => Ok(reqwest::Method::DELETE),
                _ => Err(anyhow!("{} is a unknown http method", split_url[0])),
            }
        } else {
            Err(anyhow!(
                "Invalid URL, http method couldn't be determined {}",
                self.unresolved_url
            ))
        };
    }

    pub fn uri(&self) -> Result<reqwest::Url> {
        let split_url: Vec<&str> = self.unresolved_url.split_whitespace().collect();
        return if split_url.len() == 1 {
            Ok(split_url[0]
                .parse::<reqwest::Url>()
                .map_err(|e| anyhow!("{:?} @ '{}'", e, split_url[0]))?)
        } else if split_url.len() >= 2 {
            Ok(split_url[1]
                .parse::<reqwest::Url>()
                .map_err(|e| anyhow!("{:?} @ '{}'", e, split_url[1]))?)
        } else {
            Err(anyhow!("No URL given"))
        };
    }

    pub fn headers(&self) -> Result<reqwest::header::HeaderMap> {
        let mut map = reqwest::header::HeaderMap::new();
        self.unresolved_headers
            .iter()
            .map(|it| it.split_once(":").unwrap())
            .map(|(key, value)| {
                (
                    HeaderName::try_from(key).unwrap(),
                    HeaderValue::try_from(value).unwrap(),
                )
            })
            .for_each(|(key, value)| {
                map.append(key, value);
            });
        return Ok(map);
    }

    pub fn body(&self) -> Result<String> {
        // TODO FR resolve body if function
        return Ok(self.unresolved_body.join("\n"));
    }
}

pub fn execute_http_request(
    http_request: &HttpRequest,
    context: &ExecutionContext<'_>,
) -> Result<()> {
    println!("{}", http_request.name());
    let client = context.client;
    let req = client
        .request(http_request.method()?, http_request.uri()?)
        .headers(http_request.headers()?)
        .body(http_request.body()?);

    // println!("{}", req.body());
    let res = req.send().unwrap();

    let headers: String = res
        .headers()
        .iter()
        .map(|e| format!("{}: {}", e.0.to_string(), e.1.to_str().unwrap()))
        .collect::<Vec<String>>()
        .join("\n");
    println!("{}\n", http_request.uri()?);
    println!("{}\n", headers);
    println!("{}", res.text().unwrap());

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::http_request::HttpRequest;
    use test_case::test_case;

    #[test_case("", "GET" ; "implicit GET")]
    #[test_case("GET", "GET" ; "GET")]
    #[test_case("POST", "POST" ; "POST")]
    #[test_case("PUT", "PUT" ; "PUT")]
    #[test_case("DELETE", "DELETE" ; "DELETE")]
    fn test_parse_extract_method_type(qualifier: &str, method_type: &str) {
        let request = HttpRequest {
            unresolved_url: format!(
                r#"
            {} https://www.rust-lang.org/"#,
                qualifier
            ),
            ..HttpRequest::default()
        };

        assert_eq!(request.method().unwrap().as_str(), method_type)
    }

    #[test_case("10" ; "throw on number")]
    #[test_case("test" ; "throw on unknown")]
    #[should_panic(expected = "is a unknown http method")]
    fn test_parse_throw_on_invalid_method(qualifier: &str) {
        let request = HttpRequest {
            unresolved_url: format!(
                r#"
            {} https://www.rust-lang.org/"#,
                qualifier
            ),
            ..HttpRequest::default()
        };
        request.method().unwrap();
    }

    #[test_case("https://www rust-lang org/" ; "whitespace instead dots")]
    #[should_panic(expected = "Invalid URL, http method couldn't be determined")]
    fn test_parse_throw_on_whitespaces_url(url: &str) {
        let request = HttpRequest {
            unresolved_url: format!("GET {}", url),
            ..HttpRequest::default()
        };
        request.method().unwrap();
    }
}
