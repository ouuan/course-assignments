//! Some utility functions.

use crate::error::*;
use crate::TIME_FORMAT;
use chrono::NaiveDateTime;

/// Parse `NaiveDateTime` from string with friendly error messages.
pub fn parse_time(s: &str, name: &str) -> ApiResult<NaiveDateTime> {
    NaiveDateTime::parse_from_str(s, TIME_FORMAT).map_err(|error| {
        ApiError::new(
            ApiErrorType::InvalidArgument,
            format!(
                "The '{}' [{}] is not a valid time. Should be of format [{}]. Error: {}.",
                name, s, TIME_FORMAT, error
            ),
        )
    })
}
