use nanoid::nanoid;
use ntex::web::{
    ServiceConfig, WebResponseError, delete,
    error::{ErrorBadRequest, ErrorInternalServerError},
    get, patch, post,
    types::{Json, Path, State},
};
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

#[post("")]
async fn add_application(
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

#[get("/{id}")]
async fn get_application(
    state: State<AppState>,
    params: Path<RUDApplicationParams>,
) -> Result<Json<application::Model>, impl WebResponseError> {
    let RUDApplicationParams { id } = params.into_inner();

    match Application::find_by_id(id).one(&state.db).await {
        Ok(Some(app)) => Ok(Json(app)),
        Ok(None) => Err(ErrorBadRequest("Application not found")),
        Err(e) => {
            eprintln!("Error fetching application: {:?}", e);
            Err(ErrorInternalServerError("Application not found"))
        }
    }
}

#[get("/{id}/sensors")]
async fn get_application_sensors(
    state: State<AppState>,
    params: Path<RUDApplicationParams>,
) -> Result<Json<Vec<sensor::Model>>, impl WebResponseError> {
    let RUDApplicationParams { id } = params.into_inner();

    match Sensor::find()
        .filter(sensor::Column::ApplicationId.eq(id))
        .all(&state.db)
        .await
    {
        Ok(sensors) => Ok(Json(sensors)),
        Err(e) => {
            eprintln!("Error fetching sensors: {:?}", e);
            Err(ErrorInternalServerError("Failed to fetch sensors"))
        }
    }
}

#[patch("/{id}")]
async fn update_application(
    state: State<AppState>,
    params: Path<RUDApplicationParams>,
    body: Json<ApplicationUpdate>,
) -> Result<Json<application::Model>, impl WebResponseError> {
    let RUDApplicationParams { id } = params.into_inner();
    let ApplicationUpdate { name } = body.into_inner();

    match Application::find_by_id(id).one(&state.db).await {
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

#[delete("/{id}")]
async fn delete_application(
    state: State<AppState>,
    params: Path<RUDApplicationParams>,
) -> Result<&'static str, impl WebResponseError> {
    let RUDApplicationParams { id } = params.into_inner();

    match Application::delete_by_id(id).exec(&state.db).await {
        Ok(_) => Ok("Application deleted successfully"),
        Err(e) => {
            eprintln!("Error deleting application: {:?}", e);
            Err(ErrorInternalServerError("Failed to delete application"))
        }
    }
}

pub fn add_application_route(cfg: &mut ServiceConfig) {
    cfg.service(add_application)
        .service(get_application)
        .service(get_application_sensors)
        .service(update_application)
        .service(delete_application);
}
