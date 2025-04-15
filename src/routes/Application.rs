use nanoid::nanoid;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use xitca_web::{
    codegen::route,
    error::Error,
    handler::{
        json::{Json, LazyJson},
        params::LazyParams,
        state::StateRef,
    },
};

use crate::AppState;
use crate::CustomError::*;
use crate::entities::{prelude::*, *};

#[derive(Deserialize)]
struct ApplicationCreate<'a> {
    pub name: &'a str,
    pub home_id: &'a str,
}

#[derive(Deserialize)]
struct ApplicationUpdate<'a> {
    pub name: &'a str,
}

#[derive(Deserialize)]
struct RHomeApplicationParams<'a> {
    homeId: &'a str,
}

#[derive(Deserialize)]
struct RUDApplicationParams<'a> {
    id: &'a str,
}

#[route("/application/getHomeApplication/:homeId", method = get)]
pub async fn getHomeApplication(
    state: StateRef<'_, AppState>,
    params: LazyParams<'_, RHomeApplicationParams<'_>>,
) -> Result<Json<Vec<application::Model>>, Error> {
    let RHomeApplicationParams { homeId } = params.deserialize()?;
    if let Ok(home_application) = Home::find()
        .find_with_related(Application)
        .filter(home::Column::Id.eq(homeId))
        .all(&state.db)
        .await
    {
        if let Some(home) = home_application.first() {
            return Ok(Json(home.1.clone()));
        }
        Err(BadRequest::new("Home not found").into())
    } else {
        Err(InternalServerError::new("Query failed").into())
    }
}

#[route("/application/addApplication/", method = post)]
pub async fn addApplication(
    state: StateRef<'_, AppState>,
    body: LazyJson<ApplicationCreate<'_>>,
) -> Result<Json<application::Model>, Error> {
    let ApplicationCreate { name, home_id } = body.deserialize()?;

    let new_application = application::ActiveModel {
        id: sea_orm::ActiveValue::Set(nanoid!(10)),
        name: sea_orm::ActiveValue::Set(name.to_owned()),
        home_id: sea_orm::ActiveValue::Set(home_id.to_owned()),
        ..Default::default()
    };

    if let Ok(entity) = new_application.insert(&state.db).await {
        Ok(Json(entity))
    } else {
        Err(InternalServerError::new("Failed to create application").into())
    }
}

#[route("/application/:id", method = get)]
pub async fn getApplication(
    state: StateRef<'_, AppState>,
    params: LazyParams<'_, RUDApplicationParams<'_>>,
) -> Result<Json<application::Model>, Error> {
    let RUDApplicationParams { id } = params.deserialize()?;

    match application::Entity::find_by_id(id).one(&state.db).await {
        Ok(Some(app)) => Ok(Json(app)),
        Ok(None) => Err(BadRequest::new("Application not found").into()),
        Err(e) => {
            eprintln!("Error fetching application: {:?}", e);
            Err(InternalServerError::new("Failed to fetch application").into())
        }
    }
}

#[route("/application/updateApplication/:id", method = patch)]
pub async fn updateApplication(
    state: StateRef<'_, AppState>,
    params: LazyParams<'_, RUDApplicationParams<'_>>,
    body: LazyJson<ApplicationUpdate<'_>>,
) -> Result<Json<application::Model>, Error> {
    let RUDApplicationParams { id } = params.deserialize()?;
    let ApplicationUpdate { name } = body.deserialize()?;

    match application::Entity::find_by_id(id).one(&state.db).await {
        Ok(Some(app)) => {
            let mut application: application::ActiveModel = app.into();
            application.name = sea_orm::ActiveValue::Set(name.to_owned());
            if let Ok(entity) = application.update(&state.db).await {
                Ok(Json(entity))
            } else {
                Err(InternalServerError::new("Failed to update application").into())
            }
        }
        Ok(None) => Err(BadRequest::new("Application not found").into()),
        Err(e) => {
            eprintln!("Error fetching application: {:?}", e);
            Err(InternalServerError::new("Failed to fetch application").into())
        }
    }
}

#[route("/application/:id", method = delete)]
pub async fn deleteApplication(
    state: StateRef<'_, AppState>,
    params: LazyParams<'_, RUDApplicationParams<'_>>,
) -> Result<&'static str, Error> {
    let RUDApplicationParams { id } = params.deserialize()?;

    match Application::delete_by_id(id).exec(&state.db).await {
        Ok(_) => Ok("Application deleted successfully"),
        Err(e) => {
            eprintln!("Error fetching application: {:?}", e);
            Err(InternalServerError::new("Failed to fetch application").into())
        }
    }
}
