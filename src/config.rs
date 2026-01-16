use directories::ProjectDirs;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub diff: DiffConfig,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct DiffConfig {
    /// Diff tool to use: "auto", "delta", "diff-so-fancy", "difftastic", "colordiff", "git",
    /// or any custom command name
    pub tool: String,
    /// Additional arguments to pass to the diff tool
    pub args: Vec<String>,
}

impl Default for DiffConfig {
    fn default() -> Self {
        Self {
            tool: "auto".to_string(),
            args: Vec::new(),
        }
    }
}

impl Config {
    /// Load configuration from file. Returns default config if file doesn't exist or fails to parse.
    pub fn load() -> Self {
        match Self::try_load() {
            Ok(config) => config,
            Err(e) => {
                if !matches!(e, ConfigError::NotFound) {
                    eprintln!("Warning: Failed to load config: {e}. Using defaults.");
                }
                Self::default()
            }
        }
    }

    fn try_load() -> Result<Self, ConfigError> {
        let path = Self::config_path().ok_or(ConfigError::NotFound)?;

        if !path.exists() {
            return Err(ConfigError::NotFound);
        }

        let content = fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Get the configuration file path (~/.config/diff-tui/config.toml)
    fn config_path() -> Option<PathBuf> {
        ProjectDirs::from("", "", "diff-tui").map(|dirs| dirs.config_dir().join("config.toml"))
    }
}

#[derive(Debug)]
enum ConfigError {
    NotFound,
    Io(std::io::Error),
    Parse(toml::de::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::NotFound => write!(f, "config file not found"),
            ConfigError::Io(e) => write!(f, "IO error: {e}"),
            ConfigError::Parse(e) => write!(f, "parse error: {e}"),
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        ConfigError::Io(e)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(e: toml::de::Error) -> Self {
        ConfigError::Parse(e)
    }
}
