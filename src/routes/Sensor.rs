use actix_web::{
    Error, delete,
    error::{ErrorBadRequest, ErrorInternalServerError},
    get, patch, post,
    web::{Data, Json, Path, ServiceConfig},
};
use nanoid::nanoid;
use redis::AsyncCommands;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, SqlErr};
use serde::Deserialize;

use crate::entities::{prelude::*, *};
use crate::{
    AppState,
    utils::{get_redis_id, get_redis_set_options},
};

const PREFIX: &str = "Sensor";

#[derive(Deserialize)]
struct SensorCreate {
    pub name: String,
    pub application_id: String,
}

#[derive(Deserialize)]
struct SensorUpdate {
    pub name: String,
}

#[derive(Deserialize)]
struct RUDSensorParams {
    id: String,
}

#[post("")]
async fn create_sensor(
    state: Data<AppState>,
    body: Json<SensorCreate>,
) -> Result<Json<sensor::Model>, Error> {
    let SensorCreate {
        name,
        application_id,
    } = body.into_inner();

    let new_sensor = sensor::ActiveModel {
        id: sea_orm::ActiveValue::Set(nanoid!(10)),
        name: sea_orm::ActiveValue::Set(name.to_owned()),
        application_id: sea_orm::ActiveValue::Set(application_id.to_owned()),
        ..Default::default()
    };

    match new_sensor.insert(&state.db).await {
        Ok(entity) => {
            let mut redis_conn = state
                .redis
                .get_multiplexed_tokio_connection()
                .await
                .unwrap();
            let _: () = redis_conn
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
                Err(ErrorBadRequest("Can't find application"))
            }
            _ => {
                eprintln!("Error creating sensor: {:?}", e);
                Err(ErrorInternalServerError("Query failed"))
            }
        },
    }
}

#[get("/{id}")]
async fn get_sensor(
    state: Data<AppState>,
    params: Path<RUDSensorParams>,
) -> Result<Json<sensor::Model>, Error> {
    let RUDSensorParams { id } = params.into_inner();

    let mut redis_conn = state
        .redis
        .get_multiplexed_tokio_connection()
        .await
        .unwrap();

    if let Ok(cached_sensor) = redis_conn.get::<_, String>(get_redis_id(PREFIX, &id)).await {
        if let Ok(sensor) = serde_json::from_str::<sensor::Model>(&cached_sensor) {
            return Ok(Json(sensor));
        }
    }

    match Sensor::find_by_id(&id).one(&state.db).await {
        Ok(Some(entity)) => {
            let _: () = redis_conn
                .set_options(
                    get_redis_id(PREFIX, &id),
                    serde_json::to_string(&entity).unwrap(),
                    get_redis_set_options(),
                )
                .await
                .unwrap();
            Ok(Json(entity))
        }
        Ok(None) => Err(ErrorBadRequest("Can't find sensor")),
        Err(e) => {
            eprintln!("Error fetching sensor: {:?}", e);
            Err(ErrorInternalServerError("Query failed"))
        }
    }
}

#[get("/{id}/data_container")]
async fn get_sensor_data_container(
    state: Data<AppState>,
    params: Path<RUDSensorParams>,
) -> Result<Json<Vec<data_container::Model>>, Error> {
    let RUDSensorParams { id } = params.into_inner();

    if let Ok(data_container) = Sensor::find()
        .find_with_related(DataContainer)
        .filter(sensor::Column::Id.eq(id))
        .all(&state.db)
        .await
    {
        if let Some(sensor) = data_container.first() {
            return Ok(Json(sensor.1.clone()));
        }
        Err(ErrorBadRequest("Can't find sensor"))
    } else {
        Err(ErrorBadRequest("Query Failed"))
    }
}

#[patch("/{id}")]
async fn update_sensor(
    state: Data<AppState>,
    params: Path<RUDSensorParams>,
    body: Json<SensorUpdate>,
) -> Result<Json<sensor::Model>, Error> {
    let RUDSensorParams { id } = params.into_inner();
    let SensorUpdate { name } = body.into_inner();

    match Sensor::find_by_id(&id).one(&state.db).await {
        Ok(Some(e)) => {
            let mut entity: sensor::ActiveModel = e.into();
            entity.name = sea_orm::ActiveValue::Set(name.to_owned());
            match entity.update(&state.db).await {
                Ok(updated_entity) => {
                    let mut redis_conn = state
                        .redis
                        .get_multiplexed_tokio_connection()
                        .await
                        .unwrap();
                    let _: () = redis_conn
                        .set_options(
                            get_redis_id(PREFIX, &id),
                            serde_json::to_string(&updated_entity).unwrap(),
                            get_redis_set_options(),
                        )
                        .await
                        .unwrap();
                    Ok(Json(updated_entity))
                }
                Err(e) => {
                    eprintln!("Error updating sensor: {:?}", e);
                    Err(ErrorInternalServerError("Failed to update sensor"))
                }
            }
        }
        Ok(None) => Err(ErrorBadRequest("Can't find sensor")),
        Err(e) => {
            eprintln!("Error fetching sensor: {:?}", e);
            Err(ErrorInternalServerError("Query failed"))
        }
    }
}

#[delete("/{id}")]
async fn delete_sensor(
    state: Data<AppState>,
    params: Path<RUDSensorParams>,
) -> Result<&'static str, Error> {
    let RUDSensorParams { id } = params.into_inner();

    match Sensor::delete_by_id(&id).exec(&state.db).await {
        Ok(_) => {
            let mut redis_conn = state
                .redis
                .get_multiplexed_tokio_connection()
                .await
                .unwrap();
            let _: () = redis_conn.del(get_redis_id(PREFIX, &id)).await.unwrap();
            Ok("Sensor deleted successfully")
        }
        Err(e) => {
            eprintln!("Error deleting sensor: {:?}", e);
            Err(ErrorInternalServerError("Failed to delete sensor"))
        }
    }
}

pub fn add_sensor_route(cfg: &mut ServiceConfig) {
    cfg.service(create_sensor)
        .service(get_sensor)
        .service(get_sensor_data_container)
        .service(update_sensor)
        .service(delete_sensor);
}
