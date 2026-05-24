use utoipa::OpenApi;

pub mod role_routes;
pub mod tenant_routes;
pub mod user_routes;
pub mod auth;
pub mod health;
pub mod internal;
pub mod products;
pub mod reports;
pub mod sales;
pub mod stock;
pub mod subscriptions;
pub mod sync;
pub mod licenses;
pub mod gescom;

#[derive(OpenApi)]
#[openapi(
    paths(
        health::health_check,
        auth::login,
        auth::get_profile,
        auth::update_profile,
        auth::forgot_password,
        auth::reset_password,
        auth::verify_otp,
        auth::get_device_key,
        crate::controllers::product_controller::create_product,
        crate::controllers::product_controller::list_products,
        crate::controllers::product_controller::get_product,
        crate::controllers::product_controller::update_product,
        crate::controllers::product_controller::delete_product,
        crate::controllers::role_controller::list_roles,
        crate::controllers::role_controller::get_role,
        crate::controllers::role_controller::create_role,
        crate::controllers::role_controller::update_role,
        crate::controllers::role_controller::delete_role,
        crate::controllers::role_controller::assign_role_permissions,
        crate::controllers::role_controller::list_role_permissions,
        crate::controllers::permission_controller::list_permissions,
        crate::controllers::tenant_controller::get_tenant,
        crate::controllers::tenant_controller::update_tenant,
        crate::controllers::tenant_controller::create_tenant,
        crate::controllers::tenant_controller::list_tenants,
        crate::controllers::tenant_controller::get_tenant_by_id,
        crate::controllers::tenant_controller::delete_tenant,
        crate::controllers::tenant_controller::set_tenant_two_factor,
        crate::controllers::user_controller::list_users,
        crate::controllers::user_controller::create_user,
        crate::controllers::user_controller::set_user_two_factor,
        crate::controllers::subscription_controller::create_subscription,
        crate::controllers::subscription_controller::list_subscriptions,
        crate::controllers::subscription_controller::delete_subscription,
        crate::controllers::subscription_controller::update_subscription_status,
        crate::controllers::license_controller::generate_license,
        crate::controllers::license_controller::list_licenses,
        crate::controllers::license_controller::activate_license,
        crate::controllers::license_controller::get_license_status,
        crate::controllers::license_controller::reveal_license_key,
        crate::controllers::license_controller::send_license_key_email,
        crate::controllers::user_controller::set_user_password,
        crate::controllers::user_controller::send_user_reset,
        crate::controllers::category_controller::list_categories,
        crate::controllers::category_controller::get_category,
        crate::controllers::category_controller::create_category,
        crate::controllers::category_controller::update_category,
        crate::controllers::category_controller::delete_category,
        crate::controllers::stock_controller::create_stock_item,
        crate::controllers::stock_controller::list_stock_items,
        crate::controllers::stock_controller::get_stock_item,
        crate::controllers::stock_controller::update_stock_item,
        crate::controllers::stock_controller::delete_stock_item,
        crate::controllers::stock_controller::create_stock_movement,
        crate::controllers::stock_controller::list_stock_movements,
        // Ventes
        crate::controllers::gescom_controller::create_sale,
        crate::controllers::gescom_controller::list_sales,
        crate::controllers::gescom_controller::get_sale,
        crate::controllers::gescom_controller::void_sale,
        crate::controllers::gescom_controller::refund_sale,
        crate::controllers::gescom_controller::get_sale_receipt,
        // Achats
        crate::controllers::gescom_controller::create_purchase,
        crate::controllers::gescom_controller::list_purchases,
        crate::controllers::gescom_controller::get_purchase,
        crate::controllers::gescom_controller::cancel_purchase,
        // Alertes
        crate::controllers::gescom_controller::list_alerts,
        crate::controllers::gescom_controller::mark_alert_read,
        crate::controllers::gescom_controller::mark_all_alerts_read,
        // Synchronisation
        crate::controllers::gescom_controller::create_sync_log,
        crate::controllers::gescom_controller::list_sync_logs,
    ),
    components(
        schemas(
            health::HealthResponse,
            auth::LoginPayload,
            auth::LoginResponse,
            auth::UserProfile,

            crate::dtos::response_role_dto::RoleResponse,
            crate::dtos::create_role_dto::CreateRolePayload,
            crate::dtos::update_role_dto::UpdateRolePayload,
            crate::dtos::response_role_dto::DeleteRoleResponse,
            crate::controllers::role_controller::AssignRolePermissionsPayload,
            crate::controllers::role_controller::AssignRolePermissionsResponse,
            crate::services::permission_service::PermissionResponse,
            crate::services::permission_service::GroupedPermissionsResponse,
            crate::dtos::tenant_dto::CreateTenantPayload,
            crate::dtos::tenant_dto::UpdateTenantPayload,
            crate::dtos::tenant_dto::SetTenantTwoFactorPayload,
            crate::dtos::tenant_dto::TenantResponse,
            crate::dtos::user_dto::CreateUserPayload,
            crate::dtos::user_dto::SetUserTwoFactorPayload,
            crate::dtos::user_dto::SetUserPasswordPayload,
            crate::dtos::user_dto::SendPasswordResetPayload,
            crate::dtos::user_dto::UserResponse,
            crate::dtos::user_dto::UserProfileTenantResponse,
            crate::dtos::user_dto::UserProfileResponse,
            crate::dtos::user_dto::UpdateProfilePayload,
            crate::dtos::user_dto::PaginatedUserResponse,
            auth::ForgotPasswordPayload,
            auth::ResetPasswordPayload,
            auth::VerifyOtpPayload,
            crate::dtos::subscription_dto::CreateSubscriptionPayload,
            crate::dtos::subscription_dto::UpdateSubscriptionStatusPayload,
            crate::dtos::subscription_dto::SubscriptionResponse,
            crate::dtos::subscription_dto::PaginatedSubscriptionResponse,
            crate::dtos::license_dto::GenerateLicensePayload,
            crate::dtos::license_dto::RevealLicenseResponse,
            crate::dtos::license_dto::ActivateLicensePayload,
            crate::dtos::license_dto::LicenseResponse,
            crate::dtos::license_dto::FullLicenseResponse,
            crate::dtos::license_dto::LicenseStatusResponse,
            crate::dtos::license_dto::PaginatedLicenseResponse,
            crate::dtos::tenant_dto::PaginatedTenantResponse,
            crate::dtos::response_role_dto::PaginatedRoleResponse,
            crate::dtos::category_dto::CreateCategoryPayload,
            crate::dtos::category_dto::UpdateCategoryPayload,
            crate::dtos::category_dto::CategoryResponse,
            crate::dtos::category_dto::PaginatedCategoryResponse,
            crate::dtos::product_dto::CreateProductPayload,
            crate::dtos::product_dto::UpdateProductPayload,
            crate::dtos::product_dto::ProductResponse,
            crate::dtos::product_dto::PaginatedProductResponse,
            crate::dtos::stock_dto::CreateStockItemPayload,
            crate::dtos::stock_dto::UpdateStockItemPayload,
            crate::dtos::stock_dto::StockItemResponse,
            crate::dtos::stock_dto::PaginatedStockItemResponse,
            crate::dtos::stock_dto::CreateStockMovementPayload,
            crate::dtos::stock_dto::StockMovementResponse,
            crate::dtos::stock_dto::PaginatedStockMovementResponse,
            // Gescom — Ventes
            crate::dtos::gescom_dto::CreateSalePayload,
            crate::dtos::gescom_dto::CreateSaleItemPayload,
            crate::dtos::gescom_dto::RefundSalePayload,
            crate::dtos::gescom_dto::RefundItemPayload,
            crate::dtos::gescom_dto::SaleItemResponse,
            crate::dtos::gescom_dto::SaleResponse,
            crate::dtos::gescom_dto::PaginatedSaleResponse,
            crate::dtos::gescom_dto::ReceiptPrintResponse,
            crate::dtos::gescom_dto::ReceiptItemLine,
            // Gescom — Achats
            crate::dtos::gescom_dto::CreatePurchasePayload,
            crate::dtos::gescom_dto::CreatePurchaseItemPayload,
            crate::dtos::gescom_dto::PurchaseItemResponse,
            crate::dtos::gescom_dto::PurchaseResponse,
            crate::dtos::gescom_dto::PaginatedPurchaseResponse,
            // Gescom — Alertes
            crate::dtos::gescom_dto::AlertResponse,
            crate::dtos::gescom_dto::PaginatedAlertResponse,
            // Gescom — Sync
            crate::dtos::gescom_dto::CreateSyncLogPayload,
            crate::dtos::gescom_dto::SyncLogResponse,
            crate::dtos::gescom_dto::PaginatedSyncLogResponse,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Health", description = "Health check and diagnostics"),
        (name = "Auth", description = "Authentication endpoints"),
        (name = "Licenses", description = "License activation & status"),
        (name = "Products", description = "Product catalog management"),
        (name = "Admin - Roles", description = "Tenant roles administration CRUD"),
        (name = "Admin - Tenant", description = "Tenant configuration management"),
        (name = "Admin - Users", description = "Tenant users administration management"),
        (name = "Admin - Subscriptions", description = "Subscription management (system only)"),
        (name = "Admin - Licenses", description = "License generation & management (system only)"),
        (name = "Categories", description = "Product category management"),
        (name = "Stock", description = "Stock items and movements management"),
        (name = "Ventes", description = "Gestion des ventes (enregistrement, annulation, remboursement, reçu)"),
        (name = "Achats", description = "Gestion des approvisionnements fournisseurs"),
        (name = "Alertes", description = "Journal des alertes de pénurie et de stock bas"),
        (name = "Synchronisation", description = "Journal de synchronisation offline/online")
    )
)]
pub struct ApiDoc;

pub struct SecurityAddon;

impl utoipa::Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearerAuth",
                utoipa::openapi::security::SecurityScheme::Http(
                    utoipa::openapi::security::HttpBuilder::new()
                        .scheme(utoipa::openapi::security::HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .build(),
                ),
            );
        }
    }
}
pub mod categories;
