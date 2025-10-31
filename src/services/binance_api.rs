use reqwest::blocking::Client;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct TickerPrice {
    pub symbol: String,
    pub price: String,
}

pub struct Binance {}

impl Binance {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_ticker(&self, symbol: &str) -> Result<TickerPrice, Box<dyn std::error::Error>> {
        let client = Client::new();
        let url = format!(
            "https://api.binance.com/api/v3/ticker/price?symbol={}",
            symbol
        );
        let resp = client.get(&url).send()?.json::<TickerPrice>()?;
        Ok(resp)
    }
}
