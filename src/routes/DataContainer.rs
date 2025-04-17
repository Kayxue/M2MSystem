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

use crate::entities::{prelude::*, *};
use crate::{
    AppState,
    utils::{get_redis_id, get_redis_set_options},
};

const PREFIX: &str = "DataContainer";

#[derive(Deserialize)]
struct DataContainerCreate {
    pub sensor_id: String,
}

#[derive(Deserialize)]
struct RDDataContainerParams {
    id: String,
}

#[post("")]
async fn create_data_container(
    state: Data<AppState>,
    body: Json<DataContainerCreate>,
) -> Result<Json<data_container::Model>, Error> {
    let DataContainerCreate { sensor_id } = body.into_inner();

    let new_data_container = data_container::ActiveModel {
        id: sea_orm::ActiveValue::Set(nanoid!(10)),
        sensor_id: sea_orm::ActiveValue::Set(sensor_id.to_owned()),
        ..Default::default()
    };

    match new_data_container.insert(&state.db).await {
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
                Err(ErrorBadRequest("Can't find sensor"))
            }
            _ => {
                eprintln!("Error creating data container: {:?}", e);
                Err(ErrorInternalServerError("Query failed"))
            }
        },
    }
}

#[get("/{id}")]
async fn get_data_container(
    state: Data<AppState>,
    params: Path<RDDataContainerParams>,
) -> Result<Json<data_container::Model>, Error> {
    let RDDataContainerParams { id } = params.into_inner();
    let mut redis_conn = state
        .redis
        .get_multiplexed_tokio_connection()
        .await
        .unwrap();

    if let Ok(cached_data) = redis_conn.get::<_, String>(get_redis_id(PREFIX, &id)).await {
        if let Ok(entity) = serde_json::from_str::<data_container::Model>(&cached_data) {
            return Ok(Json(entity));
        }
    }

    match DataContainer::find_by_id(&id).one(&state.db).await {
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
        Ok(None) => Err(ErrorBadRequest("Data container not found")),
        Err(e) => {
            eprintln!("Error fetching data container: {:?}", e);
            Err(ErrorInternalServerError("Query failed"))
        }
    }
}

#[get("/{id}/sensor_data")]
async fn get_sensor_data(
    state: Data<AppState>,
    params: Path<RDDataContainerParams>,
) -> Result<Json<Vec<sensor_data::Model>>, Error> {
    let RDDataContainerParams { id } = params.into_inner();
    match SensorData::find()
        .filter(sensor_data::Column::ContainerId.eq(id))
        .all(&state.db)
        .await
    {
        Ok(entities) => Ok(Json(entities)),
        Err(e) => {
            eprintln!("Error fetching sensor data: {:?}", e);
            Err(ErrorInternalServerError("Query failed"))
        }
    }
}

#[get("/{id}/subscribers")]
async fn get_subscribers(
    state: Data<AppState>,
    params: Path<RDDataContainerParams>,
) -> Result<Json<Vec<subscribers::Model>>, Error> {
    let RDDataContainerParams { id } = params.into_inner();
    match Subscribers::find()
        .filter(subscribers::Column::ContainerId.eq(id))
        .all(&state.db)
        .await
    {
        Ok(entities) => Ok(Json(entities)),
        Err(e) => {
            eprintln!("Error fetching subscribers: {:?}", e);
            Err(ErrorInternalServerError("Query failed"))
        }
    }
}

#[delete("/{id}")]
async fn delete_data_container(
    state: Data<AppState>,
    params: Path<RDDataContainerParams>,
) -> Result<&'static str, Error> {
    let RDDataContainerParams { id } = params.into_inner();

    match DataContainer::delete_by_id(&id).exec(&state.db).await {
        Ok(_) => {
            let mut redis_conn = state
                .redis
                .get_multiplexed_tokio_connection()
                .await
                .unwrap();
            let _: () = redis_conn.del(get_redis_id(PREFIX, &id)).await.unwrap();
            Ok("Data container deleted")
        }
        Err(e) => {
            eprintln!("Error deleting data container: {:?}", e);
            Err(ErrorInternalServerError("Query failed"))
        }
    }
}

pub fn add_data_container_routes(cfg: &mut ServiceConfig) {
    cfg.service(create_data_container)
        .service(get_data_container)
        .service(get_sensor_data)
        .service(get_subscribers)
        .service(delete_data_container);
}
