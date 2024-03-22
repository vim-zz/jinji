use std::collections::HashMap;
use std::str;
use tera::{Function, Value};

#[derive(Default, Debug)]
pub struct HttpGet<'a> {
    url: Option<&'a str>,
}

impl<'a> From<&'a HashMap<String, Value>> for HttpGet<'a> {
    fn from(args: &'a HashMap<String, Value>) -> Self {
        let url = args.get("url").map(|value| value.as_str()).flatten();
        HttpGet { url }
    }
}

impl Function for HttpGet<'_> {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let get_result: HttpGet = args.into();

        let url = get_result.url.ok_or(tera::Error::msg("url is missing"))?;

        let response = reqwest::blocking::get(url)
            .or_else(|e| Err(tera::Error::chain("HTTP Get failed", e)))?;
        let headers_copy = response.headers().clone();
        let headers = headers_copy.iter().map(|(key, value)| {
            (
                key.to_string(),
                Value::String(value.to_str().unwrap().to_string()),
            )
        });
        let body = response
            .text()
            .or_else(|e| Err(tera::Error::chain("HTTP Get retuened not text", e)))?;

        let mut map = serde_json::Map::new();
        map.insert("url".into(), url.into());
        map.insert("text".into(), body.into());
        map.insert(
            "headers".into(),
            Value::Object(tera::Map::from_iter(headers.into_iter())),
        );

        Ok(Value::Object(map))
    }
}
