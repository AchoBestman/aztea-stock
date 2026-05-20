use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "sales")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub tenant_id: String,
    pub user_id: Option<String>,
    pub receipt_number: String,
    pub customer_name: Option<String>,
    pub customer_phone: Option<String>,
    pub subtotal: f64,
    pub tax_total: f64,
    pub discount_total: f64,
    pub total: f64,
    pub amount_paid: f64,
    pub change_given: f64,
    pub payment_method: String,
    pub status: String,
    pub notes: Option<String>,
    pub sold_at: String,
    pub created_at: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::tenant::Entity",
        from = "Column::TenantId",
        to = "super::tenant::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Tenant,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    User,
    #[sea_orm(has_many = "super::sale_items::Entity")]
    SaleItems,
}

impl Related<super::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl Related<super::sale_items::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::SaleItems.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
