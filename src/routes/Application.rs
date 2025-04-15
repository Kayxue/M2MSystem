use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use serde::Deserialize;
use xitca_web::{
    codegen::route,
    error::Error,
    handler::{json::Json, params::LazyParams, state::StateRef},
};

use crate::AppState;
use crate::CustomError::*;
use crate::entities::{prelude::*, *};

#[derive(Deserialize)]
struct GetHomeApplicationParams<'a> {
    id: &'a str,
}

#[route("/application/getHomeApplication/:id", method = get)]
pub async fn getHomeApplication(
    state: StateRef<'_, AppState>,
    params: LazyParams<'_, GetHomeApplicationParams<'_>>,
) -> Result<Json<Vec<application::Model>>, Error> {
    let GetHomeApplicationParams { id } = params.deserialize()?;
    if let Ok(home_application) = Home::find()
        .find_with_related(Application)
        .filter(home::Column::Id.eq(id))
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
