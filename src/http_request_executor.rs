use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};
use regex::{Regex};
use reqwest::header::{HeaderName, HeaderValue};
use rhai::{Dynamic};

use crate::http_request::HttpRequest;

#[derive(Clone)]
pub struct VariableStorage {
    storage: HashMap<String, String>,
}

impl VariableStorage {
    fn new() -> VariableStorage {
        return VariableStorage {
            storage: HashMap::new(),
        };
    }
    fn set(&mut self, name: String, value: String) {
        self.storage.insert(name, value);
    }
    fn get(&self, name: &String) -> String {
        return self.storage.get(name).unwrap_or(name).to_string();
    }
    fn contains(&self, name: &str) -> bool {
        return self.storage.contains_key(name);
    }
    fn is_empty(&self) -> bool {
        return self.storage.is_empty();
    }
    fn clear(&mut self, name: &String) {
        self.storage.remove(name);
    }
    fn clear_all(&mut self) {
        self.storage.clear();
    }
}

pub struct ExecutionContext<'a> {
    pub client: reqwest::blocking::Client,
    engine: rhai::Engine,
    scope: rhai::Scope<'a>,
    pub storage: Arc<Mutex<VariableStorage>>,
}

type VariableStorageAPI = Arc<Mutex<VariableStorage>>;

impl ExecutionContext<'_> {
    pub fn new() -> Result<ExecutionContext<'static>> {
        let client = reqwest::blocking::Client::builder()
            .http1_title_case_headers()
            .build()
            .map_err(|e| anyhow!("{:?}", e))?;
        // rhai::config::hashing::set_ahash_seed(Some([123, 456, 789, 42])).unwrap();
        let mut engine = rhai::Engine::new();
        let mut scope: rhai::Scope<'static> = rhai::Scope::new();
        let storage = Arc::new(Mutex::new(VariableStorage::new()));

        let api = storage.clone();
        scope.push_constant("Storage", api);
        engine.register_type_with_name::<VariableStorageAPI>("StorageType");
        engine.register_fn(
            "set",
            |api: &mut VariableStorageAPI, key: String, value: String| {
                let mut storage = api.lock().unwrap();
                storage.set(key, value);
            },
        );
        engine.register_fn("get", |api: &mut VariableStorageAPI, key: String| {
            return api.lock().unwrap().get(&key);
        });
        engine.register_fn("clear", |api: &mut VariableStorageAPI, key: String| {
            return api.lock().unwrap().clear(&key);
        });
        engine.register_fn("clear_all", |api: &mut VariableStorageAPI| {
            return api.lock().unwrap().clear_all();
        });
        engine.register_fn("is_empty", |api: &mut VariableStorageAPI| {
            return api.lock().unwrap().is_empty();
        });

        let mut context = ExecutionContext {
            client,
            engine,
            scope,
            storage
        };
        let init_script_bytes = include_bytes!("rhai/init_scope.rhai");
        let init_script = std::str::from_utf8(&init_script_bytes.as_slice()).unwrap();
        context.eval(init_script)?;
        return Ok(context);
    }

    pub fn eval(&mut self, script: &str) -> Result<Dynamic> {
        return match self
            .engine
            .eval_with_scope::<Dynamic>(&mut self.scope, script)
        {
            Ok(value) => Ok(value),
            Err(err) => Err(anyhow!("{:?}", err)),
        };
    }
}

pub trait ReplaceTemplates {
    fn replace_templates(&self, templates: &VariableStorage) -> String;
}

impl ReplaceTemplates for str {
    fn replace_templates(&self, templates: &VariableStorage) -> String {
        let r = Regex::new(r"\{\{(\w+)+}}").unwrap();

        let mut result = self.clone().to_string();
        r.captures_iter(self).for_each(|cap| {
            if templates.contains(&cap[1]) {
                result = result.replace(&cap[0], templates.get(&cap[1].to_string()).as_str());
            }
        });
        return result;
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

    pub fn uri(&self, execution_context: &ExecutionContext) -> Result<reqwest::Url> {
        let split_url: Vec<&str> = self.unresolved_url.split_whitespace().collect();
        return if split_url.len() == 1 {
            Ok(split_url[0]
                .replace_templates(&execution_context.storage.lock().unwrap())
                .parse::<reqwest::Url>()
                .map_err(|e| anyhow!("{:?} @ '{}'", e, split_url[0]))?)
        } else if split_url.len() >= 2 {
            Ok(split_url[1]
                .replace_templates(&execution_context.storage.lock().unwrap())
                .parse::<reqwest::Url>()
                .map_err(|e| anyhow!("{:?} @ '{}'", e, split_url[1]))?)
        } else {
            Err(anyhow!("No URL given"))
        };
    }

    pub fn headers(&self, execution_context: &ExecutionContext) -> Result<reqwest::header::HeaderMap> {
        let mut map = reqwest::header::HeaderMap::new();
        self.unresolved_headers
            .iter()
            .map(|it| it.split_once(":").unwrap())
            .map(|(key, value)| {
                let header_name = key.replace_templates(&execution_context.storage.lock().unwrap()).trim().to_string();
                let header_value = value.replace_templates(&execution_context.storage.lock().unwrap()).trim().to_string();
                (
                    HeaderName::try_from(header_name).unwrap(),
                    HeaderValue::try_from(header_value).unwrap(),
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

pub fn execute_http_request(http_request: &HttpRequest, context: &ExecutionContext) -> Result<()> {
    println!("{}", http_request.name());
    let client = &context.client;
    let req = client
        .request(http_request.method()?, http_request.uri(context)?)
        .headers(http_request.headers(context)?)
        .body(http_request.body()?);

    // println!("{}", req.body());
    let res = req.send().unwrap();

    let headers: String = res
        .headers()
        .iter()
        .map(|e| format!("{}: {}", e.0.to_string(), e.1.to_str().unwrap()))
        .collect::<Vec<String>>()
        .join("\n");
    println!("{}\n", http_request.uri(context)?);
    println!("{}\n", headers);
    println!("{}", res.text().unwrap());

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::http_request::HttpRequest;
    use test_case::test_case;

    #[test_case("", "GET"; "implicit GET")]
    #[test_case("GET", "GET"; "GET")]
    #[test_case("POST", "POST"; "POST")]
    #[test_case("PUT", "PUT"; "PUT")]
    #[test_case("DELETE", "DELETE"; "DELETE")]
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

    #[test_case("10"; "throw on number")]
    #[test_case("test"; "throw on unknown")]
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

    #[test_case("https://www rust-lang org/"; "whitespace instead dots")]
    #[should_panic(expected = "Invalid URL, http method couldn't be determined")]
    fn test_parse_throw_on_whitespaces_url(url: &str) {
        let request = HttpRequest {
            unresolved_url: format!("GET {}", url),
            ..HttpRequest::default()
        };
        request.method().unwrap();
    }
}
