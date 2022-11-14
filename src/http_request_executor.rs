use std::collections::HashMap;

use anyhow::{anyhow, Result};
use reqwest::header::{HeaderName, HeaderValue};
use rhai::{Dynamic, ImmutableString};

use crate::http_request::HttpRequest;

#[derive(Clone)]
pub struct VariableStorage {
    storage: HashMap<ImmutableString, ImmutableString>,
}

impl VariableStorage {
    fn new() -> VariableStorage {
        return VariableStorage { storage: HashMap::new() };
    }
    fn set(&mut self, name: ImmutableString, value: ImmutableString) {
        self.storage.insert(name, value);
    }
    fn get(&self, name: &ImmutableString) -> &ImmutableString {
        return self.storage.get(name).unwrap();
    }
    fn is_empty(&self) -> bool {
        return self.storage.is_empty();
    }
    fn clear(&mut self, name: &ImmutableString) {
        self.storage.remove(name);
    }
    fn clear_all(&mut self) {
        self.storage.clear();
    }
}

pub struct ExecutionContext {
    pub client: reqwest::blocking::Client,
    scope: rhai::Scope<'static>,
    engine: rhai::Engine,
    pub var_storage: VariableStorage,
}

impl ExecutionContext {
    fn initialize_engine(context: &mut ExecutionContext) {
        context.engine.register_type_with_name::<VariableStorage>("VariableStorage")
            .register_fn("get", VariableStorage::get)
            .register_fn("set", VariableStorage::set)
            .register_fn("isEmpty", VariableStorage::is_empty)
            .register_fn("clear", VariableStorage::clear)
            .register_fn("clearAll", VariableStorage::clear_all);
    }

    pub fn new() -> Result<ExecutionContext> {
        let client = reqwest::blocking::Client::builder()
            .http1_title_case_headers()
            .build()
            .map_err(|e| anyhow!("{:?}", e))?;
        let engine = rhai::Engine::new();
        let var_storage = VariableStorage::new();
        let scope = rhai::Scope::new();
        let mut context = ExecutionContext { client, scope, engine, var_storage };
        ExecutionContext::initialize_engine(&mut context);
        // context.eval("let x = 4")?;
        // println!("{}", context.eval("x + 6")?);
        return Ok(context);
    }

    pub fn eval(&mut self, script: &str) -> Result<Dynamic> {
        return match self.engine.eval_with_scope::<Dynamic>(&mut self.scope, script) {
            Ok(value) => Ok(value),
            Err(err) => Err(anyhow!("{:?}", err)),
        }
    }
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
    context: &ExecutionContext,
) -> Result<()> {
    println!("{}", http_request.name());
    let client = &context.client;
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
