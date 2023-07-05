//! Enums used in the database.

use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

/// The state of a job.
#[derive(Debug, DbEnum, Serialize, Deserialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "PascalCase")]
#[DbValueStyle = "PascalCase"]
pub enum JobState {
    Queueing,
    Running,
    Finished,
    Canceled,
}

/// The result of a job or a case.
#[derive(Debug, DbEnum, Serialize, Deserialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "PascalCase")]
#[DbValueStyle = "PascalCase"]
pub enum JobResult {
    Waiting,
    Running,
    Accepted,
    #[serde(rename = "Compilation Error")]
    #[db_rename = "Compilation Error"]
    CompilationError,
    #[serde(rename = "Compilation Success")]
    #[db_rename = "Compilation Success"]
    CompilationSuccess,
    #[serde(rename = "Wrong Answer")]
    #[db_rename = "Wrong Answer"]
    WrongAnswer,
    #[serde(rename = "Runtime Error")]
    #[db_rename = "Runtime Error"]
    RuntimeError,
    #[serde(rename = "Time Limit Exceeded")]
    #[db_rename = "Time Limit Exceeded"]
    TimeLimitExceeded,
    #[serde(rename = "Memory Limit Exceeded")]
    #[db_rename = "Memory Limit Exceeded"]
    MemoryLimitExceeded,
    #[serde(rename = "System Error")]
    #[db_rename = "System Error"]
    SystemError,
    #[serde(rename = "SPJ Error")]
    #[db_rename = "SPJ Error"]
    SPJError,
    Skipped,
}
