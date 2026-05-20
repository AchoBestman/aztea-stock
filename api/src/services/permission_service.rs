use crate::{
    errors::ApiError,
    models::{permission, tenant}
};
use sea_orm::{DatabaseConnection, EntityTrait, Order, QueryOrder, ColumnTrait, QueryFilter};
use std::collections::BTreeMap;

#[derive(serde::Serialize, utoipa::ToSchema, Clone, Debug)]
pub struct PermissionResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(serde::Serialize, utoipa::ToSchema, Clone, Debug)]
pub struct GroupedPermissionsResponse {
    pub group: String,
    pub permissions: Vec<PermissionResponse>,
}

pub struct PermissionService;

impl PermissionService {
    pub async fn list_grouped_permissions(
        db: &DatabaseConnection,
        caller_tenant_id: &str,
    ) -> Result<Vec<GroupedPermissionsResponse>, ApiError> {
        // Vérifier si le tenant est système
        let caller_tenant = tenant::Entity::find_by_id(caller_tenant_id)
            .one(db)
            .await?
            .ok_or_else(|| ApiError::NotFound("Tenant introuvable".to_string()))?;

        let mut query = permission::Entity::find();

        // Si ce n'est pas un tenant système, on filtre les permissions d'administration globale
        if !caller_tenant.is_system {
            query = query.filter(
                sea_orm::Condition::all()
                    .add(permission::Column::ModelGroup.ne("tenants"))
                    .add(permission::Column::ModelGroup.ne("cross-tenant"))
                    .add(permission::Column::ModelGroup.ne("licenses"))
                    .add(permission::Column::ModelGroup.ne("subscriptions"))
            );
        }

        let perms = query
            .order_by(permission::Column::ModelGroup, Order::Asc)
            .order_by(permission::Column::Name, Order::Asc)
            .all(db)
            .await?;

        // Group by model_group using a BTreeMap to maintain sorted order of groups
        let mut grouped: BTreeMap<String, Vec<PermissionResponse>> = BTreeMap::new();

        for p in perms {
            grouped.entry(p.model_group).or_default().push(PermissionResponse {
                id: p.id,
                name: p.name,
                description: p.description,
            });
        }

        let response = grouped
            .into_iter()
            .map(|(group, permissions)| GroupedPermissionsResponse {
                group,
                permissions,
            })
            .collect();

        Ok(response)
    }
}
