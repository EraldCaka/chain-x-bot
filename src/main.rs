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
use std::time::{Duration, Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_file("config.json")?;
    // println!("Starting bot...{:?}", config);

    let backpack = Backpack::new(
        &config.backpack_rest_url,
        &config.api_key,
        &config.api_secret,
    );
    let binance = Binance::new();
    let ticker = binance.get_ticker(&config.bot.symbol.to_string())?;
    // print!("ticker {:?}", ticker);
    let mut wallet = Wallet::new(
        config.bot.balance,
        config.bot.symbol.to_string(),
        ticker.price.to_string(),
    );

    let logger = Logger::new("trades.log", config.bot.balance);
    let mut last_trade_time = Instant::now() - Duration::from_secs(600);
    let trade_cooldown = Duration::from_secs(config.bot.timeout);

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
            "RSI: {:.2} | MACD: {:.6} | Signal: {:.6} | Stochastic: {:.2}",
            rsi, macd_line, signal_line, stochastic
        );

        let current_price = *closes.last().unwrap_or(&0.0);
        let trade_amount = 0.0001;

        let bullish_cross = macd_line > signal_line;
        let bearish_cross = macd_line < signal_line;

        let buy_signal = rsi < 40.0 && bullish_cross && stochastic < 51.0 && stochastic > 5.0;
        let sell_signal = rsi > 60.0 && bearish_cross && stochastic > 75.0 && stochastic < 95.0;

        if last_trade_time.elapsed() >= trade_cooldown {
            if buy_signal && !wallet.position_open {
                let buy_price = current_price;
                let buy_amount = trade_amount;

                wallet.coin_amount = buy_amount;
                wallet.last_buy_price = buy_price;
                wallet.balance_usd = 0.0;
                wallet.position_open = true;
                logger.log_trade(&config.bot.symbol, "BUY", buy_price, buy_amount, 0.0);

                match backpack.bid_market(&config.bot.backpack_symbol, &buy_amount.to_string()) {
                    Ok(resp) => println!("Buy executed @ {:.2}: {:?}", buy_price, resp),
                    Err(err) => eprintln!("Buy failed: {}", err),
                }
                last_trade_time = Instant::now();
            } else if sell_signal && wallet.position_open {
                let sell_price = current_price;
                let sell_value = wallet.coin_amount * sell_price;
                let buy_value = wallet.coin_amount * wallet.last_buy_price;

                let profit = sell_value - buy_value;
                let percent = (profit / buy_value) * 100.0;

                wallet.realized_pnl += profit;
                wallet.balance_usd = sell_value;
                wallet.coin_amount = 0.0;
                wallet.position_open = false;

                logger.log_trade(&config.bot.symbol, "SELL", sell_price, 0.0, profit);

                match backpack.ask_market(&config.bot.backpack_symbol, &trade_amount.to_string()) {
                    Ok(resp) => println!("Sell executed: {:?}", resp),
                    Err(err) => eprintln!("Sell failed: {}", err),
                }

                last_trade_time = Instant::now();
            }
        } else {
            let remaining = trade_cooldown.as_secs() - last_trade_time.elapsed().as_secs();
            println!("Cooldown active — next trade in {}s", remaining);
        }

        thread::sleep(Duration::from_secs(config.bot.recurrence));
    }
}
