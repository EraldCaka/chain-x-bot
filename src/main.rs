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
    // let mut last_trade_time = Instant::now() - Duration::from_secs(600);
    // let trade_cooldown = Duration::from_secs(config.bot.timeout);

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
        let trade_amount = config.bot.trade;
        let has_open_position = wallet.coin_amount > 0.0;
        let bullish_cross = macd_line > signal_line;
        let bearish_cross = macd_line < signal_line;

        // println!(
        //     "rsi: {} bullish {} bearish {} stochastic {}",
        //     rsi, bullish_cross, bearish_cross, stochastic
        // );
        //
        let buy_signal = rsi < 40.0 && bullish_cross && stochastic < 40.0 && stochastic > 5.0;
        let sell_signal = rsi > 60.0 && bearish_cross && stochastic > 75.0 && stochastic < 95.0;
        let buy_price = current_price;
        let risk = buy_price * 0.005; // 0.5%

        if has_open_position {
            if current_price <= wallet.stop_loss {
                let loss = (current_price - wallet.last_buy_price) * wallet.coin_amount;
                let percent = (loss / (wallet.last_buy_price * wallet.coin_amount)) * 100.0;
                wallet.realized_pnl += loss;
                wallet.balance_usd += wallet.coin_amount * current_price;
                logger.log_trade(
                    &config.bot.symbol,
                    "STOP LOSS",
                    current_price,
                    wallet.coin_amount,
                    percent,
                );
                match backpack
                    .ask_market(&config.bot.backpack_symbol, &config.bot.trade.to_string())
                {
                    Ok(resp) => println!("Stop loss executed @ {:.2}: {:?}", current_price, resp),
                    Err(err) => eprintln!("Stop loss sell failed: {}", err),
                }
            } else if current_price >= wallet.take_profit && sell_signal {
                let profit = (current_price - wallet.last_buy_price) * wallet.coin_amount;
                let percent = (profit / (wallet.last_buy_price * wallet.coin_amount)) * 100.0;
                wallet.realized_pnl += profit;
                wallet.balance_usd += wallet.coin_amount * current_price;
                logger.log_trade(
                    &config.bot.symbol,
                    "TAKE PROFIT",
                    current_price,
                    config.bot.trade,
                    percent,
                );
                match backpack
                    .ask_market(&config.bot.backpack_symbol, &config.bot.trade.to_string())
                {
                    Ok(resp) => println!("Take profit executed @ {:.2}: {:?}", current_price, resp),
                    Err(err) => eprintln!("Take profit sell failed: {}", err),
                }
            }
        }

        if buy_signal {
            let buy_amount = trade_amount;
            let cost = buy_price * buy_amount;
            if wallet.balance_usd >= cost {
                wallet.take_profit = buy_price + (risk * 2.0);
                wallet.last_buy_price = buy_price;
                wallet.coin_amount = buy_amount;
                wallet.balance_usd -= cost;
                wallet.stop_loss = buy_price - risk;
                logger.log_trade(&config.bot.symbol, "BUY", buy_price, buy_amount, 0.0);
                match backpack
                    .bid_market(&config.bot.backpack_symbol, &config.bot.trade.to_string())
                {
                    Ok(resp) => println!("Buy executed @ {:.2}: {:?}", buy_price, resp),
                    Err(err) => eprintln!("Buy failed: {}", err),
                }
            } else {
                println!("Not enough balance to buy");
            }
        }
        thread::sleep(Duration::from_secs(config.bot.recurrence));
    }
}
