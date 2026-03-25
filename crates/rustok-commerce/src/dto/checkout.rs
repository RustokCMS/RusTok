use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use validator::Validate;

use crate::{CartResponse, FulfillmentResponse, OrderResponse, PaymentCollectionResponse};

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct CompleteCheckoutInput {
    pub cart_id: Uuid,
    #[validate(length(min = 1, max = 100))]
    pub payment_provider_id: String,
    #[validate(length(min = 1, max = 191))]
    pub provider_payment_id: String,
    pub shipping_option_id: Option<Uuid>,
    #[serde(default = "default_true")]
    pub create_fulfillment: bool,
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteCheckoutResponse {
    pub cart: CartResponse,
    pub order: OrderResponse,
    pub payment_collection: PaymentCollectionResponse,
    pub fulfillment: Option<FulfillmentResponse>,
}

const fn default_true() -> bool {
    true
}
