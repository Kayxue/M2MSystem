use nanoid::nanoid;
use sea_orm::{DatabaseConnection, EntityTrait};
use xitca_web::codegen::route;
use xitca_web::error::Error;
use xitca_web::handler::json::{Json, LazyJson};
use xitca_web::handler::state::StateRef;

use crate::entities::{prelude::*, *};
use crate::{AppState};
use crate::CustomError::*;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct HomeCreate<'a> {
    pub name: &'a str,
}

#[route("/home", method = post)]
pub async fn createHome(
    state: StateRef<'_, AppState>,
    body: LazyJson<HomeCreate<'_>>,
) -> Result<Json<crate::entities::home::Model>, Error> {
    let HomeCreate { name } = body.deserialize()?;

    let newHome = home::ActiveModel {
        id: sea_orm::ActiveValue::Set(nanoid!()),
        name: sea_orm::ActiveValue::Set(name.to_owned()),
        ..Default::default()
    };

    if let Ok(entity) = Home::insert(newHome).exec_with_returning(&state.db).await {
        Ok(Json(entity))
    } else {
        Err(InternalServerError::new("Failed to create home").into())
    }
}
