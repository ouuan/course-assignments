use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;

#[derive(PartialEq, Debug, Deserialize)]
pub struct Config {
    pub word: Option<String>,
    pub random: Option<bool>,
    pub difficult: Option<bool>,
    pub stats: Option<bool>,
    pub day: Option<usize>,
    pub seed: Option<u64>,
    pub final_set: Option<String>,
    pub acceptable_set: Option<String>,
    pub state: Option<String>,
    pub share: Option<bool>,
    pub enable_solver: Option<bool>,
}

pub fn parse_config_file(path: &str) -> Result<Config> {
    let name = format!("the config file [{}]", path);
    let content = fs::read_to_string(path).with_context(|| format!("failed to read {}", name))?;
    crate::json::parse_json(&content, &name)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_config(str: &str) -> serde_json::Result<Config> {
        serde_json::from_str(str)
    }

    #[test]
    fn parse_full_config() {
        assert_eq!(
            parse_config(
                r#"{
  "word": "salet",
  "random": true,
  "difficult": false,
  "stats": true,
  "day": 5,
  "seed": 20220123,
  "final_set": "fin.txt",
  "acceptable_set": "acc.txt",
  "state": "state.json",
  "share": true,
  "enable_solver": false
}
"#
            )
            .expect("parsing a correct config should not return an error"),
            Config {
                word: Some(String::from("salet")),
                random: Some(true),
                difficult: Some(false),
                stats: Some(true),
                day: Some(5),
                seed: Some(20220123),
                final_set: Some(String::from("fin.txt")),
                acceptable_set: Some(String::from("acc.txt")),
                state: Some(String::from("state.json")),
                share: Some(true),
                enable_solver: Some(false),
            }
        );
    }

    #[test]
    fn parse_partial_config() {
        assert_eq!(
            parse_config("{ \"difficult\": true, \"seed\": 123 }")
                .expect("parsing a correct config should not return an error"),
            Config {
                word: None,
                random: None,
                difficult: Some(true),
                stats: None,
                day: None,
                seed: Some(123),
                final_set: None,
                acceptable_set: None,
                state: None,
                share: None,
                enable_solver: None,
            }
        )
    }

    #[test]
    fn parse_incorrect_config() {
        assert!(parse_config("").unwrap_err().is_eof());
        assert!(parse_config("{ random: true }").unwrap_err().is_syntax());
        assert!(parse_config("{ \"word\": 1 }").unwrap_err().is_data());
        assert!(parse_config("{ \"random\": \"false\" }")
            .unwrap_err()
            .is_data());
        assert!(parse_config("{ \"difficult\": \"false\" }")
            .unwrap_err()
            .is_data());
        assert!(parse_config("{ \"stats\": \"true\" }")
            .unwrap_err()
            .is_data());
        assert!(parse_config("{ \"day\": 1000000000000000000000 }")
            .unwrap_err()
            .is_data());
        assert!(parse_config("{ \"seed\": -233 }").unwrap_err().is_data());
        assert!(parse_config("{ \"final_set\": true }")
            .unwrap_err()
            .is_data());
        assert!(parse_config("{ \"acceptable_set\": true }")
            .unwrap_err()
            .is_data());
        assert!(parse_config("{ \"state\": true }").unwrap_err().is_data());
        assert!(parse_config("{ \"share\": 0 }").unwrap_err().is_data());
        assert!(parse_config("{ \"enable_solver\": 1 }")
            .unwrap_err()
            .is_data());
    }
}
