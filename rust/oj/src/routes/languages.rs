//! `/languages` API routes.

use crate::config::LanguageMap;
use actix_web::{get, web, Responder, Scope};

#[get("")]
async fn get_languages(language_map: web::Data<LanguageMap>) -> impl Responder {
    let mut languages = language_map.keys().cloned().collect::<Vec<_>>();
    languages.sort_unstable();
    web::Json(languages)
}

pub fn routes() -> Scope {
    web::scope("/languages").service(get_languages)
}
