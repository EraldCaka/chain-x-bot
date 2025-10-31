use crate::models::backpack::{ExecuteOrderRequest, ExecuteOrderResponse};
use base64::{engine::general_purpose, Engine as _};
use ed25519_dalek::{Signature, Signer, SigningKey};
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Backpack {
    client: Client,
    base_url: String,
    api_key: String,
    private_key: SigningKey,
}

impl Backpack {
    pub fn new(base_url: &str, api_key: &str, private_key_base64: &str) -> Self {
        let private_key_bytes = general_purpose::STANDARD
            .decode(private_key_base64)
            .expect("Invalid private key");
        let private_key = SigningKey::from_bytes(&private_key_bytes.try_into().unwrap());

        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
            api_key: api_key.to_string(),
            private_key,
        }
    }

    fn current_timestamp() -> String {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string()
    }

    fn build_batch_signing_message(
        &self,
        orders: &[ExecuteOrderRequest],
        timestamp: &str,
        window: &str,
    ) -> String {
        let mut parts: Vec<String> = vec![];
        for order in orders {
            let order_parts = vec![
                format!("instruction=orderExecute"),
                format!("orderType={}", order.order_type),
                format!("quantity={}", order.quantity),
                format!("side={}", order.side),
                format!("symbol={}", order.symbol),
            ];
            parts.push(order_parts.join("&"));
        }
        let mut signing_string = parts.join("&");
        signing_string.push_str(&format!("&timestamp={}&window={}", timestamp, window));
        signing_string
    }

    fn generate_signature(&self, message: &str) -> String {
        let signature: Signature = self.private_key.sign(message.as_bytes());
        general_purpose::STANDARD.encode(signature.to_bytes())
    }

    pub fn execute_order(
        &self,
        order: &ExecuteOrderRequest,
    ) -> Result<ExecuteOrderResponse, reqwest::Error> {
        let url = format!("{}/orders", self.base_url);

        let timestamp = Self::current_timestamp();
        let window = "5000";

        let signing_string = self.build_batch_signing_message(&[order.clone()], &timestamp, window);
        let signature_b64 = self.generate_signature(&signing_string);

        let mut headers = HeaderMap::new();
        headers.insert("X-API-KEY", HeaderValue::from_str(&self.api_key).unwrap());
        headers.insert(
            "X-SIGNATURE",
            HeaderValue::from_str(&signature_b64).unwrap(),
        );
        headers.insert("X-TIMESTAMP", HeaderValue::from_str(&timestamp).unwrap());
        headers.insert("X-WINDOW", HeaderValue::from_static(window));
        headers.insert("X-BROKER-ID", HeaderValue::from_static("1"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        // println!("\n[SIGN MESSAGE]: {}", signing_string);
        // println!("[SIGNATURE]: {}", signature_b64);
        // println!(
        //     "[REQUEST BODY]: {}",
        //     serde_json::to_string_pretty(&[order]).unwrap()
        // );

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&[order])
            .send()?;

        let status = response.status();
        let text = response.text()?;

        println!("\n[STATUS]: {:?}", status);
        // println!("[RESPONSE]: {}", text);

        let parsed =
            serde_json::from_str::<ExecuteOrderResponse>(&text).unwrap_or(ExecuteOrderResponse {
                id: None,
                status: None,
                message: Some(text),
            });

        Ok(parsed)
    }

    pub fn bid_market(
        &self,
        symbol: &str,
        quantity: &str,
    ) -> Result<ExecuteOrderResponse, reqwest::Error> {
        let order = ExecuteOrderRequest {
            order_type: "Market".to_string(),
            side: "Bid".to_string(),
            symbol: symbol.to_string(),
            quantity: quantity.to_string(),
        };

        self.execute_order(&order)
    }

    pub fn ask_market(
        &self,
        symbol: &str,
        quantity: &str,
    ) -> Result<ExecuteOrderResponse, reqwest::Error> {
        let order = ExecuteOrderRequest {
            order_type: "Market".to_string(),
            side: "Ask".to_string(),
            symbol: symbol.to_string(),
            quantity: quantity.to_string(),
        };

        self.execute_order(&order)
    }
}
