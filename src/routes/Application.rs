use nanoid::nanoid;
use ntex::web::error::{ErrorBadRequest, ErrorInternalServerError};
use ntex::web::types::{Json, Path, State};
use ntex::web::{WebResponseError, delete, get, patch, post};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, SqlErr};
use serde::Deserialize;

use crate::AppState;
use crate::entities::{prelude::*, *};

#[derive(Deserialize)]
struct ApplicationCreate {
    pub name: String,
    pub home_id: String,
}

#[derive(Deserialize)]
struct ApplicationUpdate {
    pub name: String,
}

#[derive(Deserialize)]
struct RUDApplicationParams {
    id: String,
}

#[post("/application")]
pub async fn addApplication(
    state: State<AppState>,
    body: Json<ApplicationCreate>,
) -> Result<Json<application::Model>, impl WebResponseError> {
    let ApplicationCreate { name, home_id } = body.into_inner();

    let new_application = application::ActiveModel {
        id: sea_orm::ActiveValue::Set(nanoid!(10)),
        name: sea_orm::ActiveValue::Set(name.to_owned()),
        home_id: sea_orm::ActiveValue::Set(home_id.to_owned()),
        ..Default::default()
    };

    match new_application.insert(&state.db).await {
        Ok(entity) => Ok(Json(entity)),
        Err(e) => match e.sql_err() {
            Some(SqlErr::ForeignKeyConstraintViolation(_)) => Err(ErrorBadRequest("Query failed")),
            _ => {
                eprintln!("Error creating application: {:?}", e);
                Err(ErrorInternalServerError("Query failed"))
            }
        },
    }
}

#[get("/application/{id}")]
pub async fn getApplication(
    state: State<AppState>,
    params: Path<RUDApplicationParams>,
) -> Result<Json<application::Model>, impl WebResponseError> {
    let RUDApplicationParams { id } = params.into_inner();

    match application::Entity::find_by_id(id).one(&state.db).await {
        Ok(Some(app)) => Ok(Json(app)),
        Ok(None) => Err(ErrorBadRequest("Application not found")),
        Err(e) => {
            eprintln!("Error fetching application: {:?}", e);
            Err(ErrorInternalServerError("Application not found"))
        }
    }
}

#[patch("/application/{id}")]
pub async fn updateApplication(
    state: State<AppState>,
    params: Path<RUDApplicationParams>,
    body: Json<ApplicationUpdate>,
) -> Result<Json<application::Model>, impl WebResponseError> {
    let RUDApplicationParams { id } = params.into_inner();
    let ApplicationUpdate { name } = body.into_inner();

    match application::Entity::find_by_id(id).one(&state.db).await {
        Ok(Some(app)) => {
            let mut application: application::ActiveModel = app.into();
            application.name = sea_orm::ActiveValue::Set(name.to_owned());
            if let Ok(entity) = application.update(&state.db).await {
                Ok(Json(entity))
            } else {
                Err(ErrorInternalServerError("Failed to update application"))
            }
        }
        Ok(None) => Err(ErrorBadRequest("Failed to update application")),
        Err(e) => {
            eprintln!("Error fetching application: {:?}", e);
            Err(ErrorInternalServerError("Failed to update application"))
        }
    }
}

#[delete("/application/{id}")]
pub async fn deleteApplication(
    state: State<AppState>,
    params: Path<RUDApplicationParams>,
) -> Result<&'static str, impl WebResponseError> {
    let RUDApplicationParams { id } = params.into_inner();

    match Application::delete_by_id(id).exec(&state.db).await {
        Ok(_) => Ok("Application deleted successfully"),
        Err(e) => {
            eprintln!("Error fetching application: {:?}", e);
            Err(ErrorInternalServerError("Failed to fetch application"))
        }
    }
}
