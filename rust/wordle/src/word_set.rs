mod builtin_words;

use anyhow::{anyhow, Context, Result};
use rand::{rngs::StdRng, seq::SliceRandom, SeedableRng};
use std::collections::HashSet;
use std::fs;

fn get_word_set(path: &Option<String>, name: &str, default: &[&str]) -> Result<HashSet<String>> {
    let set = match path {
        Some(path) => {
            let content = fs::read_to_string(path)
                .with_context(|| format!("failed to read the {} set file [{}]", name, path))?;
            let mut set = HashSet::new();
            for line in content.lines() {
                let word = line.to_ascii_uppercase();
                if set.get(&word).is_some() {
                    return Err(anyhow!(
                        "the {} set contains duplicated words \"{}\"",
                        name,
                        word
                    ));
                }
                set.insert(word);
            }
            set
        }
        None => default.iter().map(|&s| s.to_ascii_uppercase()).collect(),
    };
    for word in &set {
        crate::valid_word::check_format(word, &format!("word in {} set", name))?;
    }
    Ok(set)
}

pub fn get_acceptable_set(path: &Option<String>) -> Result<HashSet<String>> {
    get_word_set(path, "acceptable", builtin_words::ACCEPTABLE)
}

// get the final set and a shuffled final list
pub fn get_final_set(
    path: &Option<String>,
    acceptable_set: &HashSet<String>,
    random: bool,
    seed: Option<u64>,
) -> Result<(HashSet<String>, Vec<String>)> {
    let final_set = get_word_set(path, "final", builtin_words::FINAL)?;

    for word in &final_set {
        if acceptable_set.get(word).is_none() {
            return Err(anyhow!(
                "the word {} is in the final set but is not in the acceptable set",
                word
            ));
        }
    }

    let mut final_list = final_set.iter().cloned().collect::<Vec<_>>();
    final_list.sort();
    if random {
        final_list.shuffle(&mut StdRng::seed_from_u64(seed.unwrap_or(114514)));
    }

    Ok((final_set, final_list))
}
