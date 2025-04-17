use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};
use actix_web::web::{Data, Json, Path, ServiceConfig};
use actix_web::{Error, delete, get, patch, post};
use nanoid::nanoid;
use redis::AsyncCommands;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, SqlErr};
use serde::Deserialize;

use crate::{AppState, utils::get_redis_id};
use crate::{
    entities::{prelude::*, *},
    utils::get_redis_set_options,
};

const PREFIX: &str = "Application";

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
    state: Data<AppState>,
    body: Json<ApplicationCreate>,
) -> Result<Json<application::Model>, Error> {
    let ApplicationCreate { name, home_id } = body.into_inner();

    let new_application = application::ActiveModel {
        id: sea_orm::ActiveValue::Set(nanoid!(10)),
        name: sea_orm::ActiveValue::Set(name.to_owned()),
        home_id: sea_orm::ActiveValue::Set(home_id.to_owned()),
        ..Default::default()
    };

    match new_application.insert(&state.db).await {
        Ok(entity) => {
            let mut conn = state
                .redis
                .get_multiplexed_tokio_connection()
                .await
                .unwrap();
            let _: () = conn
                .set_options(
                    get_redis_id(PREFIX, &entity.id),
                    serde_json::to_string(&entity).unwrap(),
                    get_redis_set_options(),
                )
                .await
                .unwrap();
            Ok(Json(entity))
        }
        Err(e) => match e.sql_err() {
            Some(SqlErr::ForeignKeyConstraintViolation(_)) => {
                Err(ErrorBadRequest("Can't find home"))
            }
            _ => {
                eprintln!("Error creating application: {:?}", e);
                Err(ErrorInternalServerError("Query failed"))
            }
        },
    }
}

#[get("/{id}")]
async fn get_application(
    state: Data<AppState>,
    params: Path<RUDApplicationParams>,
) -> Result<Json<application::Model>, Error> {
    let RUDApplicationParams { id } = params.into_inner();

    let mut conn = state
        .redis
        .get_multiplexed_tokio_connection()
        .await
        .unwrap();
    if let Ok(cached) = conn.get::<_, String>(get_redis_id(PREFIX, &id)).await {
        if let Ok(app) = serde_json::from_str::<application::Model>(&cached) {
            return Ok(Json(app));
        }
    }

    match Application::find_by_id(id).one(&state.db).await {
        Ok(Some(app)) => {
            let _: () = conn
                .set_options(
                    get_redis_id(PREFIX, &app.id),
                    serde_json::to_string(&app).unwrap(),
                    get_redis_set_options(),
                )
                .await
                .unwrap();
            Ok(Json(app))
        }
        Ok(None) => Err(ErrorBadRequest("Application not found")),
        Err(e) => {
            eprintln!("Error fetching application: {:?}", e);
            Err(ErrorInternalServerError("Failed to fetch application"))
        }
    }
}

#[get("/{id}/sensors")]
async fn get_application_sensors(
    state: Data<AppState>,
    params: Path<RUDApplicationParams>,
) -> Result<Json<Vec<sensor::Model>>, Error> {
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
    state: Data<AppState>,
    params: Path<RUDApplicationParams>,
    body: Json<ApplicationUpdate>,
) -> Result<Json<application::Model>, Error> {
    let RUDApplicationParams { id } = params.into_inner();
    let ApplicationUpdate { name } = body.into_inner();

    match Application::find_by_id(id).one(&state.db).await {
        Ok(Some(app)) => {
            let mut application: application::ActiveModel = app.into();
            application.name = sea_orm::ActiveValue::Set(name.to_owned());
            if let Ok(entity) = application.update(&state.db).await {
                let mut conn = state
                    .redis
                    .get_multiplexed_tokio_connection()
                    .await
                    .unwrap();
                let _: () = conn
                    .set_options(
                        get_redis_id(PREFIX, &entity.id),
                        serde_json::to_string(&entity).unwrap(),
                        get_redis_set_options(),
                    )
                    .await
                    .unwrap();
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
    state: Data<AppState>,
    params: Path<RUDApplicationParams>,
) -> Result<&'static str, Error> {
    let RUDApplicationParams { id } = params.into_inner();

    match Application::delete_by_id(&id).exec(&state.db).await {
        Ok(_) => {
            let mut conn = state
                .redis
                .get_multiplexed_tokio_connection()
                .await
                .unwrap();
            let _: () = conn.del(get_redis_id(PREFIX, &id)).await.unwrap();
            Ok("Application deleted successfully")
        }
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
