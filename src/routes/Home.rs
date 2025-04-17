use crate::AppState;
use crate::entities::{prelude::*, *};
use crate::utils::{get_redis_id, get_redis_set_options};
use actix_web::{
    Error, delete,
    error::{ErrorBadRequest, ErrorInternalServerError},
    get, patch, post,
    web::{Data, Json, Path, ServiceConfig},
};
use nanoid::nanoid;
use redis::AsyncCommands;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;

const PREFIX: &str = "Home";

#[derive(Deserialize)]
struct HomeCU {
    pub name: String,
}

#[derive(Deserialize)]
struct HomeParams {
    pub id: String,
}

#[derive(Deserialize)]
struct RHomeApplicationParams {
    home_id: String,
}

#[post("")]
async fn create_home(
    state: Data<AppState>,
    body: Json<HomeCU>,
) -> Result<Json<home::Model>, Error> {
    let new_home = home::ActiveModel {
        id: sea_orm::ActiveValue::Set(nanoid!(10)),
        name: sea_orm::ActiveValue::Set(body.name.clone()),
        ..Default::default()
    };

    if let Ok(entity) = new_home.insert(&state.db).await {
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
    } else {
        Err(ErrorInternalServerError("Failed to create home"))
    }
}

#[get("")]
async fn get_homes(state: Data<AppState>) -> Result<Json<Vec<home::Model>>, Error> {
    match Home::find().all(&state.db).await {
        Ok(homes) => Ok(Json(homes)),
        Err(e) => {
            eprintln!("Error fetching homes: {:?}", e);
            Err(ErrorInternalServerError("Failed to fetch homes"))
        }
    }
}

#[get("/{id}")]
async fn get_home(
    state: Data<AppState>,
    params: Path<HomeParams>,
) -> Result<Json<home::Model>, Error> {
    let HomeParams { id } = params.into_inner();

    let mut redis_conn = state
        .redis
        .get_multiplexed_tokio_connection()
        .await
        .unwrap();
    if let Ok(cached_home) = redis_conn.get::<_, String>(get_redis_id(PREFIX, &id)).await {
        if let Ok(home) = serde_json::from_str::<home::Model>(&cached_home) {
            return Ok(Json(home));
        }
    }

    match Home::find_by_id(&id).one(&state.db).await {
        Ok(Some(home)) => {
            let _: () = redis_conn
                .set_options(
                    get_redis_id(PREFIX, &id),
                    serde_json::to_string(&home).unwrap(),
                    get_redis_set_options(),
                )
                .await
                .unwrap();
            Ok(Json(home))
        }
        Ok(None) => Err(ErrorBadRequest("Home not found")),
        Err(e) => {
            eprintln!("Error fetching home: {:?}", e);
            Err(ErrorInternalServerError("Error fetching home"))
        }
    }
}

#[get("/{home_id}/applications")]
async fn get_home_application(
    state: Data<AppState>,
    params: Path<RHomeApplicationParams>,
) -> Result<Json<Vec<application::Model>>, Error> {
    let RHomeApplicationParams { home_id } = params.into_inner();
    if let Ok(home_application) = Home::find()
        .find_with_related(Application)
        .filter(home::Column::Id.eq(home_id))
        .all(&state.db)
        .await
    {
        if let Some(home) = home_application.first() {
            return Ok(Json(home.1.clone()));
        }
        Err(ErrorBadRequest("Home not found"))
    } else {
        Err(ErrorInternalServerError("Query failed"))
    }
}

#[patch("/{id}")]
async fn update_home(
    state: Data<AppState>,
    params: Path<HomeParams>,
    body: Json<HomeCU>,
) -> Result<Json<home::Model>, Error> {
    let HomeParams { id } = params.into_inner();
    let HomeCU { name } = body.into_inner();

    match Home::find_by_id(&id).one(&state.db).await {
        Ok(Some(home)) => {
            let mut home: home::ActiveModel = home.into();
            home.name = sea_orm::ActiveValue::Set(name.to_owned());
            match home.update(&state.db).await {
                Ok(updated_home) => {
                    let mut redis_conn = state
                        .redis
                        .get_multiplexed_tokio_connection()
                        .await
                        .unwrap();
                    let _: () = redis_conn
                        .set_options(
                            get_redis_id(PREFIX, &id),
                            serde_json::to_string(&updated_home).unwrap(),
                            get_redis_set_options(),
                        )
                        .await
                        .unwrap();
                    Ok(Json(updated_home))
                }
                Err(e) => {
                    eprintln!("Error updating home: {:?}", e);
                    Err(ErrorInternalServerError("Failed to update home"))
                }
            }
        }
        Ok(None) => return Err(ErrorBadRequest("Home not found")),
        Err(e) => {
            eprintln!("Error fetching home: {:?}", e);
            Err(ErrorInternalServerError("Failed to fetch home"))
        }
    }
}

#[delete("/{id}")]
async fn delete_home(
    state: Data<AppState>,
    params: Path<HomeParams>,
) -> Result<&'static str, Error> {
    let HomeParams { id } = params.into_inner();

    match Home::delete_by_id(&id).exec(&state.db).await {
        Ok(_) => {
            let mut redis_conn = state
                .redis
                .get_multiplexed_tokio_connection()
                .await
                .unwrap();
            let _: () = redis_conn.del(get_redis_id(PREFIX, &id)).await.unwrap();
            Ok("Home deleted successfully")
        }
        Err(e) => {
            eprintln!("Error deleting home: {:?}", e);
            Err(ErrorInternalServerError("Failed to delete home"))
        }
    }
}

pub fn add_home_route(cfg: &mut ServiceConfig) {
    cfg.service(create_home)
        .service(get_home)
        .service(get_homes)
        .service(get_home_application)
        .service(update_home)
        .service(delete_home);
}
