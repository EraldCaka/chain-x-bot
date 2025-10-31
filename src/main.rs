mod config {
    pub mod config;
}

mod services {
    pub mod backpack_api;
    pub mod binance_api;
    pub mod logger;
}

mod models {
    pub mod backpack;
    pub mod candle;
}

mod analysis {
    pub mod wallet;
}

use crate::models::candle::Candle as CandleModel;
use analysis::wallet::Wallet;
use config::config::Config;
use services::backpack_api::Backpack;
use services::binance_api::Binance;
use services::logger::Logger;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_file("config.json")?;
    let mut position: Option<(f64, f64)> = None;
    println!("Starting bot...{:?}", config);
    let backpack = Backpack::new(
        &config.backpack_rest_url,
        &config.api_key,
        &config.api_secret,
    );
    let binance = Binance::new();
    let wallet = Wallet::new(
        config.bot.balance,
        config.bot.symbol.to_string(),
        "0.0".to_string(),
    );

    let logger = Logger::new("trades.log", config.bot.balance);

    loop {
        let binance_candles = match binance.get_historical_klines(
            &config.bot.symbol.to_string(),
            &config.bot.period.to_string(),
            100,
        ) {
            Ok(c) => c,
            Err(err) => {
                eprintln!("Error fetching historical data: {}", err);
                thread::sleep(Duration::from_secs(10));
                continue;
            }
        };

        let candles: Vec<CandleModel> = binance_candles
            .into_iter()
            .map(|c| CandleModel {
                open_time: c.open_time,
                open: c.open,
                high: c.high,
                low: c.low,
                close: c.close,
                volume: c.volume,
                close_time: c.close_time,
            })
            .collect();

        let closes: Vec<f64> = candles.iter().map(|c| c.close).collect();

        let rsi = wallet.rsi(&closes, 14);
        let (macd_line, signal_line) = wallet.macd(&closes);
        let stochastic = wallet.stochastic(&candles, 14);

        println!(
            "Indicators => RSI: {:.2} | MACD: {:.6} | Signal: {:.6} | Stochastic: {:.2}",
            rsi, macd_line, signal_line, stochastic
        );

        let buy_signal = rsi < 35.0 && macd_line > signal_line && stochastic < 20.0;
        let sell_signal = rsi > 65.0 && macd_line < signal_line && stochastic > 80.0;
        let current_price = *closes.last().unwrap_or(&0.0);

        if buy_signal {
            let trade_price = current_price;
            let trade_amount = config.bot.trade;
            logger.log_trade(&config.bot.symbol, "BUY", trade_price, trade_amount, 0.0);
            position = Some((trade_price, trade_amount));
        } else if sell_signal {
            if let Some((buy_price, amount)) = position {
                let pl_percent = ((current_price - buy_price) / buy_price) * 100.0;
                logger.log_trade(
                    &config.bot.symbol,
                    "SELL",
                    current_price,
                    amount,
                    pl_percent,
                );
                position = None;
            }
        }
        thread::sleep(Duration::from_secs(config.bot.recurrence));
    }
}

// match backpack.ask_market("ETH_USDC", "0.0001") {
//     Ok(resp) => println!("Sell order executed: {:?}", resp),
//         Err(err) => eprintln!("Sell order failed: {}", err),
//     }
