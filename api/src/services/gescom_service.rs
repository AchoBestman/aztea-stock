use crate::{
    dtos::gescom_dto::{
        AlertResponse, CreatePurchasePayload, CreateSalePayload, CreateSyncLogPayload,
        PurchaseItemResponse, PurchaseResponse, ReceiptItemLine, ReceiptPrintResponse,
        RefundSalePayload, SaleItemResponse, SaleResponse, SyncLogResponse,
    },
    errors::ApiError,
    models::{purchase_items, purchases, sale_items, sales},
    repositories::{
        gescom_repository::GescomRepository, product_repository::ProductRepository,
        stock_repository::StockRepository, user_repository::UserRepository,
    },
};
use sea_orm::{DatabaseConnection, Set, TransactionTrait};
use validator::Validate;

pub struct GescomService;

impl GescomService {
    // --- Sales ---

    pub async fn create_sale(
        db: &DatabaseConnection,
        caller_user_id: &str,
        caller_tenant_id: &str,
        payload: CreateSalePayload,
    ) -> Result<SaleResponse, ApiError> {
        payload
            .validate()
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;

        if payload.items.is_empty() {
            return Err(ApiError::BadRequest(
                "Une vente doit contenir au moins une ligne d'article.".to_string(),
            ));
        }

        // Validate payment method
        match payload.payment_method.as_str() {
            "cash" | "card" | "mobile_money" | "credit" => {}
            _ => {
                return Err(ApiError::BadRequest(
                    "Mode de paiement invalide.".to_string(),
                ));
            }
        }

        let txn = db.begin().await.map_err(|e| ApiError::Database(e))?;

        let sale_id = uuid::Uuid::new_v4().to_string();
        let mut subtotal = 0.0;
        let mut tax_total = 0.0;
        let mut discount_total = 0.0;

        let mut sale_items_to_insert = Vec::new();

        // 1. Process items and verify stock
        for item in &payload.items {
            let prod = ProductRepository::find_by_id(&txn, &item.product_id, caller_tenant_id)
                .await?
                .ok_or_else(|| {
                    ApiError::BadRequest(format!(
                        "Produit spécifié introuvable : {}",
                        item.product_id
                    ))
                })?;

            if item.quantity <= 0.0 {
                return Err(ApiError::BadRequest(
                    "La quantité doit être strictement positive.".to_string(),
                ));
            }
            if item.unit_price < 0.0 {
                return Err(ApiError::BadRequest(
                    "Le prix unitaire doit être positif.".to_string(),
                ));
            }

            let tax_rate = item.tax_rate.unwrap_or(0.0);
            let discount = item.discount.unwrap_or(0.0);
            let line_subtotal = item.quantity * item.unit_price;
            let line_tax = line_subtotal * (tax_rate / 100.0);
            let line_total = line_subtotal + line_tax - discount;

            subtotal += line_subtotal;
            tax_total += line_tax;
            discount_total += discount;

            let sale_item_id = uuid::Uuid::new_v4().to_string();
            let active_item = sale_items::ActiveModel {
                id: Set(sale_item_id),
                tenant_id: Set(caller_tenant_id.to_string()),
                sale_id: Set(sale_id.clone()),
                product_id: Set(item.product_id.clone()),
                product_name: Set(prod.name.clone()),
                product_barcode: Set(prod.barcode.clone()),
                quantity: Set(item.quantity),
                unit_price: Set(item.unit_price),
                tax_rate: Set(tax_rate),
                discount: Set(discount),
                line_total: Set(line_total),
            };

            sale_items_to_insert.push((active_item, prod.clone()));

            // 2. Adjust inventory
            let stock = match StockRepository::find_stock_item_by_product_id(
                &txn,
                &item.product_id,
                caller_tenant_id,
            )
            .await?
            {
                Some(s) => s,
                None => {
                    // Create empty stock item if it does not exist
                    let id = uuid::Uuid::new_v4().to_string();
                    StockRepository::create_stock_item(
                        &txn,
                        &id,
                        caller_tenant_id,
                        &item.product_id,
                        0.0,
                        0.0,
                        5.0,
                        None,
                        None,
                        None,
                    )
                    .await?
                }
            };

            let qty_before = stock.quantity;
            let qty_after = qty_before - item.quantity;

            if qty_after < 0.0 {
                return Err(ApiError::BadRequest(format!(
                    "Stock insuffisant pour le produit {}.",
                    prod.name
                )));
            }

            // Update stock quantity
            StockRepository::update_stock_item(
                &txn,
                &stock.id,
                caller_tenant_id,
                Some(qty_after),
                Some(stock.quantity_reserved),
                Some(stock.low_stock_threshold),
                None,
                None,
                None,
            )
            .await?;

            // Create stock movement
            let movement_id = uuid::Uuid::new_v4().to_string();
            StockRepository::create_stock_movement(
                &txn,
                &movement_id,
                caller_tenant_id,
                &item.product_id,
                Some(caller_user_id.to_string()),
                "sale",
                qty_before,
                -item.quantity,
                qty_after,
                Some(sale_id.clone()),
                Some(format!("Vente liée #{}", sale_id)),
            )
            .await?;

            // Trigger alerts
            if qty_after <= 0.0 {
                GescomRepository::create_alert(
                    &txn,
                    caller_tenant_id,
                    Some(item.product_id.clone()),
                    "out_of_stock",
                    &format!("Rupture de stock complète pour le produit : {}.", prod.name),
                    Some(stock.low_stock_threshold),
                    Some(qty_after),
                )
                .await?;
            } else if qty_after <= stock.low_stock_threshold {
                GescomRepository::create_alert(
                    &txn,
                    caller_tenant_id,
                    Some(item.product_id.clone()),
                    "low_stock",
                    &format!(
                        "Alerte stock bas pour le produit: {} (Quantité restante: {}).",
                        prod.name, qty_after
                    ),
                    Some(stock.low_stock_threshold),
                    Some(qty_after),
                )
                .await?;
            }
        }

        let total = subtotal + tax_total - discount_total;
        let amount_paid = payload.amount_paid.unwrap_or(total);
        let change_given = payload.change_given.unwrap_or(0.0);

        if payload.payment_method == "cash" && amount_paid + 0.001 < total {
            return Err(ApiError::BadRequest(
                "Le montant reçu doit couvrir le total pour un paiement en espèces.".to_string(),
            ));
        }

        let receipt_number = format!(
            "REC-{}-{}",
            chrono::Utc::now().format("%Y%m%d%H%M"),
            rand::random::<u16>()
        );
        let sold_at: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

        let active_sale = sales::ActiveModel {
            id: Set(sale_id.clone()),
            tenant_id: Set(caller_tenant_id.to_string()),
            user_id: Set(Some(caller_user_id.to_string())),
            receipt_number: Set(receipt_number),
            customer_name: Set(payload.customer_name),
            customer_phone: Set(payload.customer_phone),
            subtotal: Set(subtotal),
            tax_total: Set(tax_total),
            discount_total: Set(discount_total),
            total: Set(total),
            amount_paid: Set(amount_paid),
            change_given: Set(change_given),
            payment_method: Set(payload.payment_method),
            status: Set("completed".to_string()),
            notes: Set(payload.notes),
            sold_at: Set(sold_at),
            created_at: Set(chrono::Utc::now().into()),
        };

        let sale = GescomRepository::create_sale(&txn, active_sale).await?;

        let mut item_responses = Vec::new();
        for (active_item, _) in sale_items_to_insert {
            let inserted = GescomRepository::create_sale_item(&txn, active_item).await?;
            item_responses.push(SaleItemResponse {
                id: inserted.id,
                product_id: inserted.product_id,
                product_name: inserted.product_name,
                product_barcode: inserted.product_barcode,
                quantity: inserted.quantity,
                unit_price: inserted.unit_price,
                tax_rate: inserted.tax_rate,
                discount: inserted.discount,
                line_total: inserted.line_total,
            });
        }

        txn.commit().await.map_err(|e| ApiError::Database(e))?;

        Ok(SaleResponse {
            id: sale.id,
            tenant_id: sale.tenant_id,
            user_id: sale.user_id,
            receipt_number: sale.receipt_number,
            customer_name: sale.customer_name,
            customer_phone: sale.customer_phone,
            subtotal: sale.subtotal,
            tax_total: sale.tax_total,
            discount_total: sale.discount_total,
            total: sale.total,
            amount_paid: sale.amount_paid,
            change_given: sale.change_given,
            payment_method: sale.payment_method,
            status: sale.status,
            notes: sale.notes,
            sold_at: sale.sold_at,
            created_at: sale.created_at,
            items: item_responses,
        })
    }

    pub async fn get_sale(
        db: &DatabaseConnection,
        id: &str,
        caller_user_id: &str,
        caller_tenant_id: &str,
    ) -> Result<SaleResponse, ApiError> {
        let sale = GescomRepository::find_sale_by_id(db, id, caller_tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Vente introuvable".to_string()))?;

        crate::utils::auth::require_tenant_access(
            db,
            caller_tenant_id,
            &sale.tenant_id,
            caller_user_id,
            "read",
        )
        .await?;

        let items = GescomRepository::find_sale_items(db, &sale.id).await?;
        let item_responses = items
            .into_iter()
            .map(|i| SaleItemResponse {
                id: i.id,
                product_id: i.product_id,
                product_name: i.product_name,
                product_barcode: i.product_barcode,
                quantity: i.quantity,
                unit_price: i.unit_price,
                tax_rate: i.tax_rate,
                discount: i.discount,
                line_total: i.line_total,
            })
            .collect();

        Ok(SaleResponse {
            id: sale.id,
            tenant_id: sale.tenant_id,
            user_id: sale.user_id,
            receipt_number: sale.receipt_number,
            customer_name: sale.customer_name,
            customer_phone: sale.customer_phone,
            subtotal: sale.subtotal,
            tax_total: sale.tax_total,
            discount_total: sale.discount_total,
            total: sale.total,
            amount_paid: sale.amount_paid,
            change_given: sale.change_given,
            payment_method: sale.payment_method,
            status: sale.status,
            notes: sale.notes,
            sold_at: sale.sold_at,
            created_at: sale.created_at,
            items: item_responses,
        })
    }

    pub async fn list_sales(
        db: &DatabaseConnection,
        caller_user_id: &str,
        caller_tenant_id: &str,
        customer_name: Option<String>,
        status: Option<String>,
        params: crate::utils::pagination::PaginationParams,
    ) -> Result<crate::utils::pagination::PaginatedResponse<SaleResponse>, ApiError> {
        let final_tenant_id = if let Some(ref t_id) = params.tenant_id {
            crate::utils::auth::require_tenant_access(
                db,
                caller_tenant_id,
                t_id,
                caller_user_id,
                "read",
            )
            .await?;
            t_id.clone()
        } else {
            caller_tenant_id.to_string()
        };

        let paginated = GescomRepository::find_sales_paginated(
            db,
            &final_tenant_id,
            customer_name,
            status,
            params,
        )
        .await?;

        let mut data = Vec::with_capacity(paginated.data.len());
        for sale in paginated.data {
            let items = GescomRepository::find_sale_items(db, &sale.id).await?;
            let item_responses = items
                .into_iter()
                .map(|i| SaleItemResponse {
                    id: i.id,
                    product_id: i.product_id,
                    product_name: i.product_name,
                    product_barcode: i.product_barcode,
                    quantity: i.quantity,
                    unit_price: i.unit_price,
                    tax_rate: i.tax_rate,
                    discount: i.discount,
                    line_total: i.line_total,
                })
                .collect();

            data.push(SaleResponse {
                id: sale.id,
                tenant_id: sale.tenant_id,
                user_id: sale.user_id,
                receipt_number: sale.receipt_number,
                customer_name: sale.customer_name,
                customer_phone: sale.customer_phone,
                subtotal: sale.subtotal,
                tax_total: sale.tax_total,
                discount_total: sale.discount_total,
                total: sale.total,
                amount_paid: sale.amount_paid,
                change_given: sale.change_given,
                payment_method: sale.payment_method,
                status: sale.status,
                notes: sale.notes,
                sold_at: sale.sold_at,
                created_at: sale.created_at,
                items: item_responses,
            });
        }

        Ok(crate::utils::pagination::PaginatedResponse {
            data,
            total: paginated.total,
            page: paginated.page,
            per_page: paginated.per_page,
            total_pages: paginated.total_pages,
        })
    }

    pub async fn void_sale(
        db: &DatabaseConnection,
        id: &str,
        caller_user_id: &str,
        caller_tenant_id: &str,
    ) -> Result<SaleResponse, ApiError> {
        let sale = GescomRepository::find_sale_by_id(db, id, caller_tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Vente introuvable".to_string()))?;

        crate::utils::auth::require_tenant_access(
            db,
            caller_tenant_id,
            &sale.tenant_id,
            caller_user_id,
            "update",
        )
        .await?;

        if sale.status == "voided" {
            return Err(ApiError::BadRequest(
                "Cette vente est déjà annulée.".to_string(),
            ));
        }

        let txn = db.begin().await.map_err(|e| ApiError::Database(e))?;

        // 1. Update sale status to voided
        let updated_sale =
            GescomRepository::update_sale_status(&txn, id, caller_tenant_id, "voided").await?;

        // 2. Restore stock for each item
        let items = GescomRepository::find_sale_items(&txn, id).await?;
        for item in &items {
            let stock = match StockRepository::find_stock_item_by_product_id(
                &txn,
                &item.product_id,
                caller_tenant_id,
            )
            .await?
            {
                Some(s) => s,
                None => {
                    let stock_id = uuid::Uuid::new_v4().to_string();
                    StockRepository::create_stock_item(
                        &txn,
                        &stock_id,
                        caller_tenant_id,
                        &item.product_id,
                        0.0,
                        0.0,
                        5.0,
                        None,
                        None,
                        None,
                    )
                    .await?
                }
            };

            let qty_before = stock.quantity;
            let qty_after = qty_before + item.quantity;

            StockRepository::update_stock_item(
                &txn,
                &stock.id,
                caller_tenant_id,
                Some(qty_after),
                Some(stock.quantity_reserved),
                Some(stock.low_stock_threshold),
                None,
                None,
                None,
            )
            .await?;

            // Create movement
            let movement_id = uuid::Uuid::new_v4().to_string();
            StockRepository::create_stock_movement(
                &txn,
                &movement_id,
                caller_tenant_id,
                &item.product_id,
                Some(caller_user_id.to_string()),
                "return", // Or adjustment
                qty_before,
                item.quantity,
                qty_after,
                Some(id.to_string()),
                Some(format!("Annulation de la vente #{}", sale.receipt_number)),
            )
            .await?;
        }

        txn.commit().await.map_err(|e| ApiError::Database(e))?;

        let item_responses = items
            .into_iter()
            .map(|i| SaleItemResponse {
                id: i.id,
                product_id: i.product_id,
                product_name: i.product_name,
                product_barcode: i.product_barcode,
                quantity: i.quantity,
                unit_price: i.unit_price,
                tax_rate: i.tax_rate,
                discount: i.discount,
                line_total: i.line_total,
            })
            .collect();

        Ok(SaleResponse {
            id: updated_sale.id,
            tenant_id: updated_sale.tenant_id,
            user_id: updated_sale.user_id,
            receipt_number: updated_sale.receipt_number,
            customer_name: updated_sale.customer_name,
            customer_phone: updated_sale.customer_phone,
            subtotal: updated_sale.subtotal,
            tax_total: updated_sale.tax_total,
            discount_total: updated_sale.discount_total,
            total: updated_sale.total,
            amount_paid: updated_sale.amount_paid,
            change_given: updated_sale.change_given,
            payment_method: updated_sale.payment_method,
            status: updated_sale.status,
            notes: updated_sale.notes,
            sold_at: updated_sale.sold_at,
            created_at: updated_sale.created_at,
            items: item_responses,
        })
    }

    pub async fn refund_sale(
        db: &DatabaseConnection,
        id: &str,
        caller_user_id: &str,
        caller_tenant_id: &str,
        payload: RefundSalePayload,
    ) -> Result<SaleResponse, ApiError> {
        payload
            .validate()
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;

        let sale = GescomRepository::find_sale_by_id(db, id, caller_tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Vente introuvable".to_string()))?;

        crate::utils::auth::require_tenant_access(
            db,
            caller_tenant_id,
            &sale.tenant_id,
            caller_user_id,
            "update",
        )
        .await?;

        if sale.status == "voided" {
            return Err(ApiError::BadRequest(
                "Impossible de rembourser une vente annulée.".to_string(),
            ));
        }

        let txn = db.begin().await.map_err(|e| ApiError::Database(e))?;

        // 1. Update sale status to refunded
        let updated_sale =
            GescomRepository::update_sale_status(&txn, id, caller_tenant_id, "refunded").await?;

        // 2. Process refund quantities and restore stock
        let sale_items = GescomRepository::find_sale_items(&txn, id).await?;
        for refund_item in &payload.refund_items {
            let original_item = sale_items
                .iter()
                .find(|i| i.product_id == refund_item.product_id)
                .ok_or_else(|| {
                    ApiError::BadRequest(format!(
                        "Le produit {} ne fait pas partie de cette vente.",
                        refund_item.product_id
                    ))
                })?;

            if refund_item.quantity <= 0.0 || refund_item.quantity > original_item.quantity {
                return Err(ApiError::BadRequest(
                    "Quantité de remboursement invalide.".to_string(),
                ));
            }

            let stock = match StockRepository::find_stock_item_by_product_id(
                &txn,
                &refund_item.product_id,
                caller_tenant_id,
            )
            .await?
            {
                Some(s) => s,
                None => {
                    let stock_id = uuid::Uuid::new_v4().to_string();
                    StockRepository::create_stock_item(
                        &txn,
                        &stock_id,
                        caller_tenant_id,
                        &refund_item.product_id,
                        0.0,
                        0.0,
                        5.0,
                        None,
                        None,
                        None,
                    )
                    .await?
                }
            };

            let qty_before = stock.quantity;
            let qty_after = qty_before + refund_item.quantity;

            StockRepository::update_stock_item(
                &txn,
                &stock.id,
                caller_tenant_id,
                Some(qty_after),
                Some(stock.quantity_reserved),
                Some(stock.low_stock_threshold),
                None,
                None,
                None,
            )
            .await?;

            // Create movement
            let movement_id = uuid::Uuid::new_v4().to_string();
            StockRepository::create_stock_movement(
                &txn,
                &movement_id,
                caller_tenant_id,
                &refund_item.product_id,
                Some(caller_user_id.to_string()),
                "return",
                qty_before,
                refund_item.quantity,
                qty_after,
                Some(id.to_string()),
                Some(format!(
                    "Remboursement partiel/total de la vente #{}",
                    sale.receipt_number
                )),
            )
            .await?;
        }

        txn.commit().await.map_err(|e| ApiError::Database(e))?;

        let item_responses = sale_items
            .into_iter()
            .map(|i| SaleItemResponse {
                id: i.id,
                product_id: i.product_id,
                product_name: i.product_name,
                product_barcode: i.product_barcode,
                quantity: i.quantity,
                unit_price: i.unit_price,
                tax_rate: i.tax_rate,
                discount: i.discount,
                line_total: i.line_total,
            })
            .collect();

        Ok(SaleResponse {
            id: updated_sale.id,
            tenant_id: updated_sale.tenant_id,
            user_id: updated_sale.user_id,
            receipt_number: updated_sale.receipt_number,
            customer_name: updated_sale.customer_name,
            customer_phone: updated_sale.customer_phone,
            subtotal: updated_sale.subtotal,
            tax_total: updated_sale.tax_total,
            discount_total: updated_sale.discount_total,
            total: updated_sale.total,
            amount_paid: updated_sale.amount_paid,
            change_given: updated_sale.change_given,
            payment_method: updated_sale.payment_method,
            status: updated_sale.status,
            notes: updated_sale.notes,
            sold_at: updated_sale.sold_at,
            created_at: updated_sale.created_at,
            items: item_responses,
        })
    }

    pub async fn get_sale_receipt(
        db: &DatabaseConnection,
        id: &str,
        caller_user_id: &str,
        caller_tenant_id: &str,
    ) -> Result<ReceiptPrintResponse, ApiError> {
        let sale = Self::get_sale(db, id, caller_user_id, caller_tenant_id).await?;

        let u_name = if let Some(ref uid) = sale.user_id {
            UserRepository::find_by_id(db, uid, caller_tenant_id)
                .await?
                .map(|u| u.name)
        } else {
            None
        };

        let items = sale
            .items
            .iter()
            .map(|i| ReceiptItemLine {
                name: i.product_name.clone(),
                qty: i.quantity,
                price: i.unit_price,
                total: i.line_total,
            })
            .collect();

        Ok(ReceiptPrintResponse {
            receipt_number: sale.receipt_number,
            date: sale.sold_at,
            cashier: u_name,
            customer_name: sale.customer_name,
            subtotal: sale.subtotal,
            tax_total: sale.tax_total,
            discount_total: sale.discount_total,
            total: sale.total,
            payment_method: sale.payment_method,
            lines: items,
            footer_note: Some("Merci pour votre visite ! À bientôt.".to_string()),
        })
    }

    /// Export all sales for a tenant, with optional date range filtering.
    /// Used for PDF/Excel export endpoints.
    pub async fn export_sales(
        db: &DatabaseConnection,
        caller_user_id: &str,
        caller_tenant_id: &str,
        target_tenant_id: Option<String>,
        start_date: Option<String>,
        end_date: Option<String>,
    ) -> Result<Vec<SaleResponse>, ApiError> {
        let final_tenant_id = if let Some(ref t_id) = target_tenant_id {
            crate::utils::auth::require_tenant_access(
                db,
                caller_tenant_id,
                t_id,
                caller_user_id,
                "read",
            )
            .await?;
            t_id.clone()
        } else {
            caller_tenant_id.to_string()
        };

        // Use a very large per_page to get all records for export
        let params = crate::utils::pagination::PaginationParams {
            page: Some(1),
            per_page: Some(100_000),
            search: None,
            start_date,
            end_date,
            tenant_id: Some(final_tenant_id.clone()),
            order_by: None,
            order_type: None,
        };

        let paginated =
            GescomRepository::find_sales_paginated(db, &final_tenant_id, None, None, params)
                .await?;

        let mut data = Vec::with_capacity(paginated.data.len());
        for sale in paginated.data {
            let items = GescomRepository::find_sale_items(db, &sale.id).await?;
            let item_responses = items
                .into_iter()
                .map(|i| SaleItemResponse {
                    id: i.id,
                    product_id: i.product_id,
                    product_name: i.product_name,
                    product_barcode: i.product_barcode,
                    quantity: i.quantity,
                    unit_price: i.unit_price,
                    tax_rate: i.tax_rate,
                    discount: i.discount,
                    line_total: i.line_total,
                })
                .collect();

            data.push(SaleResponse {
                id: sale.id,
                tenant_id: sale.tenant_id,
                user_id: sale.user_id,
                receipt_number: sale.receipt_number,
                customer_name: sale.customer_name,
                customer_phone: sale.customer_phone,
                subtotal: sale.subtotal,
                tax_total: sale.tax_total,
                discount_total: sale.discount_total,
                total: sale.total,
                amount_paid: sale.amount_paid,
                change_given: sale.change_given,
                payment_method: sale.payment_method,
                status: sale.status,
                notes: sale.notes,
                sold_at: sale.sold_at,
                created_at: sale.created_at,
                items: item_responses,
            });
        }

        Ok(data)
    }

    // --- Purchases ---

    pub async fn create_purchase(
        db: &DatabaseConnection,
        caller_user_id: &str,
        caller_tenant_id: &str,
        payload: CreatePurchasePayload,
    ) -> Result<PurchaseResponse, ApiError> {
        payload
            .validate()
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;

        if payload.items.is_empty() {
            return Err(ApiError::BadRequest(
                "Un achat doit contenir au moins une ligne d'article.".to_string(),
            ));
        }

        let txn = db.begin().await.map_err(|e| ApiError::Database(e))?;

        let purchase_id = uuid::Uuid::new_v4().to_string();
        let mut total = 0.0;
        let mut purchase_items_to_insert = Vec::new();

        for item in &payload.items {
            let prod = ProductRepository::find_by_id(&txn, &item.product_id, caller_tenant_id)
                .await?
                .ok_or_else(|| {
                    ApiError::BadRequest(format!(
                        "Produit spécifié introuvable : {}",
                        item.product_id
                    ))
                })?;

            if item.quantity <= 0.0 {
                return Err(ApiError::BadRequest(
                    "La quantité doit être strictement positive.".to_string(),
                ));
            }
            if item.unit_cost < 0.0 {
                return Err(ApiError::BadRequest(
                    "Le coût unitaire doit être positif.".to_string(),
                ));
            }

            let line_total = item.quantity * item.unit_cost;
            total += line_total;

            let purchase_item_id = uuid::Uuid::new_v4().to_string();
            let active_item = purchase_items::ActiveModel {
                id: Set(purchase_item_id),
                tenant_id: Set(caller_tenant_id.to_string()),
                purchase_id: Set(purchase_id.clone()),
                product_id: Set(item.product_id.clone()),
                quantity: Set(item.quantity),
                unit_cost: Set(item.unit_cost),
                expiry_date: Set(item.expiry_date.clone()),
                batch_number: Set(item.batch_number.clone()),
                line_total: Set(line_total),
            };

            purchase_items_to_insert.push((active_item, prod));

            // Adjust inventory (Increment Stock)
            let stock = match StockRepository::find_stock_item_by_product_id(
                &txn,
                &item.product_id,
                caller_tenant_id,
            )
            .await?
            {
                Some(s) => s,
                None => {
                    let id = uuid::Uuid::new_v4().to_string();
                    StockRepository::create_stock_item(
                        &txn,
                        &id,
                        caller_tenant_id,
                        &item.product_id,
                        0.0,
                        0.0,
                        5.0,
                        None,
                        None,
                        None,
                    )
                    .await?
                }
            };

            let qty_before = stock.quantity;
            let qty_after = qty_before + item.quantity;

            let expiry_date = item
                .expiry_date
                .as_ref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok());

            StockRepository::update_stock_item(
                &txn,
                &stock.id,
                caller_tenant_id,
                Some(qty_after),
                Some(stock.quantity_reserved),
                Some(stock.low_stock_threshold),
                None,
                Some(item.batch_number.clone()),
                Some(expiry_date),
            )
            .await?;

            // Create movement
            let movement_id = uuid::Uuid::new_v4().to_string();
            StockRepository::create_stock_movement(
                &txn,
                &movement_id,
                caller_tenant_id,
                &item.product_id,
                Some(caller_user_id.to_string()),
                "purchase",
                qty_before,
                item.quantity,
                qty_after,
                Some(purchase_id.clone()),
                Some(format!("Approvisionnement lié #{}", purchase_id)),
            )
            .await?;
        }

        let purchased_at: chrono::DateTime<chrono::FixedOffset> = chrono::Utc::now().into();

        let active_purchase = purchases::ActiveModel {
            id: Set(purchase_id.clone()),
            tenant_id: Set(caller_tenant_id.to_string()),
            user_id: Set(Some(caller_user_id.to_string())),
            supplier_name: Set(payload.supplier_name),
            supplier_phone: Set(payload.supplier_phone),
            reference: Set(payload.reference),
            total: Set(total),
            status: Set("received".to_string()),
            notes: Set(payload.notes),
            purchased_at: Set(purchased_at),
            created_at: Set(chrono::Utc::now().into()),
        };

        let purchase = GescomRepository::create_purchase(&txn, active_purchase).await?;

        let mut item_responses = Vec::new();
        for (active_item, prod) in purchase_items_to_insert {
            let inserted = GescomRepository::create_purchase_item(&txn, active_item).await?;
            item_responses.push(PurchaseItemResponse {
                id: inserted.id,
                product_id: inserted.product_id,
                product_name: prod.name,
                quantity: inserted.quantity,
                unit_cost: inserted.unit_cost,
                expiry_date: inserted.expiry_date,
                batch_number: inserted.batch_number,
                line_total: inserted.line_total,
            });
        }

        txn.commit().await.map_err(|e| ApiError::Database(e))?;

        Ok(PurchaseResponse {
            id: purchase.id,
            tenant_id: purchase.tenant_id,
            user_id: purchase.user_id,
            supplier_name: purchase.supplier_name,
            supplier_phone: purchase.supplier_phone,
            reference: purchase.reference,
            total: purchase.total,
            status: purchase.status,
            notes: purchase.notes,
            purchased_at: purchase.purchased_at,
            created_at: purchase.created_at,
            items: item_responses,
        })
    }

    pub async fn get_purchase(
        db: &DatabaseConnection,
        id: &str,
        caller_user_id: &str,
        caller_tenant_id: &str,
    ) -> Result<PurchaseResponse, ApiError> {
        let purchase = GescomRepository::find_purchase_by_id(db, id, caller_tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Achat introuvable".to_string()))?;

        crate::utils::auth::require_tenant_access(
            db,
            caller_tenant_id,
            &purchase.tenant_id,
            caller_user_id,
            "read",
        )
        .await?;

        let items = GescomRepository::find_purchase_items(db, &purchase.id).await?;
        let mut item_responses = Vec::new();
        for i in items {
            let prod_name = ProductRepository::find_by_id(db, &i.product_id, caller_tenant_id)
                .await?
                .map(|p| p.name)
                .unwrap_or_default();
            item_responses.push(PurchaseItemResponse {
                id: i.id,
                product_id: i.product_id,
                product_name: prod_name,
                quantity: i.quantity,
                unit_cost: i.unit_cost,
                expiry_date: i.expiry_date,
                batch_number: i.batch_number,
                line_total: i.line_total,
            });
        }

        Ok(PurchaseResponse {
            id: purchase.id,
            tenant_id: purchase.tenant_id,
            user_id: purchase.user_id,
            supplier_name: purchase.supplier_name,
            supplier_phone: purchase.supplier_phone,
            reference: purchase.reference,
            total: purchase.total,
            status: purchase.status,
            notes: purchase.notes,
            purchased_at: purchase.purchased_at,
            created_at: purchase.created_at,
            items: item_responses,
        })
    }

    pub async fn list_purchases(
        db: &DatabaseConnection,
        caller_user_id: &str,
        caller_tenant_id: &str,
        supplier_name: Option<String>,
        status: Option<String>,
        params: crate::utils::pagination::PaginationParams,
    ) -> Result<crate::utils::pagination::PaginatedResponse<PurchaseResponse>, ApiError> {
        let final_tenant_id = if let Some(ref t_id) = params.tenant_id {
            crate::utils::auth::require_tenant_access(
                db,
                caller_tenant_id,
                t_id,
                caller_user_id,
                "read",
            )
            .await?;
            t_id.clone()
        } else {
            caller_tenant_id.to_string()
        };

        let paginated = GescomRepository::find_purchases_paginated(
            db,
            &final_tenant_id,
            supplier_name,
            status,
            params,
        )
        .await?;

        let mut data = Vec::with_capacity(paginated.data.len());
        for p in paginated.data {
            let items = GescomRepository::find_purchase_items(db, &p.id).await?;
            let mut item_responses = Vec::new();
            for i in items {
                let prod_name = ProductRepository::find_by_id(db, &i.product_id, &final_tenant_id)
                    .await?
                    .map(|prod| prod.name)
                    .unwrap_or_default();
                item_responses.push(PurchaseItemResponse {
                    id: i.id,
                    product_id: i.product_id,
                    product_name: prod_name,
                    quantity: i.quantity,
                    unit_cost: i.unit_cost,
                    expiry_date: i.expiry_date,
                    batch_number: i.batch_number,
                    line_total: i.line_total,
                });
            }

            data.push(PurchaseResponse {
                id: p.id,
                tenant_id: p.tenant_id,
                user_id: p.user_id,
                supplier_name: p.supplier_name,
                supplier_phone: p.supplier_phone,
                reference: p.reference,
                total: p.total,
                status: p.status,
                notes: p.notes,
                purchased_at: p.purchased_at,
                created_at: p.created_at,
                items: item_responses,
            });
        }

        Ok(crate::utils::pagination::PaginatedResponse {
            data,
            total: paginated.total,
            page: paginated.page,
            per_page: paginated.per_page,
            total_pages: paginated.total_pages,
        })
    }

    pub async fn cancel_purchase(
        db: &DatabaseConnection,
        id: &str,
        caller_user_id: &str,
        caller_tenant_id: &str,
    ) -> Result<PurchaseResponse, ApiError> {
        let purchase = GescomRepository::find_purchase_by_id(db, id, caller_tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Achat introuvable".to_string()))?;

        crate::utils::auth::require_tenant_access(
            db,
            caller_tenant_id,
            &purchase.tenant_id,
            caller_user_id,
            "update",
        )
        .await?;

        if purchase.status == "cancelled" {
            return Err(ApiError::BadRequest(
                "Cet achat est déjà annulé.".to_string(),
            ));
        }

        let txn = db.begin().await.map_err(|e| ApiError::Database(e))?;

        // 1. Update purchase status
        let updated =
            GescomRepository::update_purchase_status(&txn, id, caller_tenant_id, "cancelled")
                .await?;

        // 2. Decrement stock
        let items = GescomRepository::find_purchase_items(&txn, id).await?;
        for item in &items {
            let stock = match StockRepository::find_stock_item_by_product_id(
                &txn,
                &item.product_id,
                caller_tenant_id,
            )
            .await?
            {
                Some(s) => s,
                None => {
                    let stock_id = uuid::Uuid::new_v4().to_string();
                    StockRepository::create_stock_item(
                        &txn,
                        &stock_id,
                        caller_tenant_id,
                        &item.product_id,
                        0.0,
                        0.0,
                        5.0,
                        None,
                        None,
                        None,
                    )
                    .await?
                }
            };

            let qty_before = stock.quantity;
            let qty_after = qty_before - item.quantity;

            if qty_after < 0.0 {
                return Err(ApiError::BadRequest(
                    "Impossible d'annuler cet achat car le stock deviendrait négatif.".to_string(),
                ));
            }

            StockRepository::update_stock_item(
                &txn,
                &stock.id,
                caller_tenant_id,
                Some(qty_after),
                Some(stock.quantity_reserved),
                Some(stock.low_stock_threshold),
                None,
                None,
                None,
            )
            .await?;

            // Create movement
            let movement_id = uuid::Uuid::new_v4().to_string();
            StockRepository::create_stock_movement(
                &txn,
                &movement_id,
                caller_tenant_id,
                &item.product_id,
                Some(caller_user_id.to_string()),
                "loss", // or adjustment
                qty_before,
                -item.quantity,
                qty_after,
                Some(id.to_string()),
                Some(format!(
                    "Annulation de l'approvisionnement #{}",
                    purchase.id
                )),
            )
            .await?;
        }

        txn.commit().await.map_err(|e| ApiError::Database(e))?;

        let mut item_responses = Vec::new();
        for i in items {
            let prod_name = ProductRepository::find_by_id(db, &i.product_id, caller_tenant_id)
                .await?
                .map(|p| p.name)
                .unwrap_or_default();
            item_responses.push(PurchaseItemResponse {
                id: i.id,
                product_id: i.product_id,
                product_name: prod_name,
                quantity: i.quantity,
                unit_cost: i.unit_cost,
                expiry_date: i.expiry_date,
                batch_number: i.batch_number,
                line_total: i.line_total,
            });
        }

        Ok(PurchaseResponse {
            id: updated.id,
            tenant_id: updated.tenant_id,
            user_id: updated.user_id,
            supplier_name: updated.supplier_name,
            supplier_phone: updated.supplier_phone,
            reference: updated.reference,
            total: updated.total,
            status: updated.status,
            notes: updated.notes,
            purchased_at: updated.purchased_at,
            created_at: updated.created_at,
            items: item_responses,
        })
    }

    // --- Alerts ---

    pub async fn list_alerts(
        db: &DatabaseConnection,
        caller_user_id: &str,
        caller_tenant_id: &str,
        is_read: Option<bool>,
        alert_type: Option<String>,
        params: crate::utils::pagination::PaginationParams,
    ) -> Result<crate::utils::pagination::PaginatedResponse<AlertResponse>, ApiError> {
        let final_tenant_id = if let Some(ref t_id) = params.tenant_id {
            crate::utils::auth::require_tenant_access(
                db,
                caller_tenant_id,
                t_id,
                caller_user_id,
                "read",
            )
            .await?;
            t_id.clone()
        } else {
            caller_tenant_id.to_string()
        };

        let paginated = GescomRepository::find_alerts_paginated(
            db,
            &final_tenant_id,
            is_read,
            alert_type,
            params,
        )
        .await?;

        let mut data = Vec::with_capacity(paginated.data.len());
        for alert in paginated.data {
            let prod_name = if let Some(ref pid) = alert.product_id {
                ProductRepository::find_by_id(db, pid, &final_tenant_id)
                    .await?
                    .map(|p| p.name)
            } else {
                None
            };

            data.push(AlertResponse {
                id: alert.id,
                tenant_id: alert.tenant_id,
                product_id: alert.product_id,
                product_name: prod_name,
                alert_type: alert.alert_type,
                message: alert.message,
                threshold: alert.threshold,
                current_qty: alert.current_qty,
                is_read: alert.is_read,
                is_resolved: alert.is_resolved,
                triggered_at: alert.triggered_at,
            });
        }

        Ok(crate::utils::pagination::PaginatedResponse {
            data,
            total: paginated.total,
            page: paginated.page,
            per_page: paginated.per_page,
            total_pages: paginated.total_pages,
        })
    }

    pub async fn mark_alert_read(
        db: &DatabaseConnection,
        id: &str,
        caller_user_id: &str,
        caller_tenant_id: &str,
    ) -> Result<AlertResponse, ApiError> {
        let alert = GescomRepository::find_alert_by_id(db, id, caller_tenant_id)
            .await?
            .ok_or_else(|| ApiError::NotFound("Alerte introuvable".to_string()))?;

        crate::utils::auth::require_tenant_access(
            db,
            caller_tenant_id,
            &alert.tenant_id,
            caller_user_id,
            "update",
        )
        .await?;

        let updated = GescomRepository::mark_alert_read(db, id, caller_tenant_id).await?;

        let prod_name = if let Some(ref pid) = updated.product_id {
            ProductRepository::find_by_id(db, pid, caller_tenant_id)
                .await?
                .map(|p| p.name)
        } else {
            None
        };

        Ok(AlertResponse {
            id: updated.id,
            tenant_id: updated.tenant_id,
            product_id: updated.product_id,
            product_name: prod_name,
            alert_type: updated.alert_type,
            message: updated.message,
            threshold: updated.threshold,
            current_qty: updated.current_qty,
            is_read: updated.is_read,
            is_resolved: updated.is_resolved,
            triggered_at: updated.triggered_at,
        })
    }

    pub async fn mark_all_alerts_read(
        db: &DatabaseConnection,
        caller_user_id: &str,
        caller_tenant_id: &str,
    ) -> Result<u64, ApiError> {
        // Must have permission to manage alerts
        crate::utils::auth::require_permission(db, caller_user_id, "can_manage_alert").await?;
        GescomRepository::mark_all_alerts_read(db, caller_tenant_id).await
    }

    // --- Sync Log ---

    pub async fn create_sync_log(
        db: &DatabaseConnection,
        _caller_user_id: &str,
        caller_tenant_id: &str,
        payload: CreateSyncLogPayload,
    ) -> Result<SyncLogResponse, ApiError> {
        payload
            .validate()
            .map_err(|e| ApiError::BadRequest(e.to_string()))?;

        let inserted = GescomRepository::create_sync_log(
            db,
            caller_tenant_id,
            &payload.device_id,
            Some(payload.sync_type),
            Some(payload.status),
            payload.records_pushed,
            payload.records_pulled,
            payload.error_message,
        )
        .await?;

        Ok(SyncLogResponse {
            id: inserted.id,
            tenant_id: inserted.tenant_id,
            device_id: inserted.device_id,
            sync_type: inserted.sync_type,
            status: inserted.status,
            records_pushed: inserted.records_pushed,
            records_pulled: inserted.records_pulled,
            error_message: inserted.error_message,
            started_at: inserted.started_at,
            finished_at: inserted.finished_at,
        })
    }

    pub async fn list_sync_logs(
        db: &DatabaseConnection,
        caller_user_id: &str,
        caller_tenant_id: &str,
        device_id: Option<String>,
        params: crate::utils::pagination::PaginationParams,
    ) -> Result<crate::utils::pagination::PaginatedResponse<SyncLogResponse>, ApiError> {
        let final_tenant_id = if let Some(ref t_id) = params.tenant_id {
            crate::utils::auth::require_tenant_access(
                db,
                caller_tenant_id,
                t_id,
                caller_user_id,
                "read",
            )
            .await?;
            t_id.clone()
        } else {
            caller_tenant_id.to_string()
        };

        let paginated =
            GescomRepository::find_sync_logs_paginated(db, &final_tenant_id, device_id, params)
                .await?;

        let data = paginated
            .data
            .into_iter()
            .map(|log| SyncLogResponse {
                id: log.id,
                tenant_id: log.tenant_id,
                device_id: log.device_id,
                sync_type: log.sync_type,
                status: log.status,
                records_pushed: log.records_pushed,
                records_pulled: log.records_pulled,
                error_message: log.error_message,
                started_at: log.started_at,
                finished_at: log.finished_at,
            })
            .collect();

        Ok(crate::utils::pagination::PaginatedResponse {
            data,
            total: paginated.total,
            page: paginated.page,
            per_page: paginated.per_page,
            total_pages: paginated.total_pages,
        })
    }
}
