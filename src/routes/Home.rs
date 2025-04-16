use nanoid::nanoid;
use ntex::web::{
    ServiceConfig, WebResponseError, delete,
    error::{ErrorBadRequest, ErrorInternalServerError},
    get, patch, post,
    types::{Json, Path, State},
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};

use crate::AppState;
use crate::entities::{prelude::*, *};

use serde::Deserialize;

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
    state: State<AppState>,
    body: Json<HomeCU>,
) -> Result<Json<home::Model>, impl WebResponseError> {
    let new_home = home::ActiveModel {
        id: sea_orm::ActiveValue::Set(nanoid!(10)),
        name: sea_orm::ActiveValue::Set(body.name.clone()),
        ..Default::default()
    };

    if let Ok(entity) = new_home.insert(&state.db).await {
        Ok(Json(entity))
    } else {
        Err(ErrorInternalServerError("Failed to create home"))
    }
}

#[get("")]
async fn get_homes(
    state: State<AppState>,
) -> Result<Json<Vec<home::Model>>, impl WebResponseError> {
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
    state: State<AppState>,
    params: Path<HomeParams>,
) -> Result<Json<home::Model>, impl WebResponseError> {
    let HomeParams { id } = params.into_inner();

    match Home::find_by_id(id).one(&state.db).await {
        Ok(Some(home)) => Ok(Json(home)),
        Ok(None) => Err(ErrorBadRequest("Home not found")),
        Err(e) => {
            eprintln!("Error fetching home: {:?}", e);
            Err(ErrorInternalServerError("Home not found"))
        }
    }
}

#[get("/{home_id}/applications")]
async fn get_home_application(
    state: State<AppState>,
    params: Path<RHomeApplicationParams>,
) -> Result<Json<Vec<application::Model>>, impl WebResponseError> {
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
    state: State<AppState>,
    params: Path<HomeParams>,
    body: Json<HomeCU>,
) -> Result<Json<home::Model>, impl WebResponseError> {
    let HomeParams { id } = params.into_inner();
    let HomeCU { name } = body.into_inner();

    match Home::find_by_id(id).one(&state.db).await {
        Ok(Some(home)) => {
            let mut home: home::ActiveModel = home.into();
            home.name = sea_orm::ActiveValue::Set(name.to_owned());
            match home.update(&state.db).await {
                Ok(updated_home) => Ok(Json(updated_home)),
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
    state: State<AppState>,
    params: Path<HomeParams>,
) -> Result<&'static str, impl WebResponseError> {
    let HomeParams { id } = params.into_inner();

    match Home::delete_by_id(id).exec(&state.db).await {
        Ok(_) => Ok("Home deleted successfully"),
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
