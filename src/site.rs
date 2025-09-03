use std::{env, fs, process};

/// Initialize the site structure in the current directory.
pub fn init() {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    // init site directory
    let current_dir = current_dir.join("blog");
    if current_dir.exists() {
        eprintln!("Directory 'blog' already exists in current path.");
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
            .unwrap_or_else(|_| panic!("Failed to create directory: {}", dir));
    }

    // init base theme layout
    let theme_dir = current_dir.join("theme").join("base");
    fs::create_dir(&theme_dir)
        .expect("Failed to create directory: base");
    fs::write(theme_dir.join("index.html"), base_theme_text())
        .expect("Failed to create index.html");
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

fn base_theme_text() -> String {
    String::from(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>My Blog</title>
</head>
<body>
    <header><h1>Welcome to My Blog</h1></header>
    <div class="content">{{ content }}</div>
    <footer>© 2025 My Blog</footer>
</body>
</html>
"#)
}
