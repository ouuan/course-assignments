//! `/jobs` API routes.

use crate::config::{LanguageMap, ProblemMap};
use crate::db::connection::ConnectionPool;
use crate::db::jobs::{self, JobFilter, Submission};
use crate::error::*;
use crate::judger::JobAdder;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder, Scope};

#[post("")]
async fn add_job(
    submission: web::Json<Submission>,
    problem_map: web::Data<ProblemMap>,
    language_map: web::Data<LanguageMap>,
    pool: web::Data<ConnectionPool>,
    adder: web::Data<JobAdder>,
) -> ApiResult<impl Responder> {
    if !language_map.contains_key(&submission.language) {
        return Err(ApiError::not_found(&format!(
            "Language {}",
            submission.language
        )));
    }
    let problem = match problem_map.get(&submission.problem_id) {
        None => {
            return Err(ApiError::not_found(&format!(
                "Problem {}",
                submission.problem_id
            )))
        }
        Some(problem) => problem,
    };
    let case_count = problem.cases.len();
    Ok(web::Json(
        web::block(move || jobs::add_job(submission.0, case_count, &pool, &adder)).await??,
    ))
}

#[get("")]
async fn get_jobs(
    filter: web::Query<JobFilter>,
    pool: web::Data<ConnectionPool>,
) -> ApiResult<impl Responder> {
    Ok(web::Json(
        web::block(move || jobs::get_jobs(&filter, &pool)).await??,
    ))
}

#[get("/{id}")]
async fn get_job(id: web::Path<i32>, pool: web::Data<ConnectionPool>) -> ApiResult<impl Responder> {
    Ok(web::Json(
        web::block(move || jobs::get_job(id.into_inner(), &pool)).await??,
    ))
}

#[put("/{id}")]
async fn rejudge(
    id: web::Path<i32>,
    adder: web::Data<JobAdder>,
    pool: web::Data<ConnectionPool>,
) -> ApiResult<impl Responder> {
    Ok(web::Json(
        web::block(move || jobs::rejudge(id.into_inner(), &adder, &pool)).await??,
    ))
}

#[delete("/{id}")]
async fn cancel_job(
    id: web::Path<i32>,
    pool: web::Data<ConnectionPool>,
) -> ApiResult<impl Responder> {
    web::block(move || jobs::cancel_job(id.into_inner(), &pool)).await??;
    Ok(HttpResponse::Ok())
}

pub fn routes() -> Scope {
    web::scope("/jobs")
        .service(add_job)
        .service(get_jobs)
        .service(get_job)
        .service(rejudge)
        .service(cancel_job)
}
