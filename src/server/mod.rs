use std::{collections::HashMap, env, error::Error, fs, path::PathBuf, sync::{mpsc, Arc, LazyLock}};

use arc_swap::ArcSwap;
use notify::Watcher;
use serde::{Deserialize, Serialize};

use crate::{file::{parse_file, Metadata}, result_matcher};

pub mod run;

/// Configuration structure for the application.
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Config {
    pub site: SiteConfig
}

/// Part of `[site]` configuration details.
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct SiteConfig {
    pub title: String,
    pub description: String,
    pub author: String,
    pub url: String,
    pub zone: String,
    pub theme: String,
    pub favicon: String,
    pub menu: Vec<Menu>
}

/// Menu item structure for site navigation.
/// # Fields
/// * `name` - The display name of the menu item.
/// * `link` - The URL or path the menu item points to.
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Menu {
    pub name: String,
    pub link: String
}

/// Get the path to the configuration file (`tless.toml`) in the current directory.
pub(crate) fn get_config_path() -> PathBuf {
    let current_dir = env::current_dir().unwrap();
    current_dir.join("tless.toml")
}

/// Load `tless.toml` to `CONFIG`.
pub(crate) fn get_config_toml() -> Config {
    let config_path = get_config_path();
    if !config_path.exists() {
        panic!("Configuration file not found at {}", config_path.display());
    }
    let config_content = fs::read_to_string(config_path)
        .expect("Failed to read configuration file");
    toml::from_str(&config_content)
        .expect("Failed to parse configuration file")
}

/// Global static configuration accessible throughout the application.
pub(crate) static CONFIG: LazyLock<ArcSwap<Config>> = LazyLock::new(|| {
    let config = get_config_toml();
    ArcSwap::from_pointee(config)
});

/// Watch the configuration file for changes and update the global `CONFIG` accordingly.
/// # Arguments
/// * `shutdown_rx` - A receiver to listen for shutdown signals.
pub(crate) async fn watch_config(mut shutdown_rx: tokio::sync::broadcast::Receiver<()>) -> Result<(), Box<dyn Error>> {
    let config_path = get_config_path();
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(&config_path, notify::RecursiveMode::NonRecursive)?;
    loop {
        if shutdown_rx.try_recv().is_ok() {
            break;
        }
        match rx.try_recv() {
            Ok(res) => match res {
                Ok(event) => {
                    match event.kind {
                        notify::EventKind::Modify(_) => {
                            CONFIG.store(Arc::new(get_config_toml()));
                            dbg!(CONFIG.load());
                            println!("Configuration reloaded.");
                        },
                        notify::EventKind::Remove(_) => {
                            result_matcher!(watcher.unwatch(&config_path), "Failed to unwatch old config file");
                            result_matcher!(watcher.watch(&config_path, notify::RecursiveMode::NonRecursive), "Failed to re-watch config file");
                        },
                        _ => {}
                    }
                },
                Err(e) => println!("config file watch error: {:?}", e),
            },
            Err(mpsc::TryRecvError::Empty) => {
                // Empty, sleep for a second
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            }
            Err(e) => {
                println!("receive error: {:?}", e);
                return Err("Failed to receive config file change event.".into());
            }
        }
    }
    Ok(())
}

/// Start watching the `tless.toml`.
/// # Arguments
/// * `shutdown_tx` - Subscribe the sender to recv a shutdown signal.
pub(crate) fn start_watch_config(shutdown_tx: tokio::sync::broadcast::Sender<()>) {
    tokio::spawn(async move {
        result_matcher!(watch_config(shutdown_tx.subscribe()).await, "Failed to watch configuration file");
    });
}

/// Struct of global source info, including `post`, `page`.
/// # Fields
/// * `posts` - List of all post metadata.
/// * `pages` - List of all page metadata.
/// * `categories` - Map of all categories.
/// * `tags` - Map of all tags.
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Site {
    pub posts: Vec<Metadata>,
    pub pages: Vec<Metadata>,
    pub categories: HashMap<String, ClassMap>,
    pub tags: HashMap<String, ClassMap>
}

impl Site {
    pub fn new() -> Self {
        Site { posts: vec![], pages: vec![], categories: HashMap::new(), tags: HashMap::new() }
    }
}

/// Store class info, class can be categories or tags.
/// # Fields
/// * `name` - Class name.
/// * `path` - Class url, normally as the `/self.name`.
/// * `posts` - List of posts that belong to this class.
#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct ClassMap {
    pub name: String,
    pub path: String,
    pub posts: Vec<Metadata>
}

impl ClassMap {
    pub fn new() -> Self {
        ClassMap { name: String::new(), path: String::new(), posts: vec![] }
    }
}

/// Get the path to the source dir (`./source`) in the current directory.
pub(crate) fn get_source_path() -> PathBuf {
    let current_dir = env::current_dir().unwrap();
    current_dir.join("source")
}

/// Load files' [Metadata] of `./source` into `SITE`.
pub(crate) fn get_site() -> Site {
    let source_dir = get_source_path();
    // TODO: update Metadata, load `draft` file into `SITE`
    let post_dir = source_dir.join("post");
    let page_dir = source_dir.join("page");
    let site = Site::new();

    let load = |mut site: Site, dirs: Vec<PathBuf>| -> Site {
        dirs.iter().for_each(|dir| {
            if let Ok(dir) = fs::read_dir(dir) {
                for entry in dir {
                    let entry = entry.unwrap();
                    if !entry.metadata().unwrap().is_file() {
                        continue;
                    }
                    let file = result_matcher!(fs::File::open(entry.path()), "Failed to open and parse file");
                    let metadata = result_matcher!(parse_file(file), "Failed to parse file");
                    site.posts.push(metadata.clone());
                    if let Some(categories) = metadata.categories.as_ref() {
                        for c in categories {
                            if let Some(map) = site.categories.get_mut(c) {
                                map.posts.push(metadata.clone());
                            } else {
                                let mut new_map = ClassMap::new();
                                new_map.name = c.clone();
                                new_map.posts.push(metadata.clone());
                                site.categories.insert(c.clone(), new_map);
                            }
                        }
                    }
                }
            }
        });
        site
    };

    load(site, vec![post_dir, page_dir])
}

pub(crate) static SITE: LazyLock<ArcSwap<Site>> = LazyLock::new(|| {
    let site = get_site();
    ArcSwap::from_pointee(site)
});

pub(crate) async fn watch_source(mut shutdown_rx: tokio::sync::broadcast::Receiver<()>) -> Result<(), Box<dyn Error>> {
    let source_path = get_source_path();
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(tx)?;
    watcher.watch(&source_path, notify::RecursiveMode::Recursive)?;
    loop {
        if shutdown_rx.try_recv().is_ok() {
            break;
        }
        match rx.try_recv() {
            Ok(res) => match res {
                Ok(event) => {
                    match event.kind {
                        notify::EventKind::Modify(_) => {
                            // TODO: avoid read when writing
                            SITE.store(Arc::new(get_site()));
                            dbg!(SITE.load());
                            println!("Site global info reloaded.");
                        },
                        notify::EventKind::Remove(_) => {
                            result_matcher!(watcher.unwatch(&source_path), "Failed to unwatch old source dir");
                            result_matcher!(watcher.watch(&source_path, notify::RecursiveMode::Recursive), "Failed to re-watch source dir");
                        },
                        _ => {}
                    }
                },
                Err(e) => println!("config file watch error: {:?}", e),
            },
            Err(mpsc::TryRecvError::Empty) => {
                // Empty, sleep for a second
                tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
            }
            Err(e) => {
                println!("receive error: {:?}", e);
                return Err("Failed to receive config file change event.".into());
            }
        }
    }
    Ok(())
}

/// Start watching the source dir.
/// # Arguments
/// * `shutdown_tx` - Subscribe the sender to recv a shutdown signal.
pub(crate) fn start_watch_source(shutdown_tx: tokio::sync::broadcast::Sender<()>) {
    tokio::spawn(async move {
        result_matcher!(watch_source(shutdown_tx.subscribe()).await, "Failed to watch source dir");
    });
}
