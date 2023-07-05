//! `/users` API routes.

use crate::db::connection::ConnectionPool;
use crate::db::users;
use crate::error::ApiResult;
use actix_web::{get, post, web, Responder, Scope};
use serde::Deserialize;

#[derive(Deserialize)]
struct PostData {
    id: Option<i32>,
    name: String,
}

#[post("")]
async fn post_users(
    data: web::Json<PostData>,
    pool: web::Data<ConnectionPool>,
) -> ApiResult<impl Responder> {
    let user = match data.id {
        Some(id) => {
            web::block(move || users::set_name(id, data.into_inner().name, &pool)).await??
        }
        None => web::block(move || users::add_user(data.into_inner().name, &pool)).await??,
    };
    Ok(web::Json(user))
}

#[get("")]
async fn get_users(pool: web::Data<ConnectionPool>) -> ApiResult<impl Responder> {
    Ok(web::Json(
        web::block(move || users::get_users(&pool)).await??,
    ))
}

#[get("/{id}")]
async fn get_single_user(
    id: web::Path<i32>,
    pool: web::Data<ConnectionPool>,
) -> ApiResult<impl Responder> {
    Ok(web::Json(
        web::block(move || users::get_single_user(id.into_inner(), &pool)).await??,
    ))
}

pub fn routes() -> Scope {
    web::scope("/users")
        .service(post_users)
        .service(get_users)
        .service(get_single_user)
}
