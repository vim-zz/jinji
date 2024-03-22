use std::collections::HashMap;
use tera::{Filter, Value};

#[derive(Default, Debug)]
pub struct Banana {}

impl Filter for Banana {
    fn filter(&self, value: &Value, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let count = args.get("count").map(|x| x.as_i64().unwrap()).unwrap_or(1);
        match value {
            Value::String(x) => {
                let mut bananas = x.clone();
                for _x in 0..count {
                    bananas = format!("🍌{bananas}🍌");
                }
                Ok(Value::String(bananas))
            }
            _ => Ok(value.clone()),
        }
    }
}
