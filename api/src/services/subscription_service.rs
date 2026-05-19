use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, QueryFilter, ColumnTrait, Set};
use crate::{
    errors::ApiError,
    models::subscription,
    dtos::subscription_dto::{CreateSubscriptionPayload, SubscriptionResponse},
};

pub struct SubscriptionService;

impl SubscriptionService {
    pub fn map_to_response(model: subscription::Model) -> SubscriptionResponse {
        SubscriptionResponse {
            id: model.id,
            tenant_id: model.tenant_id,
            plan: model.plan,
            status: model.status,
            price_monthly: model.price_monthly,
            currency: model.currency.unwrap_or_else(|| "XAF".to_string()),
            started_at: model.started_at.to_rfc3339(),
            expires_at: model.expires_at.to_rfc3339(),
            trial_ends_at: model.trial_ends_at.map(|d| d.to_rfc3339()),
            cancelled_at: model.cancelled_at.map(|d| d.to_rfc3339()),
            notes: model.notes,
            created_at: model.created_at.to_rfc3339(),
        }
    }

    pub async fn create_subscription(
        db: &DatabaseConnection,
        payload: CreateSubscriptionPayload,
    ) -> Result<SubscriptionResponse, ApiError> {
        let id = uuid::Uuid::new_v4().to_string();
        
        let expires_at = chrono::DateTime::parse_from_rfc3339(&payload.expires_at)
            .map_err(|_| ApiError::BadRequest("Format de date invalide pour expires_at".to_string()))?;
        
        let trial_ends_at = if let Some(d) = payload.trial_ends_at {
            Some(chrono::DateTime::parse_from_rfc3339(&d)
                .map_err(|_| ApiError::BadRequest("Format de date invalide pour trial_ends_at".to_string()))?)
        } else {
            None
        };

        let sub = subscription::ActiveModel {
            id: Set(id),
            tenant_id: Set(payload.tenant_id),
            plan: Set(payload.plan),
            status: Set(payload.status),
            price_monthly: Set(payload.price_monthly),
            currency: Set(payload.currency),
            started_at: Set(chrono::Utc::now().fixed_offset()),
            expires_at: Set(expires_at),
            trial_ends_at: Set(trial_ends_at),
            notes: Set(payload.notes),
            ..Default::default()
        };

        let model = sub.insert(db).await.map_err(|e| {
            ApiError::Database(sea_orm::DbErr::Custom(format!("Erreur création abonnement: {}", e)))
        })?;

        Ok(Self::map_to_response(model))
    }

    pub async fn list_tenant_subscriptions(
        db: &DatabaseConnection,
        tenant_id: &str,
    ) -> Result<Vec<SubscriptionResponse>, ApiError> {
        let models = subscription::Entity::find()
            .filter(subscription::Column::TenantId.eq(tenant_id))
            .all(db)
            .await?;
        
        Ok(models.into_iter().map(Self::map_to_response).collect())
    }

    pub async fn get_subscription(
        db: &DatabaseConnection,
        subscription_id: &str,
    ) -> Result<SubscriptionResponse, ApiError> {
        let model = subscription::Entity::find_by_id(subscription_id)
            .one(db)
            .await?
            .ok_or_else(|| ApiError::NotFound("Abonnement introuvable".to_string()))?;
        
        Ok(Self::map_to_response(model))
    }
}
