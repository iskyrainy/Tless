use std::collections::HashMap;
use chrono::{DateTime, Utc};
use tera::{to_value, Function, Tera, Value, Result};

use crate::define_tag_helper;

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
        helper.register("css", CssHelper);
        helper.register("js", JsHelper);
        helper.register("link", LinkHelper);
        helper.register("mail", MailHelper);
        helper.register("image", ImageHelper);
        helper.register("favicon", FaviconHelper);
        helper.register("feed", FeedHelper);
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

#[derive(Clone)]
pub struct TagHelper {
    pub tag: String,
    pub default_attrs: HashMap<String, String>,
    pub path_attr: String,
}

impl Function for TagHelper {
    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
        fn build_tag(tag: &str, default_attrs: &HashMap<String, String>, path_attr: &str, val: &Value) -> Result<String> {
            let mut html = String::new();
            if let Some(s) = val.as_str() {
                let mut path = s.to_string();
                if !path.starts_with('/') && !path.starts_with("http") && !path.starts_with("mailto:") {
                    path = format!("/{}", path);
                }

                html.push_str(&format!("<{}", tag));
                for (k, v) in default_attrs {
                    html.push_str(&format!(r#" {}="{}""#, k, v));
                }
                html.push_str(&format!(r#" {}="{}">"#, path_attr, path));
                return Ok(html);
            }

            if let Some(map) = val.as_object() {
                html.push_str(&format!("<{}", tag));
                for (k, v) in default_attrs {
                    html.push_str(&format!(r#" {}="{}""#, k, v));
                }
                for (k, v) in map {
                    let v = v.as_str().ok_or_else(|| tera::Error::msg(format!("Invalid type for key {}", k)))?;
                    let mut val = v.to_string();
                    if k == path_attr && !v.starts_with('/') && !v.starts_with("http") && !v.starts_with("mailto:") {
                        val = format!("/{}", v);
                    }
                    html.push_str(&format!(r#" {}="{}""#, k, val.replace('"', "&quot;")));
                }
                html.push('>');
                return Ok(html);
            }

            Err(tera::Error::msg("Invalid path type"))
        }

        let path = args.get("path").ok_or_else(|| tera::Error::msg("Missing 'path'"))?;
        let html = match path {
            Value::Array(arr) => {
                let mut out = Vec::new();
                for v in arr {
                    out.push(build_tag(&self.tag, &self.default_attrs, &self.path_attr, v)?);
                }
                out.join("\n")
            }
            _ => build_tag(&self.tag, &self.default_attrs, &self.path_attr, path)?,
        };
        Ok(Value::String(html))
    }
}

pub fn make_tag_helper(tag: &str, path_attr: &str, defaults: &[(&str, &str)]) -> TagHelper {
    TagHelper {
        tag: tag.to_string(),
        path_attr: path_attr.to_string(),
        default_attrs: defaults
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    }
}

define_tag_helper!(CssHelper, "link", "href", { "rel" => "stylesheet" });
define_tag_helper!(JsHelper, "script", "src", {});
define_tag_helper!(LinkHelper, "a", "href", {});
define_tag_helper!(ImageHelper, "img", "src", {});
define_tag_helper!(MailHelper, "a", "href", {});
define_tag_helper!(FaviconHelper, "link", "href", { "rel" => "icon" });
define_tag_helper!(FeedHelper, "link", "href", { "rel" => "alternate", "type" => "application/rss+xml" });
