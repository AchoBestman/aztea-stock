use sea_orm::entity::prelude::*;
use rust_decimal::Decimal;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "subscriptions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: String,
    pub tenant_id: String,
    pub plan: String,
    pub status: String,
    pub price_monthly: Decimal,
    pub currency: Option<String>,
    pub started_at: DateTimeWithTimeZone,
    pub expires_at: DateTimeWithTimeZone,
    pub trial_ends_at: Option<DateTimeWithTimeZone>,
    pub cancelled_at: Option<DateTimeWithTimeZone>,
    pub payment_method: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTimeWithTimeZone,
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
    #[sea_orm(has_many = "super::license::Entity")]
    License,
}

impl Related<super::tenant::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Tenant.def()
    }
}

impl Related<super::license::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::License.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
