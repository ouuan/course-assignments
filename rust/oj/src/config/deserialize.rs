//! Structs that match the config file format for deserializing the config.

use serde::Deserialize;

fn default_address() -> String {
    String::from("127.0.0.1")
}

fn default_port() -> u16 {
    12345
}

#[derive(Deserialize, Debug, Clone)]
pub struct ServerConfig {
    #[serde(default = "default_address")]
    pub bind_address: String,
    #[serde(default = "default_port")]
    pub bind_port: u16,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProblemType {
    Standard,
    Strict,
    Spj,
    DynamicRanking,
}

#[derive(Deserialize)]
pub struct ProblemMisc {
    pub packing: Option<Vec<Vec<i32>>>,
    pub special_judge: Option<Vec<String>>,
    pub dynamic_ranking_ratio: Option<f64>,
}

#[derive(Deserialize)]
pub struct TestCase {
    pub score: f64,
    pub input_file: String,
    pub answer_file: String,
    pub time_limit: u64,
    pub memory_limit: usize,
}

#[derive(Deserialize)]
pub struct Problem {
    pub id: i32,
    pub name: String,
    #[serde(rename = "type")]
    pub tp: ProblemType,
    pub misc: Option<ProblemMisc>,
    pub cases: Vec<TestCase>,
}

#[derive(Deserialize)]
pub struct Language {
    pub name: String,
    pub file_name: String,
    pub command: Vec<String>,
}

#[derive(Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub problems: Vec<Problem>,
    pub languages: Vec<Language>,
}
