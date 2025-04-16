//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.9

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "application")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub home_id: String,
    #[sea_orm(column_type = "Text")]
    pub name: String,
    pub created_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::home::Entity",
        from = "Column::HomeId",
        to = "super::home::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Home,
    #[sea_orm(has_many = "super::sensor::Entity")]
    Sensor,
}

impl Related<super::home::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Home.def()
    }
}

impl Related<super::sensor::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sensor.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
