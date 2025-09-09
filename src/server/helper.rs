use std::{collections::{BTreeMap, HashMap}};

use tera::{Function, Value, from_value, to_value};


fn make_url_for(urls: BTreeMap<String, String>) -> impl Function {
    Box::new(move |args: &HashMap<String, Value>| -> tera::Result<Value> {
        match args.get("name") {
            Some(val) => match from_value::<String>(val.clone()) {
                Ok(v) =>  Ok(to_value(urls.get(&v).unwrap()).unwrap()),
                Err(_) => Err("oops".into()),
            },
            None => Err("oops".into()),
        }
    })
}