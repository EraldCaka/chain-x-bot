use crate::models::candle::Candle;

pub struct Wallet {
    pub symbol: String,
    pub initial_balance: f64,
    pub balance_usd: f64,
    pub coin_amount: f64,
    pub last_buy_price: f64,
    pub realized_pnl: f64,
    pub position_open: bool,
    pub stop_loss: f64,
    pub take_profit: f64,
}

impl Wallet {
    pub fn new(initial_balance: f64, symbol: String, _initial_price: String) -> Self {
        Wallet {
            symbol,
            initial_balance,
            balance_usd: initial_balance,
            coin_amount: 0.009,
            last_buy_price: 0.0,
            realized_pnl: 0.0,
            position_open: false,
            stop_loss: 0.0,
            take_profit: 0.0,
        }
    }

    pub fn rsi(&self, closes: &[f64], period: usize) -> f64 {
        if closes.len() <= period {
            return 0.0;
        }

        let mut gains = 0.0;
        let mut losses = 0.0;

        for i in 1..=period {
            let diff = closes[i] - closes[i - 1];
            if diff >= 0.0 {
                gains += diff;
            } else {
                losses -= diff;
            }
        }

        let avg_gain = gains / period as f64;
        let avg_loss = losses / period as f64;

        if avg_loss == 0.0 {
            return 100.0;
        }

        let rs = avg_gain / avg_loss;
        100.0 - (100.0 / (1.0 + rs))
    }

    pub fn macd(&self, closes: &[f64]) -> (f64, f64) {
        if closes.len() < 26 {
            return (0.0, 0.0);
        }

        let ema = |data: &[f64], period: usize| -> Vec<f64> {
            let k = 2.0 / (period as f64 + 1.0);
            let mut ema_values = Vec::new();
            ema_values.push(data[0]);
            for i in 1..data.len() {
                let prev_ema = *ema_values.last().unwrap();
                ema_values.push(data[i] * k + prev_ema * (1.0 - k));
            }
            ema_values
        };

        let ema12 = ema(closes, 12);
        let ema26 = ema(closes, 26);
        let macd_line_series: Vec<f64> =
            ema12.iter().zip(ema26.iter()).map(|(a, b)| a - b).collect();

        let macd_line = *macd_line_series.last().unwrap();
        let signal_line_series = ema(&macd_line_series, 9);
        let signal_line = *signal_line_series.last().unwrap();

        (macd_line, signal_line)
    }

    pub fn stochastic(&self, candles: &[Candle], period: usize) -> f64 {
        if candles.len() < period {
            return 50.0;
        }

        let recent = &candles[candles.len() - period..];
        let high = recent.iter().map(|c| c.high).fold(f64::MIN, f64::max);
        let low = recent.iter().map(|c| c.low).fold(f64::MAX, f64::min);
        let close = recent.last().unwrap().close;

        if high - low == 0.0 {
            50.0
        } else {
            (close - low) / (high - low) * 100.0
        }
    }

    pub fn unrealized_pnl(&self, current_price: f64) -> (f64, f64) {
        if !self.position_open {
            return (0.0, 0.0);
        }

        let unrealized = (current_price - self.last_buy_price) * self.coin_amount;
        let percent = (unrealized / (self.last_buy_price * self.coin_amount)) * 100.0;
        (unrealized, percent)
    }
}
