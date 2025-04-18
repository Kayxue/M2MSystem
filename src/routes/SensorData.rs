use actix_web::{
    Error, delete,
    error::{ErrorBadRequest, ErrorInternalServerError},
    get, patch, post,
    web::{Data, Json, Path, ServiceConfig},
};
use nanoid::nanoid;
use sea_orm::{ActiveModelTrait, EntityTrait, SqlErr};
use serde::Deserialize;
use serde_json::Value;

use crate::AppState;
use crate::entities::{prelude::*, *};

#[derive(Deserialize)]
struct SensorDataCreate {
    pub container_id: String,
    pub data: Value,
}

#[derive(Deserialize)]
struct SensorDataUpdate {
    pub data: Value,
}

#[derive(Deserialize)]
struct RDSensorDataParams {
    id: String,
    container_id: String,
}

#[post("")]
async fn create_sensor_data(
    state: Data<AppState>,
    body: Json<SensorDataCreate>,
) -> Result<Json<sensor_data::Model>, Error> {
    let SensorDataCreate { container_id, data } = body.into_inner();

    let new_sensor_data = sensor_data::ActiveModel {
        id: sea_orm::ActiveValue::Set(nanoid!(10)),
        container_id: sea_orm::ActiveValue::Set(container_id.to_owned()),
        data: sea_orm::ActiveValue::Set(Some(data)),
        ..Default::default()
    };

    match new_sensor_data.insert(&state.db).await {
        Ok(entity) => Ok(Json(entity)),
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

#[get("/{container_id}/{id}")]
async fn get_sensor_data(
    state: Data<AppState>,
    params: Path<RDSensorDataParams>,
) -> Result<Json<sensor_data::Model>, Error> {
    let RDSensorDataParams { id, container_id } = params.into_inner();

    match SensorData::find_by_id((id, container_id))
        .one(&state.db)
        .await
    {
        Ok(Some(entity)) => Ok(Json(entity)),
        Ok(None) => Err(ErrorBadRequest("Sensor data not found")),
        Err(e) => {
            eprintln!("Error fetching sensor data: {:?}", e);
            Err(ErrorInternalServerError("Query failed"))
        }
    }
}

#[delete("/{container_id}/{id}")]
async fn delete_sensor_data(
    state: Data<AppState>,
    params: Path<RDSensorDataParams>,
) -> Result<&'static str, Error> {
    let RDSensorDataParams { id, container_id } = params.into_inner();

    match SensorData::delete_by_id((id, container_id))
        .exec(&state.db)
        .await
    {
        Ok(_) => Ok("Sensor data deleted successfully"),
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
