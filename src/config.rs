use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Config {
    pub theme: Option<String>,
    pub width: Option<u16>,
    pub spacing: Option<String>,
    pub toc: Option<bool>,
    pub frontmatter: Option<bool>,
}

/// Load config from ~/.config/ink/config.toml
pub fn load_config() -> Option<Config> {
    let config_dir = dirs::config_dir()?;
    let config_path = config_dir.join("ink").join("config.toml");
    let content = std::fs::read_to_string(config_path).ok()?;
    toml::from_str(&content).ok()
}
