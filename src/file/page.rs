use std::{error::Error, fs};

use chrono::Utc;

use crate::file::{get_path, is_file_exist};


pub fn add_page(name: &String) -> Result<(), Box<dyn Error>> {
    let class = String::from("page");
    let file_path = get_path(name, &class);
    if is_file_exist(&file_path) {
        return Err("Blog already exists.".into());
    }
    fs::write(file_path, base_page_text(name))?;
    Ok(())
}

fn base_page_text(name: &String) -> String {
    format!("---\ntitle: {}\ndate: {}\nlayout: index.html\n---\n", 
        name, 
        Utc::now().format("%Y-%m-%d %H:%M:%S")
    )
}

pub fn remove_page(name: &String) -> Result<(), Box<dyn Error>> {
    let class = String::from("page");
    let file_path = get_path(name, &class);
    if !is_file_exist(&file_path) {
        return Err("Page does not exist.".into());
    }
    fs::remove_file(file_path)?;
    Ok(())
}
