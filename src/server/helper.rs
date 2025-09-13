use sha2::{Digest, Sha256};
use std::{collections::HashMap, str::FromStr};
use chrono::{DateTime, Utc};
use tera::{to_value, Function, Map, Number, Result, Tera, Value};

use crate::server::{CONFIG, SITE};

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
        helper.register("url_for", UrlHelper);
        helper.register("full_url_for", FullUrlHelper);
        helper.register("gravatar", GravatarHelper);
        helper.register("css", CssHelper);
        helper.register("js", JsHelper);
        helper.register("link", LinkHelper);
        helper.register("mail", MailHelper);
        helper.register("image", ImageHelper);
        helper.register("favicon", FaviconHelper);
        helper.register("feed", FeedHelper);
        helper.register("meta", MetaHelper);
        helper.register("partial", PartialHelper);
        helper.register("list_categories", CategoriesHelper);
        helper.register("list_tags", TagsHelper);
        helper.register("list_posts", PostsHelper);
        helper.register("list_pages", PagesHelper);
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

macro_rules! match_value_or_default {
    ($name:expr, $pat:pat => $val:ident, $default_value:expr) => {
        match $name {
            Some(arg) => {
                match arg {
                    $pat => $val,
                    _ => $default_value
                }
            },
            None => $default_value
        }
    };
    ($name:expr, $pat:pat => $val:ident, $default_value:expr, $and_then:expr) => {
        match $name {
            Some(arg) => {
                match arg {
                    $pat => $and_then($val),
                    _ => $default_value
                }
            },
            None => $default_value
        }
    };
    ($name:expr, $pat:pat => $val:ident) => {
        match $name {
            Some(arg) => {
                match arg {
                    $pat => $val,
                    _ => return Err(tera::Error::msg(format!(
                        "Param type error: expect pattern `{}` but got value `{:?}`",
                        stringify!($pat), arg
                    )))
                }
            },
            None => return Err(tera::Error::msg(format!(
                "Param not found: `{}`",
                stringify!($name)
            )))
        }
    }
}

#[derive(Clone)]
struct DateHelper;

impl Function for DateHelper {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value> {
        let ts = args
            .get("ts");
        let ts = match ts {
            Some(ts) => match ts {
                Value::Number(n) => n.as_i64().unwrap_or(Utc::now().timestamp()),
                Value::String(s) => DateTime::parse_from_rfc3339(s.as_str())
                    .unwrap_or(Utc::now().into())
                    .timestamp(),
                _ => return Err(tera::Error::msg("Missing 'path'"))
            },
            None => Utc::now().timestamp()
        };

        let fmt = match_value_or_default!(args.get("fmt"), Value::String(v) => v, &String::from("%Y-%m-%d %H:%M:%S"));
        let date = DateTime::from_timestamp(ts, 0).unwrap_or_else(|| Utc::now());
        Ok(to_value(date.format(fmt).to_string())?)
    }
}

#[derive(Clone)]
struct TagHelper {
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

fn make_tag_helper(tag: &str, path_attr: &str, defaults: &[(&str, &str)]) -> TagHelper {
    TagHelper {
        tag: tag.to_string(),
        path_attr: path_attr.to_string(),
        default_attrs: defaults
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect(),
    }
}

macro_rules! define_tag_helper {
    ($name:ident, $tag:expr, $path_attr:expr, { $($k:expr => $v:expr),* }) => {
        #[derive(Clone)]
        struct $name;
        impl Function for $name {
            fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
                let helper = make_tag_helper($tag, $path_attr, &[ $(($k, $v)),* ]);
                helper.call(args)
            }
        }
    };
}

define_tag_helper!(CssHelper, "link", "href", { "rel" => "stylesheet" });
define_tag_helper!(JsHelper, "script", "src", {});
define_tag_helper!(LinkHelper, "a", "href", {});
define_tag_helper!(ImageHelper, "img", "src", {});
define_tag_helper!(MailHelper, "a", "href", {});
define_tag_helper!(FaviconHelper, "link", "href", { "rel" => "icon" });
define_tag_helper!(FeedHelper, "link", "href", { "rel" => "alternate", "type" => "application/rss+xml" });
define_tag_helper!(MetaHelper, "meta", "content", { "name" => "generator" });

#[macro_export]
macro_rules! define_url_helper {
    // Only root
    ($name:ident, url) => {
        #[derive(Clone)]
        struct $name;
        impl Function for $name {
            fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
                let site_url = &CONFIG.load().site.url;
                let root = super::extract_root_path(site_url);
                let path = match_value_or_default!(args.get("path"), Value::String(v) => v);
                let relative = match_value_or_default!(args.get("relative"), Value::Bool(v) => v, &true);
                let res = if *relative {
                    if path.starts_with('/') {
                        format!(".{}", path)
                    } else {
                        format!("./{}", path)
                    }
                } else {
                    if path.starts_with('/') {
                        format!("{}{}", root.trim_end_matches('/'), path)
                    } else {
                        format!("{}/{}", root.trim_end_matches('/'), path)
                    }
                };
                Ok(Value::String(res))
            }
        }
    };

    // Full URL
    ($name:ident, fullurl) => {
        #[derive(Clone)]
        struct $name;
        impl Function for $name {
            fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
                let site_url = &CONFIG.load().site.url;
                let path = match_value_or_default!(args.get("path"), Value::String(v) => v);
                let res = if path.starts_with('/') {
                    format!("{}{}", site_url.trim_end_matches('/'), path)
                } else {
                    format!("{}/{}", site_url.trim_end_matches('/'), path)
                };
                Ok(Value::String(res))
            }
        }
    };

    // Gravatar
    ($name:ident, gravatar) => {
        #[derive(Clone)]
        struct $name;
        impl Function for $name {
            fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
                let mail = match_value_or_default!(args.get("mail"), Value::String(v) => v);
                let mut hashed_email = Sha256::new();
                hashed_email.update(mail.trim());
                let url = format!("https://www.gravatar.com/avatar/{:X}", hashed_email.finalize());
                Ok(Value::String(url))
            }
        }
    };
}

define_url_helper!(UrlHelper, url);
define_url_helper!(FullUrlHelper, fullurl);
define_url_helper!(GravatarHelper, gravatar);

#[derive(Clone)]
struct PartialHelper;

impl Function for PartialHelper {
    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
        // TODO: render other layout which in layout dir
        Ok(Value::Null)
    }
}

#[derive(Clone)]
struct ListHelper {
    pub list: String
}

impl Function for ListHelper {
    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let orderby = match_value_or_default!(args.get("orderby"), Value::String(v) => v, &String::from("name"));
        let order = match_value_or_default!(args.get("order"), Value::Number(v) => v, 1, |v: &Number| v.as_i64().unwrap_or(1));
        let show_count = match_value_or_default!(args.get("show_count"), Value::Bool(v) => v, &true);
        let list = match_value_or_default!(args.get("list"), Value::Bool(v) => v, &true);
        let separator = match_value_or_default!(args.get("separator"), Value::String(v) => v, &String::from(","));
        let amount = match_value_or_default!(args.get("amount"), Value::Number(v) => v, 1 << 16, |v: &Number| v.as_i64().unwrap_or(1 << 16));
        let tag_class = match_value_or_default!(args.get("tag_class"), Value::Object(v) => v, &Map::new());
        let tag_class_ul = match_value_or_default!(tag_class.get("ul"), Value::String(v) => v, &String::from("ul"));
        let tag_class_li = match_value_or_default!(tag_class.get("li"), Value::String(v) => v, &String::from("li"));
        let tag_class_a = match_value_or_default!(tag_class.get("a"), Value::String(v) => v, &String::from("a"));
        let tag_class_count = match_value_or_default!(tag_class.get("count"), Value::String(v) => v, &String::from("count"));

        let mut res = String::new();
        let site = &SITE.load();

        let render_list = |res: &mut String, iter: Vec<(String, String, usize)>| {
            res.push_str(&format!(r#"<ul class="{}" itemprop="keywords">"#, tag_class_ul));
            for (i, (name, href, count)) in iter.into_iter().enumerate() {
                if i >= amount as usize {
                    break;
                }
                res.push_str(&format!(r#"<li class="{}">"#, tag_class_li));
                res.push_str(&format!(r#"<a class="{}" href="{}">{}</a>"#, tag_class_a, href, name));
                if *show_count && count > 0 {
                    res.push_str(&format!(r#"<span class="{}">{}</span>"#, tag_class_count, count));
                }
                res.push_str("</li>");
            }
            res.push_str("</ul>");
        };

        let render_inline = |res: &mut String, iter: Vec<(String, String, usize)>| {
            for (i, (name, href, count)) in iter.into_iter().enumerate() {
                if i >= amount as usize {
                    break;
                }
                res.push_str(&format!(r#"<a class="{}" href="{}">{}"#, tag_class_a, href, name));
                if *show_count && count > 0 {
                    res.push_str(&format!(r#"<span class="{}">{}</span>"#, tag_class_count, count));
                }
                res.push_str(&format!("</a>{}", separator));
            }
        };

        match self.list.as_str() {
            "categorie" | "tag" => {
                let data = if self.list == "categorie" {
                    site.categories.iter()
                } else {
                    site.tags.iter()
                };
                let mut tmp: Vec<_> = data.map(|(k, v)| (k.clone(), v.path.clone(), v.posts.len())).collect();
                match orderby.as_str() {
                    "name" => {
                        if order == -1 {
                            tmp.sort_by(|x, y| y.0.cmp(&x.0));
                        } else {
                            tmp.sort_by(|x, y| x.0.cmp(&y.0));
                        }
                    }
                    "count" => {
                        if order == -1 {
                            tmp.sort_by(|x, y| y.2.cmp(&x.2));
                        } else {
                            tmp.sort_by(|x, y| x.2.cmp(&y.2));
                        }
                    }
                    _ => {}
                }

                if *list {
                    render_list(&mut res, tmp);
                } else {
                    render_inline(&mut res, tmp);
                }
            },
            "post" | "page" => {
                let mut tmp = if self.list == "post" {
                    site.posts.clone()
                } else {
                    site.pages.clone()
                };
                tmp.sort_by(|x, y| {
                    let x_date = DateTime::from_str(&x.date).unwrap_or(Utc::now());
                    let y_date = DateTime::from_str(&y.date).unwrap_or(Utc::now());
                    if order == -1 { y_date.cmp(&x_date) } else { x_date.cmp(&y_date) }
                });
                let mapped: Vec<_> = tmp.into_iter().map(|p| (p.title.clone(), p.title, 0)).collect();

                if *list {
                    render_list(&mut res, mapped);
                } else {
                    render_inline(&mut res, mapped);
                }
            },
            _ => {}
        }
        Ok(Value::String(res))
    }
}

fn make_list_helper(list: &str) -> ListHelper {
    ListHelper { list: list.to_string() }
}

macro_rules! define_list_helper {
    ($name:ident, $list:expr) => {
        #[derive(Clone)]
        struct $name;
        impl Function for $name {
            fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
                let helper = make_list_helper($list);
                helper.call(args)
            }
        }
    };
}

define_list_helper!(CategoriesHelper, "categorie");
define_list_helper!(TagsHelper, "tag");
define_list_helper!(PostsHelper, "post");
define_list_helper!(PagesHelper, "page");
