use std::{env, error::Error, fs};

/// Initialize the site structure in the current directory.
pub fn init() -> Result<(), Box<dyn Error>> {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    // init site directory
    let current_dir = current_dir.join("blog");
    if current_dir.exists() {
        return Err("Site directory already exists.".into());
    }
    fs::create_dir(&current_dir)?;

    // init config file
    let conf_path = current_dir.join("tless.toml");
    fs::write(&conf_path, base_config_text())?;

    // init directory structure
    let dirs = vec!["source", "theme", "public", "plugin", "statistic"];
    for dir in dirs {
        fs::create_dir(current_dir.join(dir))?;
    }
    let blog_dirs = vec!["draft", "post", "page"];
    for dir in blog_dirs {
        fs::create_dir(current_dir.join("source").join(dir))?;
    }

    // init base theme layout
    let theme_dir = current_dir.join("theme").join("base");
    fs::create_dir(&theme_dir)?;
    let layout_dir = theme_dir.join("layout");
    fs::create_dir(&layout_dir)?;
    let resource_dir = theme_dir.join("resource");
    fs::create_dir(&resource_dir)?;
    fs::write(layout_dir.join("index.html"), base_theme_text())?;
    Ok(())
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
zone = ""
theme = "base"
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
