use crate::config::{Problem, ProblemMap, ProblemType};
use crate::db::case_results;
use crate::db::connection::ConnectionPool;
use crate::db::contests;
use crate::db::enums::JobResult;
use crate::db::jobs::{self, JobInfo};
use crate::db::users::{self, User};
use crate::error::*;
use actix_web::{get, web, Responder};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{hash_map::Entry, HashMap, HashSet};

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum ScoringRule {
    Latest,
    Highest,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum TieBreaker {
    SubmissionTime,
    SubmissionCount,
    UserId,
}

fn default_scoring_rule() -> ScoringRule {
    ScoringRule::Latest
}

/// The query params of the ranklist API.
#[derive(Deserialize)]
struct RankListQuery {
    #[serde(default = "default_scoring_rule")]
    scoring_rule: ScoringRule,
    tie_breaker: Option<TieBreaker>,
}

/// An item in the ranklist.
#[derive(Serialize)]
struct RankItem {
    user: User,
    rank: usize,
    scores: Vec<f64>,
    #[serde(skip)]
    total_score: f64,
    #[serde(skip)]
    last_job_id: i32,
    #[serde(skip)]
    submission_count: u32,
}

#[get("/{id}/ranklist")]
async fn ranklist(
    contest_id: web::Path<i32>,
    query: web::Query<RankListQuery>,
    pool: web::Data<ConnectionPool>,
    problem_map: web::Data<ProblemMap>,
) -> ApiResult<impl Responder> {
    let contest_id = contest_id.into_inner();

    let (jobs, users, problem_ids) =
        get_jobs_users_problem_ids(contest_id, pool.clone(), &problem_map).await?;
    let problem_list = get_problem_list(&problem_ids, &problem_map)?;

    let (mut submission_for_rank, submission_count) = get_submission_for_rank_and_count(
        &users,
        &problem_ids,
        jobs,
        &problem_map,
        &query.scoring_rule,
    );

    for problem in problem_list {
        update_scores_for_dynamic_ranking(&mut submission_for_rank, problem, pool.clone()).await?;
    }

    let mut rank_list =
        get_unsorted_ranklist(&submission_for_rank, &submission_count, users, &problem_ids);

    sort_ranklist(&mut rank_list, &query.tie_breaker);

    Ok(web::Json(rank_list))
}

/// Get jobs, users and problem ids of the given contest.
///
/// If `contest_id` is zero, all jobs, users, and problem ids will be returned.
async fn get_jobs_users_problem_ids(
    contest_id: i32,
    pool: web::Data<ConnectionPool>,
    problem_map: &ProblemMap,
) -> ApiResult<(Vec<JobInfo>, Vec<User>, Vec<i32>)> {
    if contest_id == 0 {
        let (jobs, users) = web::block(move || -> ApiResult<_> {
            Ok((jobs::get_all_job_info(&pool)?, users::get_users(&pool)?))
        })
        .await??;

        // `users::get_users` is already sorted, so no sort is needed for users.
        // But problem_ids needs sort.
        let mut problem_ids = problem_map.keys().cloned().collect::<Vec<_>>();
        problem_ids.sort_unstable();

        Ok((jobs, users, problem_ids))
    } else {
        let ((mut users, problem_ids), jobs) = web::block(move || -> ApiResult<_> {
            Ok((
                contests::get_contest_users_and_problem_ids(contest_id, &pool)?,
                jobs::get_contest_jobs_info(contest_id, &pool)?,
            ))
        })
        .await??;

        // Sort users by id to get correct order on tie.
        users.sort_unstable_by_key(|user| user.id);

        Ok((jobs, users, problem_ids))
    }
}

/// Get a list of `Problem`s of the given ids.
///
/// It also checks that every problem id exists in the problem map, so that subsequent codes can
/// use `problem_map.get(problem_id).unwrap()` without handling the error.
fn get_problem_list<'a>(
    problem_ids: &[i32],
    problem_map: &'a ProblemMap,
) -> ApiResult<Vec<&'a Problem>> {
    let mut problem_list = Vec::new();
    for id in problem_ids {
        match problem_map.get(id) {
            None => {
                return Err(ApiError::new(
                    ApiErrorType::Internal,
                    format!("Problem {} not found.", id),
                ))
            }
            Some(problem) => problem_list.push(problem),
        }
    }
    Ok(problem_list)
}

/// Compare two `f64`s using `f64::total_cmp` but returns `Ordering::Equal` when their difference
/// is small enough.
fn eps_cmp(lhs: f64, rhs: f64) -> Ordering {
    if (lhs - rhs).abs() < 1e-10 {
        Ordering::Equal
    } else {
        lhs.total_cmp(&rhs)
    }
}

/// Get the submission used for ranking for each user and each problem, and get the submission
/// count of each user.
///
/// The return value is a pair.
/// The first element is a map of the used submissions, with user id as the outer key and problem
/// id as the inner key.
/// The second element is a map of submission count with user id as the key.
fn get_submission_for_rank_and_count(
    users: &[User],
    problem_ids: &[i32],
    jobs: Vec<JobInfo>,
    problem_map: &ProblemMap,
    scoring_rule: &ScoringRule,
) -> (HashMap<i32, HashMap<i32, JobInfo>>, HashMap<i32, u32>) {
    let user_set: HashSet<i32> = users.iter().map(|user| user.id).collect();
    let problem_set: HashSet<i32> = problem_ids.iter().cloned().collect();

    let mut submission_for_rank = HashMap::new();
    let mut submission_count = HashMap::new();

    // The chosen submission is greater
    let scoring_rule_cmp = |lhs: &JobInfo, rhs: &JobInfo| match scoring_rule {
        ScoringRule::Latest => lhs.id.cmp(&rhs.id),
        ScoringRule::Highest => eps_cmp(lhs.score, rhs.score).then(rhs.id.cmp(&lhs.id)),
    };

    let is_accepted_dynamic_ranking = |job: &JobInfo| {
        if job.result != JobResult::Accepted {
            return false;
        }
        let problem = problem_map.get(&job.problem_id).unwrap();
        match problem.tp {
            ProblemType::DynamicRanking { .. } => true,
            _ => false,
        }
    };

    for job in jobs {
        let user_id = job.user_id;
        let problem_id = job.problem_id;
        if !user_set.contains(&user_id) || !problem_set.contains(&problem_id) {
            continue;
        }
        *submission_count.entry(user_id).or_insert(0) += 1;
        match submission_for_rank
            .entry(user_id)
            .or_insert(HashMap::new())
            .entry(problem_id)
        {
            Entry::Vacant(entry) => {
                entry.insert(job);
            }
            Entry::Occupied(mut entry) => {
                let entry_accepted_dynamic_ranking = is_accepted_dynamic_ranking(entry.get());
                let new_accepted_dynamic_ranking = is_accepted_dynamic_ranking(&job);
                if entry_accepted_dynamic_ranking && new_accepted_dynamic_ranking {
                    if job.id > entry.get().id {
                        entry.insert(job);
                    }
                } else if new_accepted_dynamic_ranking {
                    entry.insert(job);
                } else if !entry_accepted_dynamic_ranking {
                    if scoring_rule_cmp(&job, entry.get()).is_gt() {
                        entry.insert(job);
                    }
                }
            }
        }
    }

    (submission_for_rank, submission_count)
}

/// Update scores of the given problem if it's of dynamic ranking type.
async fn update_scores_for_dynamic_ranking(
    submission_for_rank: &mut HashMap<i32, HashMap<i32, JobInfo>>,
    problem: &Problem,
    pool: web::Data<ConnectionPool>,
) -> ApiResult<()> {
    if let ProblemType::DynamicRanking { ratio } = problem.tp {
        let mut min_time = vec![i64::MAX; problem.cases.len()];
        let mut job_time_map = HashMap::<i32, Vec<i64>>::new();

        // get min time for each test case
        for map in submission_for_rank.values() {
            if let Some(job) = map.get(&problem.id) {
                if job.result == JobResult::Accepted {
                    let job_id = job.id;
                    let mut conn = pool.get()?;
                    let job_time =
                        web::block(move || case_results::get_cases_time(job_id, &mut conn))
                            .await??;
                    if job_time.len() != min_time.len() {
                        return Err(ApiError::new(
                            ApiErrorType::Internal,
                            format!(
                                "Problem {} has {} cases, but {} cases found for job {}.",
                                problem.id,
                                min_time.len(),
                                job_time.len(),
                                job.id
                            ),
                        ));
                    }
                    for i in 0..min_time.len() {
                        min_time[i] = min_time[i].min(job_time[i]);
                    }
                    job_time_map.insert(job.id, job_time);
                }
            }
        }

        // update scores
        for map in submission_for_rank.values_mut() {
            if let Some(job) = map.get_mut(&problem.id) {
                if job.result == JobResult::Accepted {
                    let job_time = job_time_map.get(&job.id).unwrap();
                    for i in 0..min_time.len() {
                        job.score += min_time[i] as f64 / job_time[i] as f64
                            * problem.cases[i].score
                            * ratio;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Get an unsorted ranklist with each item having `rank: 1`.
fn get_unsorted_ranklist(
    submission_for_rank: &HashMap<i32, HashMap<i32, JobInfo>>,
    submission_count: &HashMap<i32, u32>,
    users: Vec<User>,
    problem_ids: &[i32],
) -> Vec<RankItem> {
    users
        .into_iter()
        .map(|user| {
            let mut scores = Vec::new();
            let mut total_score = 0.0;
            let mut last_job_id = 0;
            match submission_for_rank.get(&user.id) {
                None => {
                    scores = vec![0.0; problem_ids.len()];
                    last_job_id = i32::MAX;
                }
                Some(used_submission) => {
                    for problem_id in problem_ids {
                        match used_submission.get(problem_id) {
                            None => scores.push(0.0),
                            Some(job) => {
                                scores.push(job.score);
                                total_score += job.score;
                                last_job_id = last_job_id.max(job.id);
                            }
                        }
                    }
                }
            }
            RankItem {
                submission_count: *submission_count.get(&user.id).unwrap_or(&0),
                user,
                rank: 1,
                scores,
                total_score,
                last_job_id,
            }
        })
        .collect()
}

/// Sort the ranklist and calculate the `rank` field of each item.
fn sort_ranklist(rank_list: &mut Vec<RankItem>, tie_breaker: &Option<TieBreaker>) {
    let rank_cmp = |lhs: &RankItem, rhs: &RankItem| {
        eps_cmp(rhs.total_score, lhs.total_score).then(match tie_breaker {
            None => Ordering::Equal,
            Some(TieBreaker::SubmissionTime) => lhs.last_job_id.cmp(&rhs.last_job_id),
            Some(TieBreaker::SubmissionCount) => lhs.submission_count.cmp(&rhs.submission_count),
            Some(TieBreaker::UserId) => lhs.user.id.cmp(&rhs.user.id),
        })
    };
    // This sort should be stable to sort by `user_id` on tie.
    rank_list.sort_by(rank_cmp);
    // Calculate rank in case of ties.
    for index in 1..rank_list.len() {
        rank_list[index].rank = if rank_cmp(&rank_list[index - 1], &rank_list[index]).is_eq() {
            rank_list[index - 1].rank
        } else {
            index + 1
        }
    }
}
