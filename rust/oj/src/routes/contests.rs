//! `/contests` API routes.

use crate::config::ProblemMap;
use crate::db::connection::ConnectionPool;
use crate::db::contests::{self, ContestWithoutId};
use crate::error::*;
use actix_web::{get, post, web, Responder, Scope};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize, Deserialize, Clone)]
struct ContestWithOptionalId {
    id: Option<i32>,
    #[serde(flatten)]
    contest: ContestWithoutId,
}

#[post("")]
async fn post_contests(
    data: web::Json<ContestWithOptionalId>,
    pool: web::Data<ConnectionPool>,
    problem_map: web::Data<ProblemMap>,
) -> ApiResult<impl Responder> {
    let mut problem_id_set = HashSet::new();
    for id in &data.contest.problem_ids {
        if !problem_map.contains_key(id) {
            return Err(ApiError::not_found(&format!("Problem {}", id)));
        }
        if !problem_id_set.insert(id) {
            return Err(ApiError::new(
                ApiErrorType::InvalidArgument,
                format!("Duplicate problem {}.", id),
            ));
        }
    }

    let mut user_id_set = HashSet::new();
    for id in &data.contest.user_ids {
        if !user_id_set.insert(id) {
            return Err(ApiError::new(
                ApiErrorType::InvalidArgument,
                format!("Duplicate user {}.", id),
            ));
        }
    }

    match data.id {
        None => {
            let mut response = data.clone();
            let id = web::block(move || contests::add_contest(&data.contest, &pool)).await??;
            response.id = Some(id);
            Ok(web::Json(response))
        }
        Some(id) => {
            let response = data.clone();
            web::block(move || contests::update_contest(id, &data.contest, &pool)).await??;
            Ok(web::Json(response))
        }
    }
}

#[get("")]
async fn get_all_contests(pool: web::Data<ConnectionPool>) -> ApiResult<impl Responder> {
    Ok(web::Json(
        web::block(move || contests::get_all_contests(&pool)).await??,
    ))
}

#[get("/{id}")]
async fn get_contest(
    id: web::Path<i32>,
    pool: web::Data<ConnectionPool>,
) -> ApiResult<impl Responder> {
    Ok(web::Json(
        web::block(move || contests::get_contest(id.into_inner(), &pool)).await??,
    ))
}

mod ranklist;

pub fn routes() -> Scope {
    web::scope("/contests")
        .service(post_contests)
        .service(get_all_contests)
        .service(get_contest)
        .service(ranklist::ranklist)
}
