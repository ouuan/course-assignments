//! `/problems` API routes.

use crate::config::{Problem, ProblemMap, ProblemType};
use crate::error::*;
use actix_web::{get, web, Responder, Scope};
use serde::Serialize;

/// Problem type without type-specific data.
#[derive(Serialize)]
enum ProblemTypeResponse {
    Standard,
    Strict,
    Spj,
    DynamicRanking,
}

#[derive(Serialize)]
struct ProblemResponse {
    id: i32,
    name: String,
    problem_type: ProblemTypeResponse,
}

impl ProblemResponse {
    fn new(problem: &Problem) -> Self {
        Self {
            id: problem.id,
            name: problem.name.clone(),
            problem_type: match problem.tp {
                ProblemType::Standard => ProblemTypeResponse::Standard,
                ProblemType::Strict => ProblemTypeResponse::Strict,
                ProblemType::Spj { .. } => ProblemTypeResponse::Spj,
                ProblemType::DynamicRanking { .. } => ProblemTypeResponse::DynamicRanking,
            },
        }
    }
}

#[get("")]
async fn get_all_problems(problem_map: web::Data<ProblemMap>) -> impl Responder {
    let mut problems = problem_map
        .values()
        .map(ProblemResponse::new)
        .collect::<Vec<_>>();
    problems.sort_by_key(|problem| problem.id);
    web::Json(problems)
}

#[get("/{id}")]
async fn get_problem(
    id: web::Path<i32>,
    problem_map: web::Data<ProblemMap>,
) -> ApiResult<impl Responder> {
    let problem = problem_map
        .get(&id)
        .ok_or_else(|| ApiError::not_found(&format!("Problem {}", id)))?;
    Ok(web::Json(ProblemResponse::new(problem)))
}

pub fn routes() -> Scope {
    web::scope("/problems")
        .service(get_all_problems)
        .service(get_problem)
}
