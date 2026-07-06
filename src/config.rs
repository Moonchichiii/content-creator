use std::{env, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub graph_api_version: String,
    pub posts_config: PathBuf,
    pub post_log: PathBuf,
    pub dry_run: bool,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("missing required environment variable: {0}")]
    Missing(&'static str),
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let _ = dotenvy::dotenv();
        Self::from_lookup(|key| env::var(key).ok())
    }

    fn from_lookup(
        mut lookup: impl FnMut(&'static str) -> Option<String>,
    ) -> Result<Self, ConfigError> {
        let graph_api_version = lookup("GRAPH_API_VERSION").unwrap_or_else(|| "v25.0".to_string());
        let posts_config = required_path("POSTS_CONFIG", &mut lookup)?;
        let post_log = required_path("POST_LOG", &mut lookup)?;
        let dry_run = lookup("DRY_RUN")
            .map(|value| value.eq_ignore_ascii_case("true"))
            .unwrap_or(true);

        Ok(Self {
            graph_api_version,
            posts_config,
            post_log,
            dry_run,
        })
    }
}

fn required_path(
    key: &'static str,
    lookup: &mut impl FnMut(&'static str) -> Option<String>,
) -> Result<PathBuf, ConfigError> {
    lookup(key)
        .map(PathBuf::from)
        .ok_or(ConfigError::Missing(key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_uses_default_graph_api_version_when_missing() {
        let config = Config::from_lookup(|key| match key {
            "POSTS_CONFIG" => Some("assets/posts.toml".to_string()),
            "POST_LOG" => Some("logs/posts.jsonl".to_string()),
            "DRY_RUN" => Some("true".to_string()),
            _ => None,
        })
        .expect("config should load");

        assert_eq!(config.graph_api_version, "v25.0");
        assert_eq!(config.posts_config, PathBuf::from("assets/posts.toml"));
        assert_eq!(config.post_log, PathBuf::from("logs/posts.jsonl"));
        assert!(config.dry_run);
    }

    #[test]
    fn config_requires_posts_config() {
        let error = Config::from_lookup(|key| match key {
            "POST_LOG" => Some("logs/posts.jsonl".to_string()),
            _ => None,
        })
        .expect_err("POSTS_CONFIG should be required");

        assert_eq!(error, ConfigError::Missing("POSTS_CONFIG"));
    }

    #[test]
    fn config_requires_post_log() {
        let error = Config::from_lookup(|key| match key {
            "POSTS_CONFIG" => Some("assets/posts.toml".to_string()),
            _ => None,
        })
        .expect_err("POST_LOG should be required");

        assert_eq!(error, ConfigError::Missing("POST_LOG"));
    }
}
