use std::{collections::HashMap};
use chrono::{DateTime, Utc};
use tera::{to_value, Function, Tera, Value};

pub trait CloneableFunction: Function + Send + Sync {
    fn clone_box(&self) -> Box<dyn CloneableFunction>;
}

impl<T> CloneableFunction for T
where
    T: 'static + Function + Send + Sync + Clone,
{
    fn clone_box(&self) -> Box<dyn CloneableFunction> {
        Box::new(self.clone())
    }
}

type HelperFunc = Box<dyn CloneableFunction>;

impl Clone for HelperFunc {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

#[derive(Default)]
pub struct Helpers {
    funcs: HashMap<String, HelperFunc>,
}

impl Helpers {
    pub fn new() -> Self {
        let mut helper = Self { funcs: HashMap::new() };
        helper.register("date", DateHelper);
        helper
    }

    pub fn register<F>(&mut self, name: &str, f: F)
    where
        F: Function + Send + Sync + Clone + 'static,
    {
        self.funcs.insert(name.to_string(), Box::new(f));
    }

    pub fn apply_to(&self, tera: &mut Tera) {
        for (name, func) in &self.funcs {
            let f_clone = func.clone();
            tera.register_function(name, move |args: &HashMap<String, Value>| {
                f_clone.call(args)
            });
        }
    }
}

#[derive(Clone)]
struct DateHelper;

impl Function for DateHelper {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let ts = args
            .get("ts")
            .and_then(|v| v.as_i64())
            .unwrap_or_else(|| Utc::now().timestamp());

        let fmt = args
            .get("fmt")
            .and_then(|v| v.as_str())
            .unwrap_or("%Y-%m-%d");

        let date = DateTime::from_timestamp(ts, 0).unwrap_or_else(|| Utc::now());
        Ok(to_value(date.format(fmt).to_string())?)
    }
}
