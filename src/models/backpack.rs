use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug, Clone)]
pub struct ExecuteOrderRequest {
    #[serde(rename = "orderType")]
    pub order_type: String,
    pub side: String,
    pub symbol: String,
    pub quantity: String,
}

#[derive(Deserialize, Debug)]
pub struct ExecuteOrderResponse {
    pub id: Option<String>,
    pub status: Option<String>,
    pub message: Option<String>,
}
