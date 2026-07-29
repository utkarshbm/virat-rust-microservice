use actix_web::{web, HttpResponse};
use crate::state::AppState;

pub async fn find_by_pan(state: web::Data<AppState>, path: web::Path<String>) -> HttpResponse {
    match state.user_repo.find_by_pan(&path).await {
        Ok(Some(user)) => HttpResponse::Ok().json(user),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}