use crate::{
    dtos::subscription_dto::{
        CreateSubscriptionPayload, SubscriptionResponse, UpdateSubscriptionStatusPayload,
    },
    errors::ApiError,
    models::subscription,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, PaginatorTrait, QueryFilter,
    Set,
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
            max_devices: model.max_devices,
            started_at: model.started_at.to_rfc3339(),
            expires_at: model.expires_at.to_rfc3339(),
            trial_ends_at: model.trial_ends_at.map(|d| d.to_rfc3339()),
            cancelled_at: model.cancelled_at.map(|d| d.to_rfc3339()),
            notes: model.notes,
            created_at: model.created_at.to_rfc3339(),
        }
    }

    pub async fn create_subscription(
        state: &crate::AppState,
        payload: CreateSubscriptionPayload,
    ) -> Result<SubscriptionResponse, ApiError> {
        let db = state
            .db
            .as_ref()
            .ok_or_else(|| ApiError::Internal("Base de données indisponible".to_string()))?;
        let id = uuid::Uuid::new_v4().to_string();

        let expires_at =
            chrono::DateTime::parse_from_rfc3339(&payload.expires_at).map_err(|_| {
                ApiError::BadRequest("Format de date invalide pour expires_at".to_string())
            })?;

        let trial_ends_at = if let Some(d) = payload.trial_ends_at {
            Some(chrono::DateTime::parse_from_rfc3339(&d).map_err(|_| {
                ApiError::BadRequest("Format de date invalide pour trial_ends_at".to_string())
            })?)
        } else {
            None
        };

        let sub_status = payload.status.clone();
        let max_devices = payload.max_devices;
        let tenant_id = payload.tenant_id.clone();
        let sub_id = id.clone();

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
            max_devices: Set(payload.max_devices),
            notes: Set(payload.notes),
            ..Default::default()
        };

        let model = sub.insert(db).await.map_err(|e| {
            ApiError::Database(sea_orm::DbErr::Custom(format!(
                "Erreur création abonnement: {}",
                e
            )))
        })?;

        // Automatically generate initial licenses
        let license_status = if sub_status == "trial" {
            "trial"
        } else {
            "production"
        };
        let _ = crate::services::license_service::LicenseService::generate_batch_licenses(
            db,
            state,
            &tenant_id,
            &sub_id,
            max_devices,
            license_status,
        )
        .await?;

        Ok(Self::map_to_response(model))
    }

    pub async fn update_subscription(
        state: &crate::AppState,
        subscription_id: &str,
        payload: crate::dtos::subscription_dto::UpdateSubscriptionPayload,
        caller_user_id: &str,
        caller_tenant_id: &str,
    ) -> Result<SubscriptionResponse, ApiError> {
        let db = state
            .db
            .as_ref()
            .ok_or_else(|| ApiError::Internal("Base de données indisponible".to_string()))?;

        crate::utils::auth::assert_system_admin_access(db, caller_user_id, caller_tenant_id)
            .await?;

        let sub = subscription::Entity::find_by_id(subscription_id)
            .one(db)
            .await?
            .ok_or_else(|| ApiError::NotFound("Abonnement introuvable".to_string()))?;

        let old_max = sub.max_devices;
        let new_max = payload.max_devices;

        // If reducing limit, check if we have more licenses than the new limit
        if new_max < old_max {
            let total_licenses = crate::models::license::Entity::find()
                .filter(crate::models::license::Column::SubscriptionId.eq(subscription_id))
                .count(db)
                .await?;

            if total_licenses > new_max as u64 {
                return Err(ApiError::BadRequest(format!(
                    "Impossible de réduire le nombre d'appareils à {}. {} licences sont déjà générées pour cet abonnement. Veuillez révoquer ou supprimer certaines licences d'abord.",
                    new_max, total_licenses
                )));
            }
        }

        let expires_at =
            chrono::DateTime::parse_from_rfc3339(&payload.expires_at).map_err(|_| {
                ApiError::BadRequest("Format de date invalide pour expires_at".to_string())
            })?;

        let trial_ends_at = if let Some(d) = payload.trial_ends_at {
            Some(chrono::DateTime::parse_from_rfc3339(&d).map_err(|_| {
                ApiError::BadRequest("Format de date invalide pour trial_ends_at".to_string())
            })?)
        } else {
            None
        };

        let mut active: subscription::ActiveModel = sub.into();
        active.plan = Set(payload.plan);
        active.status = Set(payload.status.clone());
        active.price_monthly = Set(payload.price_monthly);
        active.currency = Set(payload.currency);
        active.expires_at = Set(expires_at);
        active.trial_ends_at = Set(trial_ends_at);
        active.max_devices = Set(new_max);
        active.notes = Set(payload.notes);

        let updated = active.update(db).await?;

        // If limit increased, generate and send the difference
        if new_max > old_max {
            let diff = new_max - old_max;
            let license_status = if payload.status == "trial" {
                "trial"
            } else {
                "production"
            };
            let _ = crate::services::license_service::LicenseService::generate_batch_licenses(
                db,
                state,
                &updated.tenant_id,
                &updated.id,
                diff,
                license_status,
            )
            .await?;
        }

        // Deactivate licenses if status changed to suspended or cancelled
        if payload.status == "suspended" || payload.status == "cancelled" {
            let lics = crate::models::license::Entity::find()
                .filter(crate::models::license::Column::SubscriptionId.eq(subscription_id))
                .filter(crate::models::license::Column::IsActive.eq(true))
                .all(db)
                .await?;

            for lic in lics {
                let mut active_lic: crate::models::license::ActiveModel = lic.into();
                active_lic.is_active = Set(Some(false));
                let _ = active_lic.update(db).await;
            }
        }

        Ok(Self::map_to_response(updated))
    }

    pub async fn list_subscriptions(
        db: &DatabaseConnection,
        params: crate::utils::pagination::PaginationParams,
        enforce_tenant_id: Option<String>,
    ) -> Result<crate::utils::pagination::PaginatedResponse<SubscriptionResponse>, ApiError> {
        let mut query = subscription::Entity::find();

        let target_tenant = enforce_tenant_id.or(params.tenant_id);
        if let Some(tenant_id) = target_tenant {
            query = query.filter(subscription::Column::TenantId.eq(tenant_id));
        }

        if let Some(search) = params.search {
            query = query.filter(
                sea_orm::Condition::any()
                    .add(subscription::Column::Plan.contains(&search))
                    .add(subscription::Column::Status.contains(&search)),
            );
        }

        if let Some(start_date) = params.start_date {
            if let Ok(date) = chrono::DateTime::parse_from_rfc3339(&start_date) {
                query = query.filter(subscription::Column::CreatedAt.gte(date));
            }
        }

        if let Some(end_date) = params.end_date {
            if let Ok(date) = chrono::DateTime::parse_from_rfc3339(&end_date) {
                query = query.filter(subscription::Column::CreatedAt.lte(date));
            }
        }

        let page = params.page.unwrap_or(1);
        let per_page = params.per_page.unwrap_or(20);
        let order_desc = params.order_type.as_deref().unwrap_or("desc") != "asc";

        use sea_orm::{PaginatorTrait, QueryOrder};

        let order_col = match params.order_by.as_deref().unwrap_or("created_at") {
            "plan" => subscription::Column::Plan,
            "status" => subscription::Column::Status,
            "expires_at" => subscription::Column::ExpiresAt,
            "started_at" => subscription::Column::StartedAt,
            _ => subscription::Column::CreatedAt,
        };

        query = if order_desc {
            query.order_by_desc(order_col)
        } else {
            query.order_by_asc(order_col)
        };

        let paginator = query.paginate(db, per_page);
        let total = paginator.num_items().await?;
        let total_pages = paginator.num_pages().await?;

        let models = paginator.fetch_page(page - 1).await?;

        Ok(crate::utils::pagination::PaginatedResponse {
            data: models.into_iter().map(Self::map_to_response).collect(),
            total,
            page,
            per_page,
            total_pages,
        })
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

    pub async fn delete_subscription(
        db: &DatabaseConnection,
        subscription_id: &str,
        caller_user_id: &str,
        caller_tenant_id: &str,
    ) -> Result<(), ApiError> {
        use sea_orm::EntityTrait;
        use sea_orm::QueryFilter;
        crate::utils::auth::assert_system_admin_access(db, caller_user_id, caller_tenant_id)
            .await?;
        subscription::Entity::find_by_id(subscription_id)
            .one(db)
            .await?
            .ok_or_else(|| ApiError::NotFound("Abonnement introuvable".to_string()))?;
            
        // Explicitly delete all licenses related to this subscription
        crate::models::license::Entity::delete_many()
            .filter(crate::models::license::Column::SubscriptionId.eq(subscription_id))
            .exec(db)
            .await?;

        subscription::Entity::delete_by_id(subscription_id)
            .exec(db)
            .await?;
        Ok(())
    }

    pub async fn update_subscription_status(
        db: &DatabaseConnection,
        subscription_id: &str,
        payload: UpdateSubscriptionStatusPayload,
        caller_user_id: &str,
        caller_tenant_id: &str,
    ) -> Result<SubscriptionResponse, ApiError> {
        use sea_orm::EntityTrait;
        use sea_orm::QueryFilter;
        crate::utils::auth::assert_system_admin_access(db, caller_user_id, caller_tenant_id)
            .await?;
        let sub = subscription::Entity::find_by_id(subscription_id)
            .one(db)
            .await?
            .ok_or_else(|| ApiError::NotFound("Abonnement introuvable".to_string()))?;

        let mut active: subscription::ActiveModel = sub.into();
        active.status = Set(payload.status.clone());
        if payload.status == "cancelled" {
            active.cancelled_at = Set(Some(chrono::Utc::now().fixed_offset()));
        }
        let updated = active.update(db).await?;

        if payload.status == "suspended" || payload.status == "cancelled" {
            let lics = crate::models::license::Entity::find()
                .filter(crate::models::license::Column::SubscriptionId.eq(subscription_id))
                .filter(crate::models::license::Column::IsActive.eq(true))
                .all(db)
                .await?;

            for lic in lics {
                let mut active_lic: crate::models::license::ActiveModel = lic.into();
                active_lic.is_active = Set(Some(false));
                let _ = active_lic.update(db).await;
            }
        }

        Ok(Self::map_to_response(updated))
    }
}
