use async_trait::async_trait;
use rustok_core::events::{EventHandler, HandlerResult};
use rustok_events::{DomainEvent, EventEnvelope};
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, FromQueryResult, Statement};
use serde_json::Value as JsonValue;
use tracing::{debug, info, instrument};
use uuid::Uuid;

use crate::error::IndexResult;
use crate::traits::{
    run_bounded_reindex, Indexer, IndexerContext, IndexerRuntimeConfig, LocaleIndexer,
};

#[derive(Debug, FromQueryResult)]
struct ProductRow {
    id: Uuid,
    tenant_id: Uuid,
    status: String,
    vendor: Option<String>,
    metadata: JsonValue,
    published_at: Option<chrono::DateTime<chrono::FixedOffset>>,
    created_at: chrono::DateTime<chrono::FixedOffset>,
    updated_at: chrono::DateTime<chrono::FixedOffset>,
    locale: Option<String>,
    title: Option<String>,
    handle: Option<String>,
    description: Option<String>,
    meta_title: Option<String>,
    meta_description: Option<String>,
}

#[derive(FromQueryResult)]
struct VariantAgg {
    variant_count: i64,
    in_stock: bool,
    total_inventory: i64,
    price_min: Option<i64>,
    price_max: Option<i64>,
}

#[derive(Clone)]
pub struct ProductIndexer {
    db: DatabaseConnection,
    runtime: IndexerRuntimeConfig,
}

impl ProductIndexer {
    pub fn new(db: DatabaseConnection) -> Self {
        Self::with_runtime(db, IndexerRuntimeConfig::load())
    }

    pub fn with_runtime(db: DatabaseConnection, runtime: IndexerRuntimeConfig) -> Self {
        Self { db, runtime }
    }

    fn backend(&self) -> DatabaseBackend {
        self.db.get_database_backend()
    }

    #[instrument(skip(self, ctx))]
    async fn build_index_product(
        &self,
        ctx: &IndexerContext,
        product_id: Uuid,
        locale: &str,
    ) -> IndexResult<Option<super::model::IndexProductModel>> {
        let stmt = Statement::from_sql_and_values(
            self.backend(),
            r#"
            SELECT
                p.id,
                p.tenant_id,
                p.status::text AS status,
                p.vendor,
                p.metadata,
                p.published_at,
                p.created_at,
                p.updated_at,
                pt.locale,
                pt.title,
                pt.handle,
                pt.description,
                pt.meta_title,
                pt.meta_description
            FROM products p
            LEFT JOIN product_translations pt
                ON pt.product_id = p.id AND pt.locale = $3
            WHERE p.id = $1
              AND p.tenant_id = $2
            "#,
            vec![product_id.into(), ctx.tenant_id.into(), locale.into()],
        );

        let row = ProductRow::find_by_statement(stmt)
            .one(&self.db)
            .await
            .map_err(crate::error::IndexError::from)?;

        let row = match row {
            Some(r) => r,
            None => {
                debug!(product_id = %product_id, "Product not found, skipping index");
                return Ok(None);
            }
        };

        let agg_stmt = Statement::from_sql_and_values(
            self.backend(),
            r#"
            SELECT
                COUNT(pv.id)::bigint AS variant_count,
                COALESCE(SUM(inv.available_quantity), 0) > 0 AS in_stock,
                COALESCE(SUM(inv.available_quantity), 0)::bigint AS total_inventory,
                MIN(price_agg.min_amount)::bigint AS price_min,
                MAX(price_agg.max_amount)::bigint AS price_max
            FROM product_variants pv
            LEFT JOIN LATERAL (
                SELECT COALESCE(SUM(il.stocked_quantity - il.reserved_quantity), 0) AS available_quantity
                FROM inventory_items ii
                LEFT JOIN inventory_levels il ON il.inventory_item_id = ii.id
                WHERE ii.variant_id = pv.id
            ) inv ON TRUE
            LEFT JOIN LATERAL (
                SELECT
                    MIN(pr.amount) AS min_amount,
                    MAX(pr.amount) AS max_amount
                FROM prices pr
                WHERE pr.variant_id = pv.id
            ) price_agg ON TRUE
            WHERE pv.product_id = $1
              AND pv.tenant_id = $2
            "#,
            vec![product_id.into(), ctx.tenant_id.into()],
        );

        let agg = VariantAgg::find_by_statement(agg_stmt)
            .one(&self.db)
            .await
            .map_err(crate::error::IndexError::from)?
            .unwrap_or(VariantAgg {
                variant_count: 0,
                in_stock: false,
                total_inventory: 0,
                price_min: None,
                price_max: None,
            });

        let is_published = row.status == "active";

        let model = super::model::IndexProductModel {
            id: Uuid::new_v4(),
            tenant_id: row.tenant_id,
            product_id: row.id,
            locale: row.locale.unwrap_or_else(|| locale.to_string()),
            status: row.status,
            is_published,
            title: row.title.unwrap_or_default(),
            subtitle: None,
            handle: row.handle.unwrap_or_default(),
            description: row.description,
            category_id: None,
            category_name: None,
            category_path: None,
            tags: vec![],
            brand: row.vendor,
            currency: None,
            price_min: agg.price_min,
            price_max: agg.price_max,
            compare_at_price_min: None,
            compare_at_price_max: None,
            on_sale: false,
            in_stock: agg.in_stock,
            total_inventory: i32::try_from(agg.total_inventory).unwrap_or(i32::MAX),
            variant_count: i32::try_from(agg.variant_count).unwrap_or(i32::MAX),
            options: vec![],
            thumbnail_url: None,
            images: vec![],
            meta_title: row.meta_title,
            meta_description: row.meta_description,
            attributes: row.metadata,
            sales_count: 0,
            view_count: 0,
            rating: None,
            review_count: 0,
            published_at: row.published_at.map(|dt| dt.with_timezone(&chrono::Utc)),
            created_at: row.created_at.with_timezone(&chrono::Utc),
            updated_at: row.updated_at.with_timezone(&chrono::Utc),
        };

        Ok(Some(model))
    }

    async fn upsert_index_product(
        &self,
        model: &super::model::IndexProductModel,
    ) -> IndexResult<()> {
        let tags_json = JsonValue::Array(
            model
                .tags
                .iter()
                .map(|t| JsonValue::String(t.clone()))
                .collect(),
        );
        let options_json = serde_json::to_value(&model.options).unwrap_or(JsonValue::Array(vec![]));
        let images_json = serde_json::to_value(&model.images).unwrap_or(JsonValue::Array(vec![]));

        let stmt = Statement::from_sql_and_values(
            self.backend(),
            r#"
            INSERT INTO index_products (
                id, tenant_id, product_id, locale, status, is_published,
                title, subtitle, handle, description,
                category_id, category_name, category_path,
                tags, brand, currency,
                price_min, price_max, compare_at_price_min, compare_at_price_max, on_sale,
                in_stock, total_inventory, variant_count, options,
                thumbnail_url, images,
                meta_title, meta_description, attributes,
                sales_count, view_count, rating, review_count,
                published_at, created_at, updated_at, indexed_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10,
                $11, $12, $13,
                $14, $15, $16,
                $17, $18, $19, $20, $21,
                $22, $23, $24, $25,
                $26, $27,
                $28, $29, $30,
                $31, $32, $33, $34,
                $35, $36, $37, NOW()
            )
            ON CONFLICT (product_id, locale) DO UPDATE SET
                status = EXCLUDED.status,
                is_published = EXCLUDED.is_published,
                title = EXCLUDED.title,
                subtitle = EXCLUDED.subtitle,
                handle = EXCLUDED.handle,
                description = EXCLUDED.description,
                category_id = EXCLUDED.category_id,
                category_name = EXCLUDED.category_name,
                category_path = EXCLUDED.category_path,
                tags = EXCLUDED.tags,
                brand = EXCLUDED.brand,
                currency = EXCLUDED.currency,
                price_min = EXCLUDED.price_min,
                price_max = EXCLUDED.price_max,
                compare_at_price_min = EXCLUDED.compare_at_price_min,
                compare_at_price_max = EXCLUDED.compare_at_price_max,
                on_sale = EXCLUDED.on_sale,
                in_stock = EXCLUDED.in_stock,
                total_inventory = EXCLUDED.total_inventory,
                variant_count = EXCLUDED.variant_count,
                options = EXCLUDED.options,
                thumbnail_url = EXCLUDED.thumbnail_url,
                images = EXCLUDED.images,
                meta_title = EXCLUDED.meta_title,
                meta_description = EXCLUDED.meta_description,
                attributes = EXCLUDED.attributes,
                sales_count = EXCLUDED.sales_count,
                view_count = EXCLUDED.view_count,
                rating = EXCLUDED.rating,
                review_count = EXCLUDED.review_count,
                published_at = EXCLUDED.published_at,
                updated_at = EXCLUDED.updated_at,
                indexed_at = NOW()
            "#,
            vec![
                model.id.into(),
                model.tenant_id.into(),
                model.product_id.into(),
                model.locale.clone().into(),
                model.status.clone().into(),
                model.is_published.into(),
                model.title.clone().into(),
                model.subtitle.clone().into(),
                model.handle.clone().into(),
                model.description.clone().into(),
                model.category_id.into(),
                model.category_name.clone().into(),
                model.category_path.clone().into(),
                tags_json.into(),
                model.brand.clone().into(),
                model.currency.clone().into(),
                model.price_min.into(),
                model.price_max.into(),
                model.compare_at_price_min.into(),
                model.compare_at_price_max.into(),
                model.on_sale.into(),
                model.in_stock.into(),
                model.total_inventory.into(),
                model.variant_count.into(),
                options_json.into(),
                model.thumbnail_url.clone().into(),
                images_json.into(),
                model.meta_title.clone().into(),
                model.meta_description.clone().into(),
                model.attributes.clone().into(),
                model.sales_count.into(),
                model.view_count.into(),
                model.rating.into(),
                model.review_count.into(),
                model.published_at.into(),
                model.created_at.into(),
                model.updated_at.into(),
            ],
        );

        self.db
            .execute(stmt)
            .await
            .map(|_| ())
            .map_err(crate::error::IndexError::from)
    }

    async fn delete_product_from_index(&self, product_id: Uuid) -> IndexResult<()> {
        let stmt = Statement::from_sql_and_values(
            self.backend(),
            "DELETE FROM index_products WHERE product_id = $1",
            vec![product_id.into()],
        );
        self.db
            .execute(stmt)
            .await
            .map(|_| ())
            .map_err(crate::error::IndexError::from)
    }

    async fn get_tenant_locales(&self, ctx: &IndexerContext) -> IndexResult<Vec<String>> {
        #[derive(FromQueryResult)]
        struct LocaleRow {
            locale: String,
        }

        let stmt = Statement::from_sql_and_values(
            self.backend(),
            "SELECT locale FROM tenant_locales WHERE tenant_id = $1",
            vec![ctx.tenant_id.into()],
        );

        let rows = LocaleRow::find_by_statement(stmt)
            .all(&self.db)
            .await
            .unwrap_or_default();

        if rows.is_empty() {
            Ok(vec!["en".to_string()])
        } else {
            Ok(rows.into_iter().map(|r| r.locale).collect())
        }
    }
}

#[async_trait]
impl Indexer for ProductIndexer {
    fn name(&self) -> &'static str {
        "product_indexer"
    }

    #[instrument(skip(self, ctx))]
    async fn index_one(&self, ctx: &IndexerContext, entity_id: Uuid) -> IndexResult<()> {
        let locales = self.get_tenant_locales(ctx).await?;

        for locale in locales {
            let model = self.build_index_product(ctx, entity_id, &locale).await?;
            if let Some(m) = model {
                self.upsert_index_product(&m).await?;
                debug!(product_id = %entity_id, locale = locale, "Indexed product");
            }
        }

        Ok(())
    }

    #[instrument(skip(self, ctx))]
    async fn remove_one(&self, ctx: &IndexerContext, entity_id: Uuid) -> IndexResult<()> {
        let _ = ctx;
        debug!(product_id = %entity_id, "Removing product from index");
        self.delete_product_from_index(entity_id).await
    }

    #[instrument(skip(self, ctx))]
    async fn reindex_all(&self, ctx: &IndexerContext) -> IndexResult<u64> {
        info!(tenant_id = %ctx.tenant_id, "Reindexing all products");

        #[derive(FromQueryResult)]
        struct IdRow {
            id: Uuid,
        }

        let stmt = Statement::from_sql_and_values(
            self.backend(),
            "SELECT id FROM products WHERE tenant_id = $1",
            vec![ctx.tenant_id.into()],
        );

        let rows = IdRow::find_by_statement(stmt)
            .all(&self.db)
            .await
            .unwrap_or_default();

        let ids = rows.into_iter().map(|row| row.id).collect();
        let stats = run_bounded_reindex(self.clone(), ctx, ids, "reindex_all").await;
        Ok(stats.scheduled)
    }
}

#[async_trait]
impl LocaleIndexer for ProductIndexer {
    async fn index_locale(
        &self,
        ctx: &IndexerContext,
        entity_id: Uuid,
        locale: &str,
    ) -> IndexResult<()> {
        let model = self.build_index_product(ctx, entity_id, locale).await?;
        if let Some(m) = model {
            self.upsert_index_product(&m).await?;
        }
        Ok(())
    }

    async fn remove_locale(
        &self,
        _ctx: &IndexerContext,
        entity_id: Uuid,
        locale: &str,
    ) -> IndexResult<()> {
        let stmt = Statement::from_sql_and_values(
            self.backend(),
            "DELETE FROM index_products WHERE product_id = $1 AND locale = $2",
            vec![entity_id.into(), locale.into()],
        );
        self.db
            .execute(stmt)
            .await
            .map(|_| ())
            .map_err(crate::error::IndexError::from)
    }
}

#[async_trait]
impl EventHandler for ProductIndexer {
    fn name(&self) -> &'static str {
        "product_indexer"
    }

    fn handles(&self, event: &DomainEvent) -> bool {
        matches!(
            event,
            DomainEvent::ProductCreated { .. }
                | DomainEvent::ProductUpdated { .. }
                | DomainEvent::ProductPublished { .. }
                | DomainEvent::ProductDeleted { .. }
                | DomainEvent::VariantCreated { .. }
                | DomainEvent::VariantUpdated { .. }
                | DomainEvent::VariantDeleted { .. }
                | DomainEvent::InventoryUpdated { .. }
                | DomainEvent::PriceUpdated { .. }
        ) || matches!(
            event,
            DomainEvent::ReindexRequested { target_type, .. } if target_type == "product"
        )
    }

    async fn handle(&self, envelope: &EventEnvelope) -> HandlerResult {
        let ctx = IndexerContext::new_with_runtime(
            self.db.clone(),
            envelope.tenant_id,
            self.runtime.clone(),
        );

        match &envelope.event {
            DomainEvent::ProductCreated { product_id }
            | DomainEvent::ProductUpdated { product_id }
            | DomainEvent::ProductPublished { product_id } => {
                self.index_one(&ctx, *product_id).await?;
            }

            DomainEvent::ProductDeleted { product_id } => {
                self.remove_one(&ctx, *product_id).await?;
            }

            DomainEvent::VariantCreated { product_id, .. }
            | DomainEvent::VariantUpdated { product_id, .. }
            | DomainEvent::VariantDeleted { product_id, .. } => {
                self.index_one(&ctx, *product_id).await?;
            }

            DomainEvent::InventoryUpdated { product_id, .. } => {
                self.index_one(&ctx, *product_id).await?;
            }

            DomainEvent::PriceUpdated { product_id, .. } => {
                self.index_one(&ctx, *product_id).await?;
            }

            DomainEvent::ReindexRequested { target_id, .. } => {
                if let Some(id) = target_id {
                    self.index_one(&ctx, *id).await?;
                } else {
                    self.reindex_all(&ctx).await?;
                }
            }

            _ => {}
        }

        Ok(())
    }
}
