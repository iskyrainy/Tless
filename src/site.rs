use std::{env, fs, process};

/// Initialize the site structure in the current directory.
/// # usage:
/// ```bash
/// tless site -i
/// ```
pub fn init() {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    // init site directory
    let current_dir = current_dir.join("site");
    if current_dir.exists() {
        eprintln!("Directory 'site' already exists in current path.");
        process::exit(1);
    }
    fs::create_dir(&current_dir).expect("Failed to create site directory");

    // init config file
    let conf_path = current_dir.join("tless.toml");
    fs::write(&conf_path, base_config_text())
        .expect("Failed to create configuration file");

    // init directory structure
    let dirs = vec!["source", "theme", "plugin", "statistic"];
    for dir in dirs {
        fs::create_dir(current_dir.join(dir))
            .unwrap_or_else(|_| panic!("Failed to create directory: {}", dir));
    }
    let blog_dirs = vec!["draft", "post", "page"];
    for dir in blog_dirs {
        fs::create_dir(current_dir.join("source").join(dir))
            .unwrap_or_else(|_| panic!("Failed to create blog directory: {}", dir));
    }
}

/// Generate a base configuration file content.
fn base_config_text() -> String {
    String::from(r#"
# Tless Configuration File
# This is a sample configuration file for Tless.
# You can customize the settings as per your requirements.
[site]
title = ""
description = ""
author = ""
url = ""
favicon = ""
menu = [
    { name = "Home", link = "/" }
]
# End of configuration file
"#)
}
