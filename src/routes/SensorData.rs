use actix_web::{
    Error, delete,
    error::{ErrorBadRequest, ErrorInternalServerError},
    get, post,
    web::{Data, Json, Path, ServiceConfig},
};
use nanoid::nanoid;
use redis::AsyncCommands;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, SqlErr};
use serde::Deserialize;
use serde_json::Value;

use crate::{
    AppState,
    entities::{prelude::*, *},
    utils::{get_redis_id, get_redis_set_options},
};

const PREFIX: &str = "SensorData";

#[derive(Deserialize)]
struct SensorDataCreate {
    pub container_id: String,
    pub data: Value,
}

#[derive(Deserialize)]
struct RDSensorDataParams {
    id: String,
}

#[post("")]
async fn create_sensor_data(
    state: Data<AppState>,
    body: Json<SensorDataCreate>,
) -> Result<Json<sensor_data::Model>, Error> {
    let SensorDataCreate { container_id, data } = body.into_inner();

    let new_sensor_data = sensor_data::ActiveModel {
        id: sea_orm::ActiveValue::Set(nanoid!(21)),
        container_id: sea_orm::ActiveValue::Set(container_id.to_owned()),
        data: sea_orm::ActiveValue::Set(Some(data)),
        ..Default::default()
    };

    match new_sensor_data.insert(&state.db).await {
        //TODO: Send new data to subscribers
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
            let subscriberList = Subscribers::find()
                .filter(subscribers::Column::ContainerId.eq(&entity.container_id))
                .all(&state.db)
                .await
                .unwrap();
            for subscriber in subscriberList {
                // Send new data to subscriber
            }
            Ok(Json(entity))
        }
        Err(e) => match e.sql_err() {
            Some(SqlErr::ForeignKeyConstraintViolation(_)) => {
                Err(ErrorBadRequest("Can't find data container"))
            }
            _ => {
                eprintln!("Error creating sensor data: {:?}", e);
                Err(ErrorInternalServerError("Query failed"))
            }
        },
    }
}

#[get("/{id}")]
async fn get_sensor_data(
    state: Data<AppState>,
    params: Path<RDSensorDataParams>,
) -> Result<Json<sensor_data::Model>, Error> {
    let RDSensorDataParams { id } = params.into_inner();

    let mut redis_conn = state
        .redis
        .get_multiplexed_tokio_connection()
        .await
        .unwrap();

    if let Ok(cached_data) = redis_conn.get::<_, String>(get_redis_id(PREFIX, &id)).await {
        if let Ok(entity) = serde_json::from_str::<sensor_data::Model>(&cached_data) {
            return Ok(Json(entity));
        }
    }

    match SensorData::find_by_id(&id).one(&state.db).await {
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
        Ok(None) => Err(ErrorBadRequest("Sensor data not found")),
        Err(e) => {
            eprintln!("Error fetching sensor data: {:?}", e);
            Err(ErrorInternalServerError("Query failed"))
        }
    }
}

#[delete("/{id}")]
async fn delete_sensor_data(
    state: Data<AppState>,
    params: Path<RDSensorDataParams>,
) -> Result<&'static str, Error> {
    let RDSensorDataParams { id } = params.into_inner();

    match SensorData::delete_by_id(&id).exec(&state.db).await {
        Ok(_) => {
            let mut redis_conn = state
                .redis
                .get_multiplexed_tokio_connection()
                .await
                .unwrap();
            let _: () = redis_conn.del(get_redis_id(PREFIX, &id)).await.unwrap();
            Ok("Sensor data deleted successfully")
        }
        Err(e) => {
            eprintln!("Error deleting sensor data: {:?}", e);
            Err(ErrorInternalServerError("Query failed"))
        }
    }
}

pub fn add_sensor_data_route(cfg: &mut ServiceConfig) {
    cfg.service(create_sensor_data)
        .service(get_sensor_data)
        .service(delete_sensor_data);
}
