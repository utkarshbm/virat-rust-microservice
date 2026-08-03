use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false, column_type = "String(Some(36))")]
    pub uuid: String,

    #[sea_orm(column_name = "createdOn")]
    pub created_on: DateTime,

    #[sea_orm(column_name = "modifiedOn")]
    pub modified_on: DateTime,

    #[sea_orm(column_name = "createdBy")]
    pub created_by: String,

    #[sea_orm(column_name = "modifiedBy")]
    pub modified_by: String,

    #[sea_orm(unique)]
    pub pan: String,

    pub arn: Option<String>,
    pub euin: Option<String>,
    pub name: String,
    pub password: String,
    pub dob: Option<String>,
    pub email: Option<String>,
    pub mobile: Option<String>,

    #[sea_orm(column_name = "loginFlag")]
    pub login_flag: i8,

    #[sea_orm(column_name = "primaryFolio")]
    pub primary_folio: Option<String>,

    #[sea_orm(column_name = "isPasswordSet")]
    pub is_password_set: i8,

    #[sea_orm(column_name = "lastVisitedOn")]
    pub last_visited_on: Option<DateTime>,

    #[sea_orm(column_name = "nctUpdateDate")]
    pub nct_update_date: Option<DateTime>,

    pub mpin: Option<String>,
    pub preference: Option<String>,

    #[sea_orm(column_name = "nri_ack_flag")]
    pub nri_ack_flag: i8,

    #[sea_orm(column_name = "nri_ack_timestamp")]
    pub nri_ack_timestamp: Option<DateTime>,

    #[sea_orm(column_name = "roleUuid")]
    pub role_uuid: Option<String>,

    #[sea_orm(column_name = "gaUuid", unique)]
    pub ga_uuid: Option<String>,

    #[sea_orm(column_name = "sifPrimaryFolio")]
    pub sif_primary_folio: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}