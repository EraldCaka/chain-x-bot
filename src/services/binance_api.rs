use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug)]
pub struct TickerPrice {
    pub symbol: String,
    pub price: String,
}

#[derive(Debug, Clone)]
pub struct Candle {
    pub open_time: u64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub close_time: u64,
}

pub struct Binance {
    client: Client,
}

impl Binance {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub fn get_ticker(&self, symbol: &str) -> Result<TickerPrice, Box<dyn std::error::Error>> {
        let url = format!(
            "https://api.binance.com/api/v3/ticker/price?symbol={}",
            symbol
        );
        let resp = self.client.get(&url).send()?.json::<TickerPrice>()?;
        Ok(resp)
    }

    pub fn get_historical_klines(
        &self,
        symbol: &str,
        interval: &str,
        limit: usize,
    ) -> Result<Vec<Candle>, Box<dyn std::error::Error>> {
        let url = format!(
            "https://api.binance.com/api/v3/klines?symbol={}&interval={}&limit={}",
            symbol, interval, limit
        );

        let resp = self.client.get(&url).send()?.json::<Vec<Value>>()?;

        let candles: Vec<Candle> = resp
            .into_iter()
            .filter_map(|kline| {
                Some(Candle {
                    open_time: kline[0].as_u64()?,
                    open: kline[1].as_str()?.parse().ok()?,
                    high: kline[2].as_str()?.parse().ok()?,
                    low: kline[3].as_str()?.parse().ok()?,
                    close: kline[4].as_str()?.parse().ok()?,
                    volume: kline[5].as_str()?.parse().ok()?,
                    close_time: kline[6].as_u64()?,
                })
            })
            .collect();

        Ok(candles)
    }
}
