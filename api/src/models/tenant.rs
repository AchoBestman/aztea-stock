use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "tenants")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub name: String,
    pub business_type: String,
    pub email: String,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub timezone: Option<String>,
    pub logo_url: Option<String>,
    pub is_active: Option<bool>,
    pub is_system: bool,
    pub two_factor_enabled: bool,
    /// Display/from email address for outgoing emails
    pub sender_email: Option<String>,
    /// SMTP login username — stored AES-256-CBC encrypted
    pub sender_user: Option<String>,
    /// SMTP password — stored AES-256-CBC encrypted
    pub sender_password: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::role::Entity")]
    Role,
    #[sea_orm(has_many = "super::user::Entity")]
    User,
}

impl Related<super::role::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Role.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
