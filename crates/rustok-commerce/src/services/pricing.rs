use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set, TransactionTrait,
};
use tracing::instrument;
use uuid::Uuid;

use rustok_core::{generate_id, DomainEvent};
use rustok_outbox::TransactionalEventBus;

use crate::dto::PriceInput;
use crate::entities;
use crate::error::{CommerceError, CommerceResult};

pub struct PricingService {
    db: DatabaseConnection,
    event_bus: TransactionalEventBus,
}

impl PricingService {
    pub fn new(db: DatabaseConnection, event_bus: TransactionalEventBus) -> Self {
        Self { db, event_bus }
    }

    #[instrument(skip(self))]
    pub async fn set_price(
        &self,
        tenant_id: Uuid,
        actor_id: Uuid,
        variant_id: Uuid,
        currency_code: &str,
        amount: Decimal,
        compare_at_amount: Option<Decimal>,
    ) -> CommerceResult<()> {
        let txn = self.db.begin().await?;
        
        let variant = entities::product_variant::Entity::find_by_id(variant_id)
            .filter(entities::product_variant::Column::TenantId.eq(tenant_id))
            .one(&txn)
            .await?
            .ok_or(CommerceError::VariantNotFound(variant_id))?;

        if amount < Decimal::ZERO {
            return Err(CommerceError::InvalidPrice(
                "Amount cannot be negative".into(),
            ));
        }
        if let Some(compare_at) = compare_at_amount {
            if compare_at < amount {
                return Err(CommerceError::InvalidPrice(
                    "Compare at price must be greater than amount".into(),
                ));
            }
        }

        let existing = entities::price::Entity::find()
            .filter(entities::price::Column::VariantId.eq(variant_id))
            .filter(entities::price::Column::CurrencyCode.eq(currency_code))
            .one(&txn)
            .await?;

        let old_amount = existing.as_ref().map(|price| price.amount);

        match existing {
            Some(price) => {
                let mut price_active: entities::price::ActiveModel = price.into();
                price_active.amount = Set(amount);
                price_active.compare_at_amount = Set(compare_at_amount);
                price_active.update(&txn).await?;
            }
            None => {
                let price = entities::price::ActiveModel {
                    id: Set(generate_id()),
                    variant_id: Set(variant_id),
                    currency_code: Set(currency_code.to_string()),
                    amount: Set(amount),
                    compare_at_amount: Set(compare_at_amount),
                };
                price.insert(&txn).await?;
            }
        }

        let old_cents = old_amount.and_then(decimal_to_cents);
        let new_cents = decimal_to_cents(amount).unwrap_or(0);

        // Create and validate event
        let event = DomainEvent::PriceUpdated {
            variant_id,
            product_id: variant.product_id,
            currency: currency_code.to_string(),
            old_amount: old_cents,
            new_amount: new_cents,
        };
        event.validate()
            .map_err(|e| CommerceError::Validation(format!("Invalid price event: {}", e)))?;

        self.event_bus
            .publish_in_tx(&txn, tenant_id, Some(actor_id), event)
            .await?;

        txn.commit().await?;
        Ok(())
    }

    #[instrument(skip(self, prices))]
    pub async fn set_prices(
        &self,
        tenant_id: Uuid,
        actor_id: Uuid,
        variant_id: Uuid,
        prices: Vec<PriceInput>,
    ) -> CommerceResult<()> {
        for price_input in prices {
            self.set_price(
                tenant_id,
                actor_id,
                variant_id,
                &price_input.currency_code,
                price_input.amount,
                price_input.compare_at_amount,
            )
            .await?;
        }
        Ok(())
    }

    #[instrument(skip(self))]
    pub async fn get_price(
        &self,
        variant_id: Uuid,
        currency_code: &str,
    ) -> CommerceResult<Option<Decimal>> {
        let price = entities::price::Entity::find()
            .filter(entities::price::Column::VariantId.eq(variant_id))
            .filter(entities::price::Column::CurrencyCode.eq(currency_code))
            .one(&self.db)
            .await?;

        Ok(price.map(|price| price.amount))
    }

    #[instrument(skip(self))]
    pub async fn get_variant_prices(
        &self,
        variant_id: Uuid,
    ) -> CommerceResult<Vec<entities::price::Model>> {
        let prices = entities::price::Entity::find()
            .filter(entities::price::Column::VariantId.eq(variant_id))
            .all(&self.db)
            .await?;

        Ok(prices)
    }

    #[instrument(skip(self))]
    pub async fn apply_discount(
        &self,
        tenant_id: Uuid,
        actor_id: Uuid,
        variant_id: Uuid,
        currency_code: &str,
        discount_percent: Decimal,
    ) -> CommerceResult<Decimal> {
        let price = entities::price::Entity::find()
            .filter(entities::price::Column::VariantId.eq(variant_id))
            .filter(entities::price::Column::CurrencyCode.eq(currency_code))
            .one(&self.db)
            .await?
            .ok_or_else(|| {
                CommerceError::InvalidPrice(format!(
                    "No price found for currency {}",
                    currency_code
                ))
            })?;

        let original_amount = price.compare_at_amount.unwrap_or(price.amount);
        let discount_multiplier = (Decimal::from(100) - discount_percent) / Decimal::from(100);
        let new_amount = (original_amount * discount_multiplier).round_dp(2);

        self.set_price(
            tenant_id,
            actor_id,
            variant_id,
            currency_code,
            new_amount,
            Some(original_amount),
        )
        .await?;

        Ok(new_amount)
    }
}

fn decimal_to_cents(amount: Decimal) -> Option<i64> {
    (amount * Decimal::from(100)).round_dp(0).to_i64()
}
