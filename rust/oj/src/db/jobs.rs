//! Database operations on the `jobs` table.

use super::case_results::{self, Case};
use super::connection::ConnectionPool;
use super::enums::{JobResult, JobState};
use super::schema::jobs::dsl;
use super::{contest_problems, contest_users, contests, users};
use crate::error::*;
use crate::judger::JobAdder;
use crate::TIME_FORMAT;
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Get jobs with either Queueing or Running state.
/// This can be used to continue running unfinished jobs at startup.
pub fn get_unfinished_jobs(pool: &ConnectionPool) -> ApiResult<Vec<i32>> {
    Ok(dsl::jobs
        .select(dsl::id)
        .filter(dsl::state.eq(JobState::Queueing))
        .or_filter(dsl::state.eq(JobState::Running))
        .order(dsl::id)
        .load(&mut pool.get()?)?)
}

/// Job info that is useful for the judger.
#[derive(Queryable)]
pub struct JobInfoForJudger {
    pub source_code: String,
    pub language: String,
    pub problem_id: i32,
}

/// Returns `Some(JobInfoForJudger)` and set state to "Running" if job is not canceled;
/// returns `None` and do no update if job is canceled.
pub fn fetch_job_for_judger(id: i32, pool: &ConnectionPool) -> ApiResult<Option<JobInfoForJudger>> {
    pool.get()?.immediate_transaction(|conn| {
        let state = dsl::jobs
            .select(dsl::state)
            .filter(dsl::id.eq(id))
            .first::<JobState>(conn)?;
        if state == JobState::Canceled {
            return Ok(None);
        }
        diesel::update(dsl::jobs)
            .filter(dsl::id.eq(id))
            .set((
                dsl::updated_time.eq(Utc::now().naive_utc()),
                dsl::state.eq(JobState::Running),
            ))
            .execute(conn)?;
        let info = dsl::jobs
            .select((dsl::source_code, dsl::language, dsl::problem_id))
            .filter(dsl::id.eq(id))
            .first(conn)?;
        Ok(Some(info))
    })
}

/// Set job state to finished and set other fields in the parameters.
pub fn finish_job(id: i32, result: &JobResult, score: f64, pool: &ConnectionPool) -> ApiResult<()> {
    diesel::update(dsl::jobs)
        .filter(dsl::id.eq(id))
        .set((
            dsl::updated_time.eq(Utc::now().naive_utc()),
            dsl::state.eq(JobState::Finished),
            dsl::result.eq(result),
            dsl::score.eq(score),
        ))
        .execute(&mut pool.get()?)?;
    Ok(())
}

/// Update the score of a job.
pub(super) fn update_score(id: i32, score: f64, conn: &mut SqliteConnection) -> ApiResult<()> {
    diesel::update(dsl::jobs)
        .filter(dsl::id.eq(id))
        .set((
            dsl::updated_time.eq(Utc::now().naive_utc()),
            dsl::score.eq(score),
        ))
        .execute(conn)?;
    Ok(())
}

/// The columns of a job in the database.
#[derive(Queryable, Insertable, AsChangeset)]
#[diesel(table_name = super::schema::jobs)]
pub struct JobInfo {
    pub id: i32,
    created_time: NaiveDateTime,
    updated_time: NaiveDateTime,
    source_code: String,
    language: String,
    pub user_id: i32,
    contest_id: i32,
    pub problem_id: i32,
    state: JobState,
    pub result: JobResult,
    pub score: f64,
}

/// The API request of a submission.
#[derive(Deserialize, Serialize)]
pub struct Submission {
    source_code: String,
    pub language: String,
    user_id: i32,
    contest_id: i32,
    pub problem_id: i32,
}

/// The API response of a job.
#[derive(Serialize)]
pub struct Job {
    id: i32,
    created_time: String,
    updated_time: String,
    submission: Submission,
    state: JobState,
    result: JobResult,
    score: f64,
    cases: Vec<Case>,
}

impl Job {
    fn new(info: JobInfo, cases: Vec<Case>) -> Self {
        Self {
            id: info.id,
            created_time: info.created_time.format(TIME_FORMAT).to_string(),
            updated_time: info.updated_time.format(TIME_FORMAT).to_string(),
            submission: Submission {
                source_code: info.source_code,
                language: info.language,
                user_id: info.user_id,
                contest_id: info.contest_id,
                problem_id: info.problem_id,
            },
            state: info.state,
            result: info.result,
            score: info.score,
            cases,
        }
    }
}

/// Check a submission against a contest.
fn check_contest_submission(
    submission: &Submission,
    now: NaiveDateTime,
    conn: &mut SqliteConnection,
) -> ApiResult<()> {
    let Submission {
        user_id,
        contest_id,
        problem_id,
        ..
    } = *submission;

    let contest_info = contests::get_contest_info(contest_id, conn)?;

    if !contest_users::is_user_in_contest(contest_id, user_id, conn)? {
        return Err(ApiError::new(
            ApiErrorType::InvalidArgument,
            format!("User {} is not in contest {}.", user_id, contest_id),
        ));
    }
    if !contest_problems::is_problem_in_contest(contest_id, problem_id, conn)? {
        return Err(ApiError::new(
            ApiErrorType::InvalidArgument,
            format!("Problem {} is not in contest {}.", problem_id, contest_id),
        ));
    }

    let contest_submission_count = dsl::jobs
        .count()
        .filter(dsl::problem_id.eq(problem_id))
        .filter(dsl::user_id.eq(user_id))
        .filter(dsl::contest_id.eq(contest_id))
        .get_result::<i64>(conn)?;
    if contest_submission_count >= contest_info.submission_limit as i64 {
        return Err(ApiError::new(
            ApiErrorType::RateLimit,
            String::from("Submission limit exceeded."),
        ));
    }

    if now < contest_info.from {
        return Err(ApiError::new(
            ApiErrorType::InvalidArgument,
            String::from("Contest has not started."),
        ));
    }
    if now > contest_info.to {
        return Err(ApiError::new(
            ApiErrorType::InvalidArgument,
            String::from("Contest is over."),
        ));
    }

    Ok(())
}

/// Add a new job. This does not check that language and problem_id are in the config.
pub fn add_job(
    submission: Submission,
    case_count: usize,
    pool: &ConnectionPool,
    adder: &JobAdder,
) -> ApiResult<Job> {
    pool.get()?.immediate_transaction(|conn| {
        let user_count = users::user_count(conn)?;
        if submission.user_id < 0 || submission.user_id >= user_count {
            return Err(ApiError::not_found(&format!("User {}", submission.user_id)));
        }
        let now = Utc::now().naive_utc();
        if submission.contest_id != 0 {
            check_contest_submission(&submission, now, conn)?;
        }
        let id = dsl::jobs.count().get_result::<i64>(conn)? as i32;
        let job_info = JobInfo {
            id,
            created_time: now,
            updated_time: now,
            source_code: submission.source_code,
            language: submission.language,
            user_id: submission.user_id,
            contest_id: submission.contest_id,
            problem_id: submission.problem_id,
            state: JobState::Queueing,
            result: JobResult::Waiting,
            score: 0.0,
        };
        diesel::insert_into(dsl::jobs)
            .values(&job_info)
            .execute(conn)?;
        let cases = case_results::init_cases(id, case_count, conn)?;
        adder.add_job(job_info.id)?;
        Ok(Job::new(job_info, cases))
    })
}

/// Get a single job info.
fn get_job_info(id: i32, conn: &mut SqliteConnection) -> ApiResult<JobInfo> {
    let job_info = dsl::jobs.filter(dsl::id.eq(id)).first(conn).optional()?;
    match job_info {
        Some(info) => Ok(info),
        None => Err(ApiError::not_found(&format!("Job {}", id))),
    }
}

/// Get a job with case results.
pub fn get_job(id: i32, pool: &ConnectionPool) -> ApiResult<Job> {
    pool.get()?.immediate_transaction(|conn| {
        let job_info = get_job_info(id, conn)?;
        let cases = case_results::get_cases(id, conn)?;
        Ok(Job::new(job_info, cases))
    })
}

/// The job filters in the API query params.
#[derive(Deserialize, Queryable)]
pub struct JobFilter {
    pub user_id: Option<i32>,
    pub user_name: Option<String>,
    pub contest_id: Option<i32>,
    pub problem_id: Option<i32>,
    pub language: Option<String>,
    // The Deserialize of NaiveDateTime uses a different format, so manually parse instead
    pub from: Option<String>,
    pub to: Option<String>,
    pub state: Option<JobState>,
    pub result: Option<JobResult>,
}

/// Get a list of jobs under the given filter.
pub fn get_jobs(filter: &JobFilter, pool: &ConnectionPool) -> ApiResult<Vec<Job>> {
    pool.get()?.immediate_transaction(|conn| {
        let mut query = dsl::jobs.order(dsl::id).into_boxed();
        if let Some(user_id) = filter.user_id {
            query = query.filter(dsl::user_id.eq(user_id));
        }
        if let Some(contest_id) = filter.contest_id {
            query = query.filter(dsl::contest_id.eq(contest_id));
        }
        if let Some(problem_id) = filter.problem_id {
            query = query.filter(dsl::problem_id.eq(problem_id));
        }
        if let Some(language) = &filter.language {
            query = query.filter(dsl::language.eq(language));
        }
        if let Some(from) = &filter.from {
            query = query.filter(dsl::created_time.ge(super::utils::parse_time(from, "from")?));
        }
        if let Some(to) = &filter.to {
            query = query.filter(dsl::created_time.le(super::utils::parse_time(to, "to")?));
        }
        if let Some(state) = filter.state {
            query = query.filter(dsl::state.eq(state));
        }
        if let Some(result) = filter.result {
            query = query.filter(dsl::result.eq(result));
        }
        if let Some(user_name) = &filter.user_name {
            match users::get_user_id(user_name, conn)? {
                None => return Ok(Vec::new()),
                Some(user_id) => query = query.filter(dsl::user_id.eq(user_id)),
            }
        }
        let jobs_info = query.load::<JobInfo>(conn)?;
        let mut jobs = Vec::new();
        for info in jobs_info {
            let cases = case_results::get_cases(info.id, conn)?;
            jobs.push(Job::new(info, cases));
        }
        Ok(jobs)
    })
}

/// Rejudge a single job.
pub fn rejudge(id: i32, adder: &JobAdder, pool: &ConnectionPool) -> ApiResult<Job> {
    let (job_info, cases) = pool.get()?.immediate_transaction(|conn| {
        let mut job_info = get_job_info(id, conn)?;
        if job_info.state != JobState::Finished {
            return Err(ApiError::new(
                ApiErrorType::InvalidState,
                format!("Job {} not finished.", id),
            ));
        }
        job_info.state = JobState::Queueing;
        job_info.result = JobResult::Waiting;
        job_info.score = 0.0;
        job_info.updated_time = Utc::now().naive_utc();
        diesel::update(dsl::jobs)
            .filter(dsl::id.eq(id))
            .set(&job_info)
            .execute(conn)?;
        case_results::reinit_cases(id, conn)?;
        let cases = case_results::get_cases(id, conn)?;
        Ok((job_info, cases))
    })?;
    adder.add_job(id)?;
    Ok(Job::new(job_info, cases))
}

/// Cancel a single job if it's queueing.
pub fn cancel_job(id: i32, pool: &ConnectionPool) -> ApiResult<()> {
    pool.get()?.immediate_transaction(|conn| {
        let job_info = get_job_info(id, conn)?;
        if job_info.state != JobState::Queueing {
            return Err(ApiError::new(
                ApiErrorType::InvalidState,
                format!("Job {} not queueing.", id),
            ));
        }
        diesel::update(dsl::jobs)
            .filter(dsl::id.eq(id))
            .set(dsl::state.eq(JobState::Canceled))
            .execute(conn)?;
        Ok(())
    })
}

/// Get a list of all job info.
pub fn get_all_job_info(pool: &ConnectionPool) -> ApiResult<Vec<JobInfo>> {
    Ok(dsl::jobs.load(&mut pool.get()?)?)
}

/// Get a list of all job info in the given contest.
///
/// Note: It returns an empty list without an error if the contest does not exist.
pub fn get_contest_jobs_info(contest_id: i32, pool: &ConnectionPool) -> ApiResult<Vec<JobInfo>> {
    Ok(dsl::jobs
        .filter(dsl::contest_id.eq(contest_id))
        .load(&mut pool.get()?)?)
}
