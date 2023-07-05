//! Get the configuration.

mod deserialize;

use anyhow::{anyhow, bail, Context, Result};
use serde_json::error::Category;
use std::collections::HashSet;
use std::collections::{hash_map::Entry, HashMap};
use std::time::Duration;

pub use deserialize::ServerConfig;

/// The type of a problem with type-specific data.
#[derive(Debug, Clone)]
pub enum ProblemType {
    Standard,
    Strict,
    Spj { command: String, args: Vec<String> },
    DynamicRanking { ratio: f64 },
}

#[readonly::make]
#[derive(Debug, Clone)]
pub struct TestCase {
    pub score: f64,
    pub input_file: String,
    pub answer_file: String,
    pub time_limit: Duration,
    pub memory_limit: usize,
}

#[readonly::make]
#[derive(Debug, Clone)]
pub struct Problem {
    pub id: i32,
    pub name: String,
    pub tp: ProblemType,
    pub cases: Vec<TestCase>,
    pub packing: Vec<Vec<i32>>,
}

#[derive(Debug, Clone)]
pub struct Language {
    pub command: String,
    pub args: Vec<String>,
    pub file_name: String,
}

pub type ProblemMap = HashMap<i32, Problem>;
pub type LanguageMap = HashMap<String, Language>;

#[derive(Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub problem_map: ProblemMap,
    pub language_map: LanguageMap,
}

impl Config {
    /// Parse, validate and construct the config from a JSON string.
    pub fn new(json: &str) -> Result<Self> {
        if json.trim().is_empty() {
            bail!("config is empty");
        }
        match serde_json::from_str::<deserialize::Config>(&json) {
            Ok(config) => {
                let mut problem_map = HashMap::new();

                for problem in config.problems {
                    match problem_map.entry(problem.id) {
                        Entry::Occupied(_) => {
                            bail!("config contains duplicate problem id {}", problem.id);
                        }
                        Entry::Vacant(entry) => {
                            let packing = match problem
                                .misc
                                .as_ref()
                                .and_then(|misc| misc.packing.as_ref())
                            {
                                None => {
                                    // Treat each test case of a problem without packing as a
                                    // subtask with only a single case.
                                    (0..problem.cases.len()).map(|id| vec![id as i32]).collect()
                                }
                                Some(packing) => {
                                    // Minus case ids by one to get case indices.
                                    // Sort cases by id.
                                    let mut packing = packing
                                        .iter()
                                        .map(|subtask| {
                                            let mut subtask = subtask
                                                .iter()
                                                .map(|case| *case - 1)
                                                .collect::<Vec<_>>();
                                            subtask.sort();
                                            subtask
                                        })
                                        .collect::<Vec<_>>();
                                    packing.sort();
                                    // Check that the packing is a partition.
                                    let mut used_in_packing = HashSet::new();
                                    for subtask in &packing {
                                        for case in subtask {
                                            if *case < 0 || *case >= problem.cases.len() as i32 {
                                                bail!(
                                                    "the packing of problem {} contains case id {} which is out of the bound of [1, {}]",
                                                    problem.id, case + 1, problem.cases.len()
                                                );
                                            }
                                            if !used_in_packing.insert(case) {
                                                bail!("duplicated test case {} in the packing of problem {}", case, problem.id);
                                            }
                                        }
                                    }
                                    if used_in_packing.len() != problem.cases.len() {
                                        bail!(
                                            "missing cases in the packing of problem {}",
                                            problem.id
                                        );
                                    }
                                    packing
                                }
                            };

                            let tp = match problem.tp {
                                deserialize::ProblemType::Standard => ProblemType::Standard,
                                deserialize::ProblemType::Strict => ProblemType::Strict,
                                deserialize::ProblemType::Spj => {
                                    let mut command_iter = problem.misc
                                        .ok_or(anyhow!("problem {} is of spj type but has no misc field", problem.id))?
                                        .special_judge
                                        .ok_or(anyhow!("problem {} is of spj type but has no misc.special_judge field", problem.id))?
                                        .into_iter();
                                    // Split the command into command and args.
                                    let command = command_iter.next().ok_or(anyhow!("problem {} has empty spj command", problem.id))?;
                                    let args = command_iter.collect();
                                    ProblemType::Spj {command, args}
                                },
                                deserialize::ProblemType::DynamicRanking => ProblemType::DynamicRanking {
                                    ratio: problem.misc
                                        .ok_or(anyhow!("problem {} is of dynamic_ranking type but has no misc field", problem.id))?
                                        .dynamic_ranking_ratio
                                        .ok_or(anyhow!("problem {} is of dynamic_ranking type but has no misc.dynamic_ranking_ratio field", problem.id))?,
                                },
                            };

                            // Ensure that the total score is 100.
                            let total_score =
                                problem.cases.iter().map(|case| case.score).sum::<f64>();
                            if (total_score - 100.0).abs() > 1e-10 {
                                bail!(
                                    "the total score of problem {} is {} instead of 100",
                                    problem.id,
                                    total_score
                                );
                            }

                            // Transform time limit from micros to `Duration`.
                            // Transform no limit to the `MAX` value.
                            let cases = problem
                                .cases
                                .into_iter()
                                .map(|case| TestCase {
                                    score: case.score,
                                    input_file: case.input_file,
                                    answer_file: case.answer_file,
                                    time_limit: match case.time_limit {
                                        0 => Duration::MAX,
                                        micros => Duration::from_micros(micros),
                                    },
                                    memory_limit: match case.memory_limit {
                                        0 => usize::MAX,
                                        bytes => bytes,
                                    },
                                })
                                .collect();

                            entry.insert(Problem {
                                id: problem.id,
                                name: problem.name,
                                tp,
                                cases,
                                packing,
                            });
                        }
                    }
                }

                let mut language_map = HashMap::new();

                for language in config.languages {
                    match language_map.entry(language.name.clone()) {
                        Entry::Occupied(_) => {
                            bail!("duplicate language name {} in the config", language.name);
                        }
                        Entry::Vacant(entry) => {
                            // Split command into command and args.
                            let mut iter = language.command.into_iter();
                            let command = match iter.next() {
                                None => bail!("language {} has empty command", language.name),
                                Some(command) => command,
                            };
                            let args = iter.collect();
                            entry.insert(Language {
                                command,
                                args,
                                file_name: language.file_name,
                            });
                        }
                    }
                }

                Ok(Self {
                    server: config.server,
                    problem_map,
                    language_map,
                })
            }
            Err(error) => {
                let reason = match error.classify() {
                    Category::Io => "failed to parse",
                    Category::Eof => "unexpected EOF when parsing",
                    Category::Syntax => "invalid JSON syntax in",
                    Category::Data => "invalid content in",
                };
                Err(error).context(format!("{} the config", reason))
            }
        }
    }
}
