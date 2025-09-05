use std::{env, fs, sync::LazyLock};

use arc_swap::ArcSwap;
use serde::{Deserialize, Serialize};

pub mod run;

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Config {
    pub site: SiteConfig
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct SiteConfig {
    pub title: String,
    pub author: String,
    pub url: String,
    pub zone: String,
    pub theme: String,
    pub favicon: String,
    pub menu: Vec<Menu>
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct Menu {
    pub name: String,
    pub link: String
}

pub(crate) static CONFIG: LazyLock<ArcSwap<Config>> = LazyLock::new(|| {
    let current_dir = env::current_dir().unwrap();
    let config_path = current_dir.join("tless.toml");
    if !config_path.exists() {
        panic!("Configuration file not found at {}", config_path.display());
    }
    let config_content = fs::read_to_string(config_path)
        .expect("Failed to read configuration file");
    let config: Config = toml::from_str(&config_content).expect("Failed to parse configuration file");
    ArcSwap::from_pointee(config)
});
