use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use serde_json::error::Category;

/// It parses JSON from *content* and returns a *T* or provides friendly error messages
///
/// The *name* parameter is used in error messages and can look like "the config file [config.json]"
pub fn parse_json<'a, 'b, T: Deserialize<'a>>(content: &'a str, name: &'b str) -> Result<T> {
    if content.trim().is_empty() {
        return Err(anyhow!("{} is empty", name));
    }
    match serde_json::from_str(&content) {
        Ok(value) => Ok(value),
        Err(error) => {
            let reason = match error.classify() {
                Category::Io => "failed to parse",
                Category::Eof => "unexpected EOF when parsing",
                Category::Syntax => "invalid JSON syntax in",
                Category::Data => "invalid content in",
            };
            Err(error).context(format!("{} {}", reason, name))
        }
    }
}
