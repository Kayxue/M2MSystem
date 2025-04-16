//! `SeaORM` Entity, @generated by sea-orm-codegen 1.1.9

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "data_container")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub sensor_id: String,
    pub create_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::sensor::Entity",
        from = "Column::SensorId",
        to = "super::sensor::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Sensor,
    #[sea_orm(has_many = "super::sensor_data::Entity")]
    SensorData,
    #[sea_orm(has_many = "super::subscribers::Entity")]
    Subscribers,
}

impl Related<super::sensor::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Sensor.def()
    }
}

impl Related<super::sensor_data::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SensorData.def()
    }
}

impl Related<super::subscribers::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Subscribers.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
