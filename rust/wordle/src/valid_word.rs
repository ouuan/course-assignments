use crate::LETTER_NUM_IN_WORD;
use anyhow::{anyhow, Result};
use std::collections::HashSet;
use std::ops::Deref;

/// A wrapper of `[char; LETTER_NUM_IN_WORD]` which guarantees to be a valid word in the `word_set`.
#[derive(PartialEq, Debug, Clone)]
pub struct ValidWord([char; LETTER_NUM_IN_WORD]);

/// Validate a word for format only.
///
/// # Parameters
///
/// -   `word`: the word to be validated and to construct the `ValidWord` from
/// -   `word_type`: usually "guess" or "answer", used in the error message
pub fn check_format(word: &str, word_type: &str) -> Result<()> {
    if !word.is_ascii() {
        return Err(anyhow!(
            "the {} \"{}\" contains non-ascii characters",
            word_type,
            word,
        ));
    }
    if !word.chars().all(|c| c.is_ascii_alphabetic()) {
        return Err(anyhow!(
            "the {} \"{}\" contains non-alphabetic characters",
            word_type,
            word,
        ));
    }

    if word.len() != LETTER_NUM_IN_WORD {
        return Err(anyhow!(
            "the {} \"{}\" contains {} letters, expect exactly {} letters",
            word_type,
            word,
            word.len(),
            LETTER_NUM_IN_WORD
        ));
    }

    Ok(())
}

impl ValidWord {
    /// Validate format and presence in the word set and construct a new `ValidWord`.
    ///
    /// # Parameters
    ///
    /// -   `word`: the word to be validated and to construct the `ValidWord` from
    /// -   `word_type`: usually "guess" or "answer", used in the error message
    /// -   `word_set`: the set of valid words
    /// -   `set_name`: the name of the word set used in the error message
    pub fn new(
        word: &str,
        word_type: &str,
        word_set: &HashSet<String>,
        set_name: &str,
    ) -> Result<Self> {
        check_format(word, word_type)?;
        let word = word.to_ascii_uppercase();
        if word_set.get(&word).is_none() {
            return Err(anyhow!(
                "the {} \"{}\" is not in the {} word list",
                word_type,
                word,
                set_name,
            ));
        }
        unsafe { Ok(Self::new_unchecked(&word)) }
    }

    /// Construct a ValidWord without checking the requirements.
    /// Used in the solver for better performance.
    pub unsafe fn new_unchecked(word: &str) -> Self {
        // Can be ValidWord(word.chars().collect::<Vec<_>>().try_into().unwrap()),
        // but that will have worse performance
        let mut iter = word.chars();
        ValidWord([
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
            iter.next().unwrap(),
        ])
    }

    pub fn to_string(&self) -> String {
        self.iter().collect()
    }
}

/// Implement Deref so that the `.0` is usually not needed.
impl Deref for ValidWord {
    type Target = [char; LETTER_NUM_IN_WORD];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn construct_word_set() -> HashSet<String> {
        HashSet::from_iter(
            ["TRACE", "FIGHT", "测试五个字", "12345", "abcd", "abcdef"]
                .iter()
                .map(|&s| String::from(s)),
        )
    }

    #[test]
    fn construct_valid_word() {
        assert_eq!(
            ValidWord::new("tRacE", "test", &construct_word_set(), "test")
                .expect("valid word should be successfully constructed"),
            ValidWord(['T', 'R', 'A', 'C', 'E'])
        );
    }

    #[test]
    fn invalid_characters() {
        assert!(ValidWord::new("测试五个字", "test", &construct_word_set(), "test").is_err());
        assert!(ValidWord::new("12345", "test", &construct_word_set(), "test").is_err());
    }

    #[test]
    fn invalid_length() {
        assert!(ValidWord::new("abcd", "test", &construct_word_set(), "test").is_err());
        assert!(ValidWord::new("abcdef", "test", &construct_word_set(), "test").is_err());
    }
}
